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
//use tracing::info;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

use serde::{Serialize, Deserialize};

//use postgres::{Client, NoTls};

mod configs;

#[derive(Debug)]
struct ExecutionDetails {
    method_name: String,
    args: serde_json::Value,
    signer_id: String,
    deposit: String,
    success_value: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonToken {
    pub owner_id: String,
}

// Assuming fayyr contract deployed to account id fayyr_market_contract_5.testnet
// We want to catch all *successfull* transactions sent to this contract
// In the demo we'll just look for them and log them
// For real world application we need some real handlers

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>, view_client: Addr<ViewClientActor>) {
    eprintln!("listen_blocks");
    while let Some(streamer_message) = stream.recv().await {
        for shard in streamer_message.shards {
            for receipt_and_execution_outcome in shard.receipt_execution_outcomes {
                // Check if transaction is fayyr related
                if is_fayyr_receipt(&receipt_and_execution_outcome.receipt) {
                    let execution_outcome = receipt_and_execution_outcome.execution_outcome;

                    let mut execution_details = ExecutionDetails {
                        method_name: "".to_string(),
                        args: serde_json::Value::String("".to_string()),
                        signer_id: "".to_string(),
                        deposit: "".to_string(),
                        success_value: matches!(execution_outcome.outcome.status, ExecutionStatusView::SuccessValue(_) | ExecutionStatusView::SuccessReceiptId(_)),
                    };

                    let signer_id = if let near_indexer::near_primitives::views::ReceiptEnumView::Action { ref signer_id, .. } = receipt_and_execution_outcome.receipt.receipt {
                        Some(signer_id)
                    } else {
                        None
                    };

                    match signer_id {
                        Some(signer_id) => {
                            execution_details.signer_id = signer_id.to_string();
                        },
                        _ => {},
                    };

                    eprintln!("Entire Execution Outcome ---> {:#?}", execution_outcome);   

                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        actions,
                        ..
                    } = receipt_and_execution_outcome.receipt.receipt
                    {
                        for action in actions.iter() {
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                args,
                                ..
                            } = action
                            {
                                if let Ok(decoded_args) = base64::decode(args) {
                                    if let Ok(args_json) = serde_json::from_slice::<serde_json::Value>(&decoded_args) {
                                        execution_details.args = args_json;
                                    }
                                }
                            }
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                method_name,
                                ..
                            } = action
                            {
                                execution_details.method_name = method_name.to_string();
                            }
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                deposit,
                                ..
                            } = action
                            {
                                execution_details.deposit = deposit.to_string();
                            }
                        }
                    }

                    //eprintln!("Current Token Owner {:?}", current_token_owner);
                    match execution_details.method_name.as_str() {
                        "remove_sale" => {
                            eprintln!("Remove Sale Has Been Called")
                        },
                        "update_price" => {
                            eprintln!("Update Price Has Been Called")
                        },
                        "offer" => {
                            eprintln!("Offer Has Been Called");
                            let block_reference = BlockReference::latest();
                            let request = QueryRequest::CallFunction {
                                account_id: "test.near".parse().unwrap(),
                                method_name: "nft_tokens_batch".to_string(),
                                args: FunctionArgs::from(r#"{"token_ids": ["3"]}"#.as_bytes().to_owned()),
                            };
                            let query = Query::new(block_reference, request);
                            let response = view_client.send(query).await.unwrap().unwrap();

                            if let near_indexer::near_primitives::views::QueryResponseKind::CallResult(call_result) = response.kind {
                                let result = call_result.result;
    
                            let output: Vec<JsonToken> = serde_json::from_slice(&result).unwrap();
                            eprintln!("Current Token Owner {:?}", output);
                            if output[0].owner_id == execution_details.signer_id {
                                eprintln!("Signer Is Owner! Transaction Successful!");
                            } else {
                                eprintln!("Signer Is Not Owner... Transaction Failed.");
                            }
                    }
                        },
                        "nft_on_approve" => {
                            eprintln!("Offer Has Been Called")
                        },
                        _ => {
                            eprintln!("Other Transaction Called...")
                        }
                    }
                    // log the tx because we've found it
                    eprintln!("Execution Details {:?} related to Fayyr", execution_details);
                }
            }
        }
    }
}

fn is_fayyr_receipt(receipt: &near_indexer::near_primitives::views::ReceiptView) -> bool {
    receipt.receiver_id.as_str() == "market.test.near"
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
        SubCommand::Run => {
            eprintln!("Starting...");

            // eprintln!("Connecting to Database...");
            // let mut connecting = Client::connect("host=fayyr-indexer-testing.postgres.database.azure.com 
            // port=5432 dbname=testingDB1 user=IndexerAdmin password=8724fa993oooooo?@ sslmode=require", NoTls);
            // match connecting {
            //     Ok(cli) => {
            //         println!("Database connected.");
            //         //db_client = Some(cli);
            //     }
            //     Err(error) => panic!("Database ERROR while connecting: {:?}",error),
            // }

            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync, //streamwhilesyncing
            };

            let sys = actix::System::new();
            sys.block_on(async move {
                eprintln!("Actix");
                let indexer = near_indexer::Indexer::new(indexer_config);
                let view_client = indexer.client_actors().0; //returns tuple, second is another client actor - we only care about first value
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(stream, view_client));
            });
            sys.run().unwrap();
        }
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}
