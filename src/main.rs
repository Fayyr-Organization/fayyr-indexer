use actix;

use near_indexer::near_primitives::views::ExecutionStatusView;
use near_indexer::near_primitives::types::BlockReference;
use near_indexer::near_primitives::types::FunctionArgs;
use near_indexer::near_primitives::views::QueryRequest;
use near_client::Query;
use near_client::ViewClientActor;

use clap::Clap;
use tokio::sync::mpsc;

use actix::Addr;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

use serde::{Serialize, Deserialize};

mod configs;

//use this struct to store information that we want to pass to database
#[derive(Debug)] //derive debug so that we can print
struct ExecutionDetails {
    method_name: String,
    args: serde_json::Value,
    signer_id: String,
    deposit: String,
    success_value: bool,
}

//declare struct for the return type of the blockchain view call
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonToken {
    pub owner_id: String, //only declaring the field we care about
}

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>, view_client: Addr<ViewClientActor>) {
    eprintln!("listen_blocks");
    //listen for streams
    while let Some(streamer_message) = stream.recv().await {
        //iterate through each shard in the incoming stream
        for shard in streamer_message.shards {
            //for each receipt and execution outcome pair in the shard
            for receipt_and_execution_outcome in shard.receipt_execution_outcomes {
                // Check if receipt is related to Fayyr
                if is_fayyr_receipt(&receipt_and_execution_outcome.receipt) {
                    //get the execution outcome from the receipt and execution outcome pair from the shard
                    let execution_outcome = receipt_and_execution_outcome.execution_outcome;

                    //declare the execution details that will hold specific wanted information
                    let mut execution_details = ExecutionDetails {
                        method_name: "".to_string(),
                        args: serde_json::Value::String("".to_string()),
                        signer_id: "".to_string(),
                        deposit: "".to_string(),
                        success_value: matches!(execution_outcome.outcome.status, ExecutionStatusView::SuccessValue(_) | ExecutionStatusView::SuccessReceiptId(_)),
                    };
                    
                    //get the signer id from the receipt
                    let signer_id = if let near_indexer::near_primitives::views::ReceiptEnumView::Action { ref signer_id, .. } = receipt_and_execution_outcome.receipt.receipt {
                        Some(signer_id)
                    } else {
                        None
                    };

                    //if there is some signer id, set the execution detail's signer equal to it
                    match signer_id {
                        Some(signer_id) => {
                            execution_details.signer_id = signer_id.to_string();
                        },
                        _ => {},
                    };

                    //print the entire execution outcome for the current receipt
                    eprintln!("Entire Execution Outcome ---> {:#?}", execution_outcome);   

                    //get the actions from the receipt
                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        actions,
                        ..
                    } = receipt_and_execution_outcome.receipt.receipt
                    {   
                        //go through each action
                        for action in actions.iter() {
                            //get the args from the action
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                args,
                                ..
                            } = action
                            {   
                                //decode the args
                                if let Ok(decoded_args) = base64::decode(args) {
                                    //turn args into json and populate execution details
                                    if let Ok(args_json) = serde_json::from_slice::<serde_json::Value>(&decoded_args) {
                                        execution_details.args = args_json;
                                    }
                                }
                            }
                            //get the method name 
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                method_name,
                                ..
                            } = action
                            {
                                execution_details.method_name = method_name.to_string();
                            }
                            //get the deposit
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                deposit,
                                ..
                            } = action
                            {
                                execution_details.deposit = deposit.to_string();
                            }
                        }
                    }

                    //only do stuff with the receipts if the outcome was successful
                    if execution_details.success_value == true {
                        //different cases based on the method that was called
                        match execution_details.method_name.as_str() {
                            //if remove_sale was called
                            "remove_sale" => {
                                eprintln!("Remove Sale Has Been Called")
                            },
                            //if update_price was called
                            "update_price" => {
                                eprintln!("Update Price Has Been Called")
                            },
                            //if offer was called
                            "offer" => {
                                eprintln!("Offer Has Been Called");

                                //build the function arguments to get passed into nft_tokens_batch
                                let function_args = serde_json::json!({
                                    "token_ids": [execution_details.args.get("token_id").unwrap()],
                                });
                                
                                //call nft_tokens_batch in order to get access to the current token owner
                                let block_reference = BlockReference::latest();
                                let request = QueryRequest::CallFunction {
                                    //contract to call
                                    account_id: "test.near".parse().unwrap(),
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
                                    eprintln!("Current Token Owner {:?}", output[0].owner_id);

                                    //check if the owner is the signer
                                    if output[0].owner_id == execution_details.signer_id {
                                        eprintln!("Signer Is Owner! Transaction Successful!");
                                    } else {
                                        eprintln!("Signer Is Not Owner... Transaction Failed.");
                                    }
                                }
                            },
                            //nft_on_approve was called
                            "nft_on_approve" => {
                                eprintln!("Token Has Been Put Up For Sale")
                            },
                            //some other transaction was called
                            _ => {
                                eprintln!("Other Transaction Called...")
                            }
                        }
                        //print execution details (SUCCESSFUL)
                        eprintln!("Successful Execution Details {:?} related to Fayyr", execution_details);
                    } else {
                        //print execution details (FAILED)
                        eprintln!("Unsuccessful Execution Details {:?} related to Fayyr", execution_details);
                    }
                }
            }
        }
    }
}

// Assuming contract deployed to account id market.test.near
// Checks if receipt receiver is equal to the account id we care about
fn is_fayyr_receipt(receipt: &near_indexer::near_primitives::views::ReceiptView) -> bool {
    receipt.receiver_id.as_str() == "market2.test.near"
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
            eprintln!("Starting...");

            //get the indexer config from the home directory
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                //Starts syncing from the block by which the indexer was interupted the las time it was run
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
                //wait until the entire syncing process is finished before streaming starts. 
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync, 
            };

            let sys = actix::System::new();
            sys.block_on(async move {
                eprintln!("Actix");
                let indexer = near_indexer::Indexer::new(indexer_config);
                //use view client to make view calls to the blockchain
                let view_client = indexer.client_actors().0; //returns tuple, second is another client actor - we only care about first value
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(stream, view_client));
            });
            sys.run().unwrap();
        }
        //if we run cargo run -- init
        //initialize configs in the home directory (~./near)
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}
