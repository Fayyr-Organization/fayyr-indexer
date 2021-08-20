# Fayyr Indexer 

## 🚨🚨🚨 WORK IN PROGRESS 🚨🚨🚨

An indexer that catches specific methods for any desired deployed contracts and relays information to a CRUD web API. 
Created by: 
- @BenKurrek - https://github.com/BenKurrek
- @mariavmihu - https://github.com/mariavmihu
- @frol - https://github.com/frol
- @khorolets - https://github.com/khorolets

## TODO / DONE:
- [x] basic indexer working
- [x] match statements to determine which method has been called
- [x] struct for passing information to database
- [x] picking up cross-contract calls
- [x] connect to CRUD Web API to do posts
- [x] finish writing up documentation fo how to use

## Background Information:
- this indexer currently catches specific methods on a modified version of Matt Lockyer's NFT Marketplace contract which can be found here: https://github.com/BenKurrek/nft-market
- To change the desired contract that the indexer will look for, simply go to the function `is_fayyr_receipt` and change the accountId accordingly. 
- In order to make a POST to an API, you will need to uncomment the lines of code in the `remove_token_forsale_database`, `insert_token_forsale_database`, and `update_token_forsale_database` functions and add a valid URL. 

## Running Indexer:
- install dependencies and compile code using `cargo check`
- initialize config using `cargo run -- init`
- run indexer using `cargo run -- run` if this fails, navigate to your `./near` directory (which is usually in your home directory) and open the `config.json` file. 
- Scroll to the bottom of the file and replace `"tracked_shards": [],` with `"tracked_shards": [0],`

## Complete Guide To Local Testing
- Make sure the indexer is running by calling `cargo run -- run` as mentioned above. 
- In a new terminal, navigate to the nft-market directory
- The first thing you want to do is to set the NEAR_ENV variable by calling: `export NEAR_ENV=localnet`
- Build all the NFT and Market contracts using the following command: `cd contracts/nft-simple && ./build.sh && cd ../.. && cd contracts/market-simple && ./build.sh && cd ../..` This will create the wasm files that will be deployed to the local network. 
- deploy the nft contract using `near deploy --wasmFile out/main.wasm --accountId test.near`
- instantiate a new instance of the nft contract using: `near call --accountId test.near test.near new '{"owner_id": "test.near", "metadata": {"spec": "1.0.0", "name": "TESTING CONTRACT", "symbol": "N/A"}}'`
- create a subaccount that can be used to deploy the market contract using: `near create-account market.test.near --masterAccount test.near --initialBalance=40 --keyPath ~/.near/validator_key.json`
- deploy the market contract using `near deploy --wasmFile out/market.wasm --accountId market.test.near`
- instantiate a new instance of the market contract: `near call --accountId market.test.near market.test.near new '{"owner_id": "market.test.near"}'` The indexer should pickup this transaction and print the execution outcome and display that a transaction was called. 
- You now have a market contract and nft contract deployed on your local network which you can play around with and see if the indexer picks up methods called on the market contract.

## Useful Contract calls For Testing
View For Sale Listings
- `near view market.test.near get_sales_by_nft_contract_id '{"nft_contract_id": "test.near", "from_index": "0", "limit": 50}'`

View Tokens On NFT Contract
- `near view test.near nft_tokens '{"from_index": "0", "limit": 50}'`

Update Price (make sure the token exists)
- `near call --accountId ben.test.near market.test.near update_price '{"nft_contract_id": "test.near", "token_id": "3", "ft_token_id": "near", "price": "5"}' --amount 0.000000000000000000000001`

Remove Sale (make sure the token exists)
- `near call --accountId ben.test.near market.test.near remove_sale '{"nft_contract_id": "test.near", "token_id": "3"}' --amount 0.000000000000000000000001`

Place Item For Sale
- `near call --accountId ben.test.near test.near nft_approve '{"token_id": "3", "account_id": "market.test.near", "msg": "{\"sale_conditions\":[{\"ft_token_id\":\"near\",\"price\":\"5000000000000000000000000\"}]}"}' --amount 1` 

Offer
- `near call --accountId bob.test.near market.test.near offer '{"nft_contract_id": "test.near", "token_id": "3"}' --amount 5 --gas=200000000000000`

Deposit Storage
- `near call --accountId bob.test.near market.test.near storage_deposit '{}' --amount 0.1`

## Commands For NEAR CLI Account Stuff:
Create New SubAccount
- `near create-account bob.test.near --masterAccount test.near --initialBalance=40 --keyPath ~/.near/validator_key.json`
