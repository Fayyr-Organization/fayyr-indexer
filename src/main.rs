use actix;

use std::env;

use clap::Clap;
use tokio::sync::mpsc;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

mod configs;

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>, nft_contract: String) {
    //listen for streams
    while let Some(streamer_message) = stream.recv().await {
        //iterate through each shard in the incoming stream
        for shard in streamer_message.shards {
            //for each receipt and execution outcome pair in the shard
            for receipt_and_execution_outcome in shard.receipt_execution_outcomes {
                // Check if receipt is related to Fayyr
                if is_fayyr_receipt(&receipt_and_execution_outcome.receipt, nft_contract.clone()) {
                    let mut found_method_name = "".to_string(); 
                    //get the actions from the receipt
                    if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                        actions,
                        ..
                    } = receipt_and_execution_outcome.receipt.receipt
                    {   
                        //go through each action
                        for action in actions.iter() {
                            //get the method name 
                            if let near_indexer::near_primitives::views::ActionView::FunctionCall {
                                method_name,
                                ..
                            } = action
                            {
                                found_method_name = method_name.to_string();
                            }
                        }
                    }

                    if found_method_name == "nft_mint".to_string() {
                        eprintln!("nft_mint was called!");
                    }
                }
            }
        }
    }
}

// Assuming contract deployed to account id market.test.near
// Checks if receipt receiver is equal to the account id we care about
fn is_fayyr_receipt(receipt: &near_indexer::near_primitives::views::ReceiptView, nft_contract: String) -> bool {
    receipt.receiver_id.as_ref() == nft_contract
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

            let nft_contract: String = env::var("contract").unwrap();

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
                let indexer = near_indexer::Indexer::new(indexer_config);
                //use view client to make view calls to the blockchain
                let stream = indexer.streamer();
                actix::spawn(listen_blocks(stream, nft_contract));
            });
            sys.run().unwrap();
        }
        //if we run cargo run -- init
        //initialize configs in the home directory (~./near)
        SubCommand::Init(config) => near_indexer::indexer_init_configs(&home_dir, config.into()),
    }
}
