#![allow(non_snake_case)]
use actix;
use std::env;

use near_client::Query;
use near_client::ViewClientActor;
use near_indexer::near_primitives::types::BlockReference;
use near_indexer::near_primitives::types::FunctionArgs;
use near_indexer::near_primitives::views::ExecutionStatusView;
use near_indexer::near_primitives::views::QueryRequest;
use near_sdk::AccountId;

use std::str::FromStr;

use futures::{join, StreamExt};

use clap::Clap;
use tokio::sync::mpsc;

use actix::Addr;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use near_sdk::json_types::U128;

mod configs;
mod database;

pub type FungibleTokenId = AccountId;
pub type SaleConditions = HashMap<FungibleTokenId, U128>;

//use this struct to store information that we want to pass to database
#[derive(Debug, Clone)] //derive debug so that we can print
struct ExecutionDetails {
    method_name: String,
    args: serde_json::Value,
    signer_id: String,
    deposit: u128,
    success_value: bool,
    transaction_id: String,
    predecessor_id: String,
    receiver_id: String,
}

//declare struct for the return type of the blockchain view call
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub charity_account_id: String,
    pub artist_account_id: Option<String>,
    pub copies: Option<u64>,
}

//declare struct for the return type of the blockchain view call
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonToken {
    pub owner_id: String, //only declaring the field we care about
    pub metadata: TokenMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub ft_token_id: AccountId,
    pub price: Option<U128>,
}

//declare struct for storing sales conditions. useful for parsing json from execution details
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SaleConditions,
}

pub(crate) fn human(yocto: u128) -> f64 {
    let foo = yocto as f64 / 1000000000000000000000000_f64;
    foo
}

