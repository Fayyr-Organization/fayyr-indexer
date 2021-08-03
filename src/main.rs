use actix;
use std::collections::{HashMap, HashSet};

use clap::Clap;
use tokio::sync::mpsc;
//use tracing::info;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

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


// Assuming fayyr contract deployed to account id fayyr_market_contract_5.testnet
// We want to catch all *successfull* transactions sent to this contract
// In the demo we'll just look for them and log them
// For real world application we need some real handlers

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>) {
    eprintln!("listen_blocks");
    // map of correspondence between txs and receipts
    let mut tx_receipt_ids = HashMap::<String, String>::new();
    // list of receipt ids we're following
    let mut wanted_receipt_ids = HashSet::<String>::new();
    while let Some(streamer_message) = stream.recv().await {
        for shard in streamer_message.shards {
            let chunk = if let Some(chunk) = shard.chunk {
                chunk
            } else {
                continue;
            };

            for transaction in chunk.transactions {
                // Check if transaction is fayyr related
                if is_fayyr_tx(&transaction) {
                    // extract receipt_id transaction was converted into
                    let converted_into_receipt_id = transaction
                        .outcome
                        .execution_outcome
                        .outcome
                        .receipt_ids
                        .first()
                        .expect("`receipt_ids` must contain one Receipt Id")
                        .to_string();
                    // add `converted_into_receipt_id` to the list of receipt ids we are interested in
                    wanted_receipt_ids.insert(converted_into_receipt_id.clone());
                    // add key value pair of transaction hash and in which receipt id it was converted for further lookup
                    tx_receipt_ids.insert(
                        converted_into_receipt_id,
                        transaction.transaction.hash.to_string(),
                    );
                }
            }

            for execution_outcome in shard.receipt_execution_outcomes {
                if let Some(receipt_id) =
                    wanted_receipt_ids.take(&execution_outcome.receipt.receipt_id.to_string())
                {
                    let mut execution_details = ExecutionDetails {
                        method_name: "".to_string(),
                        args: serde_json::Value::String("".to_string()),
                        signer_id: "".to_string(),
                        deposit: "".to_string(),
                        success_value: true    //execution_outcome.execution_outcome.outcome.status, == SuccessValue(``),
                    };

                    let signer_id = if let near_indexer::near_primitives::views::ReceiptEnumView::Action { ref signer_id, .. } = execution_outcome.receipt.receipt {
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
                    } = execution_outcome.receipt.receipt
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
                    match execution_details.method_name.as_str() {
                        "remove_sale" => {
                            eprintln!("Remove Sale Has Been Called")
                        },
                        "update_price" => {
                            eprintln!("Update Price Has Been Called")
                        },
                        "offer" => {
                            eprintln!("Offer Has Been Called")
                        },
                        _ => {
                            eprintln!("Other Transaction Called...")
                        }
                    }
                    // log the tx because we've found it
                    eprintln!("Execution Details {:?} related to Fayyr", execution_details);

                    // remove tx from hashmap
                    tx_receipt_ids.remove(receipt_id.as_str());
                }
            }
        }
    }
}

fn is_fayyr_tx(tx: &near_indexer::IndexerTransactionWithOutcome) -> bool {
    tx.transaction.receiver_id.as_str() == "market.test.near" //|| tx.transaction.receiver_id.as_str() == "test.near"
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
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync,
            };

            let sys = actix::System::new();
            sys.block_on(async move {
                eprintln!("Actix");
                let indexer = near_indexer::Indexer::new(indexer_config);
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(stream));
            });
            sys.run().unwrap();
        }
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}
