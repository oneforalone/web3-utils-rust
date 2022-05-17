use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use serde::de::value::Error;
use serde_json::Value;

use web3::contract::{Contract, Options};
use web3::transports::WebSocket;
use web3::types::{Address, U256};

fn load_abi_json(json_file: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(json_file).unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();

    Ok(json_str)
}

fn get_oracle_feeds(token: &str) -> Result<(String, String), Error> {
    let feeds_file = "chainlink-data-feeds.json";
    let data = load_abi_json(feeds_file).unwrap();

    let json: Value = serde_json::from_str(&data).unwrap();
    let addr: &str = json[token]["address"].as_str().unwrap();
    let abi: &str = json[token]["abi"].as_str().unwrap();

    Ok((addr.to_string(), abi.to_string()))
}

async fn get_price(token: &str) -> Result<f64, web3::Error> {
    let wss = "wss://mainnet.infura.io/ws/v3/48c4fb93a3794a1fb80da6c53226db1c";
    let websocket = WebSocket::new(wss).await.unwrap();
    let web3 = web3::Web3::new(websocket);

    let (addr, abi_file) = get_oracle_feeds(token).unwrap();

    let contract_addr = Address::from_str(addr.as_str()).unwrap();

    // get abi, cause include_bytes!() need take a string literal,
    // the macro works at compile time, can not take a variable as parameter
    let abi_str = load_abi_json(abi_file.as_str()).unwrap();
    let abi = abi_str.as_bytes();

    let contract = Contract::from_json(web3.eth(), contract_addr, abi).unwrap();

    let decimals: u32 = contract
        .query("decimals", (), None, Options::default(), None)
        .await
        .unwrap();

    let price_result: (U256, i64, U256, U256, U256) = contract
        .query("latestRoundData", (), None, Options::default(), None)
        .await
        .unwrap();

    Ok(price_result.1 as f64 / i64::pow(10, decimals) as f64)
}

#[tokio::main]
async fn main() {
    let price = get_price("btc").await.unwrap();
    println!("Current Price of BTC is: {}", price);
}