async fn handle_messages(
    streamer_message: near_indexer::StreamerMessage,
    view_client: Addr<ViewClientActor>,
    nft_contract: String,
    market_contract: String,
    admin_account: String,
    public_api_root: String,
    private_api_root: String, 
    signature_header: String,
    debug_mode: String,
) {
    //iterate through each shard in the incoming stream
    for shard in streamer_message.shards {
        //for each receipt and execution outcome pair in the shard
        for receipt_and_execution_outcome in shard.receipt_execution_outcomes {
            // Check if receipt is related to Fayyr
            if is_valid_receipt(
                &receipt_and_execution_outcome.receipt,
                nft_contract.clone(),
                market_contract.clone(),
            ) {
                //get the execution outcome from the receipt and execution outcome pair from the shard
                let execution_outcome = receipt_and_execution_outcome.execution_outcome;
                //only do stuff with the receipts if the outcome was successful
                if matches!(
                    execution_outcome.outcome.status,
                    ExecutionStatusView::SuccessValue(_) | ExecutionStatusView::SuccessReceiptId(_)
                ) {
                    //declare values for the execution details that will be used for this entire loop
                    let mut method_name_ = "".to_string(); 
                    let mut args_ = serde_json::Value::String("".to_string()); 
                    let signer_id_ = if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        ref signer_id,
                        ..
                    } = receipt_and_execution_outcome.receipt.receipt
                    {
                        signer_id.to_string()
                    } else {
                        "".to_string()
                    }; 
                    let mut deposit_ = 0; 
                    let success_value_ = true; 
                    let transaction_id_ = execution_outcome.id.to_string(); 
                    let predecessor_id_ = receipt_and_execution_outcome
                        .receipt
                        .predecessor_id
                        .to_string(); 
                    let receiver_id_ = receipt_and_execution_outcome
                        .receipt
                        .receiver_id
                        .to_string();
                    
                    //created the vector of execution details associated with this receipt. 
                    //it will be greater than length 1 IF the receipt contains a batch txn.
                    let mut execution_details_vector: Vec<ExecutionDetails> = vec![];
                    
                    //get the actions from the receipt
                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        actions,
                        ..
                    } = receipt_and_execution_outcome.receipt.receipt
                    {
                        //go through each action
                        for action in actions.iter() {
                            //get the args from the action
                            match action {
                                near_indexer::near_primitives::views::ActionView::FunctionCall {
                                    args,
                                    method_name,
                                    deposit,
                                    ..
                                } => {
                                    //decode the args
                                    if let Ok(decoded_args) = base64::decode(args) {
                                        //turn args into json and populate execution details
                                        if let Ok(args_json) =
                                            serde_json::from_slice::<serde_json::Value>(&decoded_args)
                                        {
                                            args_ = args_json;
                                        }
                                    }
                                    method_name_ = method_name.to_string();
                                    deposit_ = *deposit;

                                    //create the execution details to push into the vector
                                    let execution_details = ExecutionDetails {
                                        method_name: method_name_,
                                        args: args_.clone(),
                                        signer_id: signer_id_.clone(),
                                        deposit: deposit_,
                                        success_value: success_value_,
                                        transaction_id: transaction_id_.clone(), // it's not tx id, it's Receipt id
                                        predecessor_id: predecessor_id_.clone(), 
                                        receiver_id: receiver_id_.clone(),
                                    };

                                    execution_details_vector.push(execution_details); 
                                }
                                _ => {}
                            }
                        }
                    }

                    //loop through each execution detail 
                    for execution_details in execution_details_vector.iter() {
                        eprintln!("Looping through execution details vector. It's of length {}", execution_details_vector.len()); 
                        
                        //different cases based on the method that was called
                        match execution_details.method_name.as_str() {
                            //mint function was called
                            "nft_mint" => {
                                eprintln!("Beginning NFT Mint");
                                //get person who called nft_mint, the token, and contract.
                                let signer_id = execution_details.signer_id.clone();
                                let unclean_token_id = execution_details.args.get("token_id").unwrap();
                                let token_id = str::replace(&unclean_token_id.to_string(), '"', ""); //cleaning up RUST strings
                                let contract_id = execution_details.receiver_id.clone();

                                //get info from metadata --> add as many fields as are relevant to you
                                let metadata = execution_details.args.get("metadata").unwrap();

                                let media_option = metadata.get("media");
                                let artist_account_id_option = metadata.get("artist_account_id");
                                let charity_account_id_option = metadata.get("charity_account_id");
                                let title_option = metadata.get("title");
                                let description_option = metadata.get("description");
                                let copies_option = metadata.get("copies");
                                
                                // HANDLING MINTING LOGIC HERE --> FOR ACTUAL EXAMPLES OF MAKING API CALLS FROM THE INDEXER, REFER TO THE HANDLING OF SOME OF THE OTHER METHODS
                                eprintln!("Handling nft_mint method here.");
                            }
                            //nft_mint_payout was called
                            "nft_mint_payout" => {
                                eprintln!("Beginning NFT Mint Payout");
                                let unclean_base_token_id = execution_details.args.get("base_token_id").unwrap();
                                let base_token_id = str::replace(&unclean_base_token_id.to_string(), '"', "");

                                let unclean_edition_number = execution_details.args.get("edition_number").unwrap();
                                let edition_number = str::replace(&unclean_edition_number.to_string(), '"', "");

                                let unclean_price = execution_details.args.get("balance").unwrap();
                                let price = str::replace(&unclean_price.to_string(), '"', "");

                                let price_for_api_string =
                                    format!("{:.2}", human(price.parse().unwrap()));
                                let price_for_api: f64 = price_for_api_string.parse().unwrap();

                                //get the transaction ID for the api
                                let transaction_id = execution_details.transaction_id.clone();
                                    
                                //get the purchaser which in this case is the receiver
                                let unclean_receiver_id = execution_details.args.get("receiver_id").unwrap();
                                let receiver_id = str::replace(&unclean_receiver_id.to_string(), '"', "");

                                let contract_id = execution_details.receiver_id.clone();
                                let token_split_option = base_token_id.rsplit_once("_0");

                                if !token_split_option.is_none() {
                                    //get the token split
                                    let token_split = token_split_option.unwrap(); 

                                    //get the base token
                                    let base_token_no_edition = token_split.0;
                                    let token_id = format!("{}_{}", base_token_no_edition, edition_number);
                                    
                                    //we sell the token in the database because it was lazy purchased (minting without approvals process means it is not a base token)
                                    let _sell_token_result = database::sell_token_in_database(
                                        token_id.to_string(), 
                                        contract_id.to_string(), 
                                        Some(price_for_api), 
                                        receiver_id, 
                                        admin_account.clone(), 
                                        transaction_id.to_string(), 
                                        signature_header.clone(),
                                            &private_api_root,
                                        debug_mode.clone()).await;
                                } else {
                                    eprintln!("Cannot proceed with nft_mint_payout logic. the token ID was not a base token: {:?}", base_token_id.to_string());
                                }
                            }
                            //nft_on_approve was called
                            "nft_on_approve" => {
                                eprintln!("Beginning NFT On Approve");
                                let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                let contract_id_for_api = execution_details.predecessor_id.to_string();

                                //build the function arguments to get passed into nft_tokens_batch
                                let function_args = serde_json::json!({
                                    "token_ids": [token_id_for_api],
                                });

                                //call nft_tokens_batch in order to get access to the current token owner
                                let block_reference = BlockReference::latest();
                                let request = QueryRequest::CallFunction {
                                    //contract to call
                                    account_id: contract_id_for_api.parse().unwrap(),
                                    //method to view
                                    method_name: "nft_tokens_batch".to_string(),
                                    //passed in arguments
                                    args: FunctionArgs::from(function_args.to_string().into_bytes()),
                                };
                                let query = Query::new(block_reference, request);
                                //get the response
                                let response = view_client.send(query).await.unwrap().unwrap();

                                if let near_indexer::near_primitives::views::QueryResponseKind::CallResult(call_result) = response.kind {
                                    let result = call_result.result;

                                    //set the output (of type vec<JsonToken>) equal to the result
                                    let output: Vec<JsonToken> = serde_json::from_slice(&result).unwrap();
                                    if let Some(media) = output[0].clone().metadata.media {
                                        let SaleArgs { sale_conditions } = near_sdk::serde_json::from_str(execution_details.args.get("msg").unwrap().as_str().unwrap()).unwrap();

                                        for (ft_token_id, price) in sale_conditions.clone() {
                                            if &ft_token_id.to_string() == "near" {
                                                let price_for_api_string = format!("{:.2}", human(price.0));
                                                let price_for_api: f64 = price_for_api_string.parse().unwrap();

                                                eprintln!("Beginning API call to put up for sale.");
                                                let _put_for_sale_status = database::insert_token_forsale_in_database(
                                                    token_id_for_api.to_string(),
                                                    contract_id_for_api.to_string(),
                                                    price_for_api,
                                                    signature_header.clone(),
                                                                &private_api_root,
                                                    debug_mode.clone()
                                                ).await;

                                        }
                                    } else {
                                        eprintln!("Metadata has no media field... --> {:?}", output[0].metadata);
                                    }
                                }
                            }
                            //if update_price was called
                            "update_price" => {
                                eprintln!("Update Price Has Been Called");
                                let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                let contract_id_for_api =
                                    execution_details.args.get("nft_contract_id").unwrap();

                                let price = execution_details.args.get("price").unwrap();
                                let clean_price = str::replace(&price.to_string(), '"', "");
                                let price_for_api_string =
                                    format!("{:.2}", human(clean_price.parse().unwrap()));
                                let price_for_api: f64 = price_for_api_string.parse().unwrap();

                                let _result = database::update_price_for_token_in_database(
                                    token_id_for_api.to_string(),
                                    contract_id_for_api.to_string(),
                                    price_for_api,
                                    signature_header.clone(),
                                    &private_api_root,
                                    debug_mode.clone()
                                )
                                .await;
                            }
                            //if offer was called
                            "offer" => {
                                eprintln!("Offer has been called");
                                let lazy_purchase_option = execution_details.args.get("lazy_purchase");

                                //only do stuff if lazy_purchase_option wasn't passed in (meaning it's a regular purchase)
                                if lazy_purchase_option.is_none() {
                                    //build the function arguments to get passed into nft_tokens_batch
                                    let function_args = serde_json::json!({
                                        "token_ids": [execution_details.args.get("token_id").unwrap()],
                                    });

                                    //call nft_tokens_batch in order to get access to the current token owner
                                    let block_reference = BlockReference::latest();
                                    let request = QueryRequest::CallFunction {
                                        //contract to call
                                        account_id:
                                            near_indexer::near_primitives::types::AccountId::from_str(
                                                &nft_contract,
                                            )
                                            .expect(
                                                "failed to convert nft contract to account ID in view call",
                                            ),
                                        //method to view
                                        method_name: "nft_tokens_batch".to_string(),
                                        //passed in arguments
                                        args: FunctionArgs::from(function_args.to_string().into_bytes()),
                                    };
                                    let query = Query::new(block_reference, request);
                                    //get the response
                                    let response = view_client.send(query).await.unwrap().unwrap();

                                    if let near_indexer::near_primitives::views::QueryResponseKind::CallResult(call_result) = response.kind {
                                        let result = call_result.result;

                                        //set the output (of type vec<JsonToken>) equal to the result
                                        let output: Vec<JsonToken> = serde_json::from_slice(&result).unwrap();

                                        //check if the owner is the signer
                                        if output[0].owner_id == execution_details.signer_id {
                                            let price: f64 = human(execution_details.deposit);
                                            let purchaser_account_id = execution_details.signer_id.clone();
                                            let transaction_id = execution_details.transaction_id.clone();

                                            let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                            let contract_id_for_api = execution_details.args.get("nft_contract_id").unwrap();

                                            let _sell_token_result = database::sell_token_in_database(
                                                token_id_for_api.to_string(), 
                                                contract_id_for_api.to_string(), 
                                                Some(price), purchaser_account_id, 
                                                admin_account.clone(), 
                                                transaction_id.to_string(), 
                                                signature_header.clone(),
                                                &private_api_root,
                                                debug_mode.clone()).await;

                                        } else {
                                            eprintln!("Signer Is Not Owner... Transaction Failed.");
                                        }
                                    }
                                }
                            }
                            //if remove_sale was called
                            "remove_sale" => {
                                eprintln!("Beginning API Call to remove sale");

                                let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                let contract_id_for_api =
                                    execution_details.args.get("nft_contract_id").unwrap();

                                let _result = database::remove_token_forsale_in_database(
                                    token_id_for_api.to_string(),
                                    contract_id_for_api.to_string(),
                                    signature_header.clone(),
                                    &private_api_root,
                                    debug_mode.clone()
                                )
                                .await;
                            }
                            //if place was called
                            "place_bid" => {
                                eprintln!("Place Bid Was Called ---> {:#?}", execution_details.clone());

                                // handle place_bid method here
                            }
                            //accept offer was called
                            "accept_offer" => {
                                eprintln!(
                                    "Accept Offer Was Called ---> {:#?}",
                                    execution_details.clone()
                                );
                            }
                            //accept offer was called
                            "nft_revoke" => {
                                eprintln!("nft_revoke was called");

                                let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                let account_being_revoked =
                                    execution_details.args.get("account_id").unwrap();

                                let contract_id_for_api = execution_details.predecessor_id.clone();

                                if account_being_revoked.to_string() == market_contract {
                                    eprintln!("nft_revoke was called on OUR market account...");
                                    // TODO: the result is not used, do next call rely on this one?
                                    let _result = database::remove_token_forsale_in_database(
                                        token_id_for_api.to_string(),
                                        contract_id_for_api.to_string(),
                                        signature_header.clone(),
                                        &private_api_root,
                                        debug_mode.clone(),
                                    )
                                    .await;
                                }
                            }
                            //accept offer was called
                            "nft_revoke_all" => {
                                eprintln!(
                                    "Beginning API Call to remove sale since nft_revoke_all was called"
                                );
                                let token_id_for_api = execution_details.args.get("token_id").unwrap();
                                let contract_id_for_api = execution_details.predecessor_id.clone();
                                // TODO: the result is not used, do next call rely on this one?
                                let _result = database::remove_token_forsale_in_database(
                                    token_id_for_api.to_string(),
                                    contract_id_for_api.to_string(),
                                    signature_header.clone(),
                                    &private_api_root,
                                    debug_mode.clone(),
                                )
                                .await;
                            }
                            //some other transaction was called
                            _ => {
                                //print the entire execution outcome for the current receipt
                                eprintln!(
                                    "Other TXN Called ---> {:?} By {:?}",
                                    execution_details.method_name.as_str(),
                                    execution_details.signer_id.as_str()
                                );
                            }
                    }
                    
                    }
                } else {
                    //print execution details (FAILED)
                    eprintln!(
                        "Unsuccessful Execution Details related to Fayyr --> {:#?}",
                        execution_outcome
                    );
                }
            }
        }
    }
}

