#![allow(non_snake_case)]
use reqwest::Error;
use reqwest::StatusCode;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ------------------------------- API CALLS ----------------------------------
// this file was created to separate the handling of various API calls into their 
//  own functions, which can be called in main.rs if imported correctly (main.rs, line 31)
// we have left examples of GET and POST methods to use as reference.

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MintedTokenPOSTBody {
    pub token_id: String,
    pub contract_id: String,
    pub tags: Vec<String>,
    pub artwork_url: String,
    pub owner_account_id: String,
    pub artist_account_id: String,
    pub charity_account_id: String,
    pub aspect_ratio: f32,
    pub copies: u64,
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Debug)]
struct InsertForSalePOSTBody {
    token_id: String,
    contract_id: String,
    price_near: f64,
}

#[derive(Serialize, Debug)]
struct UpdatePricePOSTBody {
    token_id: String,
    contract_id: String,
    price_near: f64,
}


#[derive(Serialize, Debug)]
struct SellTokenPostBody {
    token_id: String,
    contract_id: String,
    purchaser_account_id: String,
    price_near: Option<f64>,
    admin_account_id: String,
    receipt_id: String,
}

pub async fn remove_token_forsale_in_database(
    tok_id: String,
    contr_id: String,
    SIGNATURE_HEADER: String,
    URL: &str,
    debug_mode: String,
) -> Result<(), Error> {
    // cleaning up rust strings
    let clean_token_id = str::replace(&tok_id, '"', "");
    let clean_contract_id = str::replace(&contr_id, '"', "");

    eprintln!("Please Add Your API Endpoint and Uncomment Lines 67-94");

//    let final_url = format!("YOUR API ENDPOINT HERE!", URL); 

    // This will POST a body defined by the hashmap
//    let mut map = HashMap::new();
//    map.insert("token_id", clean_token_id);
//    map.insert("contract_id", clean_contract_id);
    // make POST request
//    let client = reqwest::Client::new();
//    let res = client
//        .post(final_url)
//        .header("Signature", SIGNATURE_HEADER.clone())
//        .json(&map)
//        .send()
//        .await?;

//    match res.status() {
//        StatusCode::OK => println!("Success when removing token for sale in database"),

//        s => println!(
//            "Received response status when removing token for sale in db --> {:?}: {:?} | {:?}",
//            map, s, res
//        ),
//    };
//    if debug_mode == "TRUE".to_string() {
//        println!(
//            "Passing In This Body To Remove Token For Sale --> {:?} With this signature header: {:?}",
//            map, SIGNATURE_HEADER);
//    }

    Ok(())
}

pub async fn sell_token_in_database(
    tok_id: String,
    contr_id: String,
    price_near: Option<f64>,
    purchaser_account_id: String,
    admin_account: String,
    blockchain_receipt_id: String,
    SIGNATURE_HEADER: String,
    URL: &str,
    debug_mode: String,
) -> Result<(), Error> {
    // cleaning up rust strings
    let clean_token_id = str::replace(&tok_id, '"', "");
    let clean_contract_id = str::replace(&contr_id, '"', "");
    let clean_purchaser_account_id = str::replace(&purchaser_account_id, '"', "");
    let clean_blockchain_receipt_id = str::replace(&blockchain_receipt_id, '"', "");

    let PostBody = SellTokenPostBody {
        token_id: clean_token_id,
        contract_id: clean_contract_id,
        purchaser_account_id: clean_purchaser_account_id,
        admin_account_id: admin_account,
        receipt_id: clean_blockchain_receipt_id,
        price_near,
    };

    eprintln!("Please Add Your API Endpoint and Uncomment Lines 126-150");
//    let final_url = format!("YOUR URL ENDPOINT HERE !!", URL);

    // make POST request
//    let client = reqwest::Client::new();
//    let res = client
//        .post(final_url)
//        .header("Signature", SIGNATURE_HEADER.clone())
//        .json(&PostBody)
//        .send()
//        .await?;

//    match res.status() {
//        StatusCode::OK => println!("Success when selling token in database"),

//        s => println!(
//            "Received response status when selling token in database --> {:?}: {:?} | {:?}",
//            PostBody, s, res
//        ),
//    };
//    if debug_mode == "TRUE".to_string() {
//        println!(
//            "Passing In This Body To Sell NFT --> {:?} With this signature header: {:?}",
//            PostBody, SIGNATURE_HEADER);
//    }
    Ok(())
}

