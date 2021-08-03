# Fayyr Indexer 

## 🚨🚨🚨 WORK IN PROGRESS 🚨🚨🚨

An indexer that catches specific method calls for specific contracts and relays information to a PostgreSQL database. Created by: 
- @BenKurrek - https://github.com/BenKurrek
- @mariavmihu - https://github.com/mariavmihu
- @frol - https://github.com/frol
- @khorolets - https://github.com/khorolets

## TODO / DONE:
- [x] basic indexer working
- [x] match statements to determine which method has been called
- [x] struct for passing information to database
- [x] cross-contract calls
- [ ] connect to database to do posts
- [ ] do we want to keep track of unsuccessful transactions in a DB? 

## Running Indexer:
- install dependencies and compile code using `cargo check`
- initialize config using `cargo run -- init`
- run indexer using `cargo run -- run`

## Commands For Local Testing:
MAKE SURE TO DO THIS FIRST: set NEAR_ENV variable locally using: `export NEAR_ENV=localnet`

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