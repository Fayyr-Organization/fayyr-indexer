# Fayyr Indexer 

## 🚨🚨🚨 WORK IN PROGRESS 🚨🚨🚨

An indexer that catches specific method calls for specific contracts and relays information to a PostgreSQL database. 

## TODO / DONE:
- [x] basic indexer working
- [x] match statements to determine which method has been called
- [x] struct for passing information to database
- [ ] connect to database to do posts
- [ ] cross-contract calls
- [ ] do we want to keep track of unsuccessful transactions in a DB? 

## Running Indexer:
- install dependencies and compile code using `cargo check`
- initialize config using `cargo run -- init`
- run indexer using `cargo run -- run`

## Commands For Local Testing:
MAKE SURE TO DO THIS FIRST: set NEAR_ENV variable locally using: `export NEAR_ENV=localnet`

Update Price (make sure the token exists)
- `near call --accountId ben.test.near market.test.near update_price '{"nft_contract_id": "test.near", "token_id": "3", "ft_token_id": "near", "price": "5"}' --amount 0.000000000000000000000001`

Remove Sale (make sure the token exists)
- `near call --accountId ben.test.near market.test.near remove_sale '{"nft_contract_id": "test.near", "token_id": "3"}' --amount 0.000000000000000000000001`

Place Item For Sale
- `near call --accountId ben.test.near test.near nft_approve '{"token_id": "3", "account_id": "market.test.near", "msg": "{\"sale_conditions\":[{\"ft_token_id\":\"near\",\"price\":\"5000000000000000000000000\"}]}"}' --amount 0.000000000000000000000001`