pub async fn insert_token_forsale_in_database(
    tok_id: String,
    contr_id: String,
    price: f64,
    SIGNATURE_HEADER: String,
    URL: &str,
    debug_mode: String,
) -> Result<(), Error> {
    //cleaning up rust strings
    let clean_token_id = str::replace(&tok_id, '"', "");
    let clean_contract_id = str::replace(&contr_id, '"', "");

    let PostBody = InsertForSalePOSTBody {
        token_id: clean_token_id,
        contract_id: clean_contract_id,
        price_near: price,
    };

    eprintln!("Please Add Your API Endpoint and Uncomment Lines 171-194");
//    let final_url = format!("YOUR URL ENPOINT HERE!!", URL);

    //make POST request
//    let client = reqwest::Client::new();
//    let res = client
//        .post(final_url)
//        .header("Signature", SIGNATURE_HEADER.clone())
//        .json(&PostBody)
//        .send()
//        .await?;

//    match res.status() {
//            StatusCode::OK => println!("Success when inserting token for sale in database!"),
                                  
//            s => println!("Received response status when inserting token for sale in database --> {:?}: {:?} | {:?}", PostBody, s, res),
//        };
    
//    if debug_mode == "TRUE".to_string() {
//        println!(
//            "Passing In This Body To Insert Token For Sale --> {:?} With this signature header: {:?}",
//            PostBody, SIGNATURE_HEADER);
//    }
    Ok(())
}

pub async fn update_price_for_token_in_database(
    tok_id: String,
    contr_id: String,
    price: f64,
    SIGNATURE_HEADER: String,
    URL: &str,
    debug_mode: String,
) -> Result<(), Error> {
    //cleaning up rust strings
    let clean_token_id = str::replace(&tok_id, '"', "");
    let clean_contract_id = str::replace(&contr_id, '"', "");

    let PostBody = UpdatePricePOSTBody {
        token_id: clean_token_id,
        contract_id: clean_contract_id,
        price_near: price,
    };

    eprintln!("Please Add Your API Endpoint and Uncomment Lines 216-239");
//    let final_url = format!("YOUR API URL ENDPOINT HERE!!", URL);

    //make POST request
//    let client = reqwest::Client::new();
//    let res = client
//        .post(final_url)
//        .header("Signature", SIGNATURE_HEADER.clone())
//        .json(&PostBody)
//        .send()
//        .await?;

//    match res.status() {
//        StatusCode::OK => println!("Success when updating price for token in database!"),

//        s => println!(
//            "Received response status when updating price for token in db --> {:?}: {:?} | {:?}",
//            PostBody, s, res
//        ),
//    };
//    if debug_mode == "TRUE".to_string() {
//        println!(
//            "Passing In This Body To Update Price --> {:?} With this signature header: {:?}",
//            PostBody, SIGNATURE_HEADER);
//    }
    Ok(())
}

// example of using reqwest::get to query information about a token and store the retrived information as a struct of your choosing
// NOTE: example of usage not shown in main.rs ... please contact us if you have trouble implementing this.
pub async fn get_minted_token_from_database(
    token_id: String,
    contract_id: String,
    SIGNATURE_HEADER: String,
    URL: &str,
    debug_mode: String,
) -> Result<MintedTokenPOSTBody, Error> {
    // cleaning up rust strings
    let clean_token_id = str::replace(&token_id, '"', "");
    let clean_contract = str::replace(&contract_id, '"', "");

    eprintln!("Please Add Your API Endpoint and Uncomment Lines 257-272");
//    let final_url: String = format!(
//        "YOUR API URL ENDPOINT HERE",
//        URL, clean_contract, clean_token_id
//    )
//    .to_string();

//    let res = reqwest::get(final_url.clone()).await?;
//    let json_body: Vec<MintedTokenPOSTBody> = res.json().await?;

//    if debug_mode == "TRUE".to_string() {
//        println!(
//            "Passing In This URL To Get Minted Token From Database --> {:?}",
//            final_url.clone());
//    }
//    Ok(json_body[0].clone())
}