async fn listen_blocks(
    stream: mpsc::Receiver<near_indexer::StreamerMessage>,
    view_client: Addr<ViewClientActor>,
    nft_contract: String,
    market_contract: String,
    admin_account_id: String,
    public_api_root: String,
    private_api_root: String, 
    signature_header: String,
    debug_mode: String,
) {
    //listen for streams
    let mut handle_messages = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(|streamer_message| {
            eprintln!("Block Height {}", &streamer_message.block.header.height);
            handle_messages(
                streamer_message,
                view_client.clone(),
                nft_contract.clone(),
                market_contract.clone(),
                admin_account_id.clone(),
                public_api_root.clone(),
                private_api_root.clone(), 
                signature_header.clone(),
                debug_mode.clone(),
            )
        })
        .buffer_unordered(100); //1 is concurrency as per Bohdan's suggestion

    while let Some(_handled_message) = handle_messages.next().await {}
}

// Checks if the receipt is for our target nft and market contracts, and that
//    the receipt receiver is equal to the account id we care about
fn is_valid_receipt(
    receipt: &near_indexer::near_primitives::views::ReceiptView,
    nft_contract: String,
    market_contract: String,
) -> bool {
    receipt.receiver_id.as_ref() == nft_contract || receipt.receiver_id.as_ref() == market_contract
}

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();
    init_logging();

    let opts: Opts = Opts::parse();

    let home_dir = opts
        .home_dir
        .unwrap_or(std::path::PathBuf::from(near_indexer::get_default_home()));

    match opts.subcmd {
        //if we run cargo run -- run
        SubCommand::Run => {
            let nft_contract: String =
                env::var("NFT").expect("NFT Contract Env Variable Not Specified");
            let market_contract: String =
                env::var("MARKET").expect("Market Contract Env Variable Not Specified");
            let admin_account: String =
                env::var("ADMIN").expect("Admin Account Env Variable Not Specified");
            let private_api_root: String =
                env::var("PRIVATE_API").expect("Fayyr Private API Root Env Variable Not Specified");
            let public_api_root: String =
                env::var("PUBLIC_API").expect("Fayyr Public API Root Env Variable Not Specified");    
            let debug_mode: String =
                env::var("DEBUG").expect("Debugging Mode Env Variable Not Specified");
            let signature_header: String =
                env::var("HEADER").expect("Signature Header Env Variable Not Specified");


            eprintln!("Starting Indexer With NFT: {:?}, Market: {:?}, and Fayyr Account: {:?} and Debugging With: {:?} with Signature Header: {:?}", nft_contract, market_contract, admin_account, debug_mode, signature_header);

            //get the indexer config from the home directory
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                //Starts syncing from the block by which the indexer was interupted the las time it was run
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
                //wait until the entire syncing process is finished before streaming starts.
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::StreamWhileSyncing,
            };

            let sys = actix::System::new();
            sys.block_on(async move {
                let indexer = near_indexer::Indexer::new(indexer_config);
                //use view client to make view calls to the blockchain
                let view_client = indexer.client_actors().0; //returns tuple, second is another client actor - we only care about first value
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(
                    stream,
                    view_client,
                    nft_contract,
                    market_contract,
                    admin_account,
                    public_api_root,
                    private_api_root, 
                    signature_header,
                    debug_mode,
                ));
            });
            sys.run().unwrap();
        }
        //if we run cargo run -- init
        //initialize configs in the home directory (~./near)
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}
