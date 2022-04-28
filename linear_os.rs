// This file was automatically generated
// VERSION = 1

use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm::{execute_entry_point, ext, oei, prepare_entry_point};
use std::collections::hash_map::*;
use std::collections::HashMap;
use std::str::FromStr;
use strum::{EnumProperty, IntoEnumIterator, ParseError};
use strum_macros::{EnumIter, EnumProperty as EnumPropertyTrait, EnumString, ToString};

#[derive(OBIDecode, OBISchema)]
struct Input {
    symbols: Vec<String>,
    multiplier: u64,
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    rates: Vec<u64>,
}

const EXCHANGE_COUNT: u64 = 13;

const API_SOURCE: [Exchange; 13] = [
    Exchange::BIBOX,
    Exchange::BINANCE,
    Exchange::BITFINEX,
    Exchange::BITTREX,
    Exchange::BRAVENEWCOIN,
    Exchange::COINBASEPRO,
    Exchange::COINGECKO,
    Exchange::COINMARKETCAP,
    Exchange::CRYPTOCOMPARE,
    Exchange::HITBTC,
    Exchange::HUOBIPRO,
    Exchange::KRAKEN,
    Exchange::OKX,
];

#[derive(ToString, EnumString, EnumIter, PartialEq, Debug, Copy, Clone)]
enum Token {
    AAVE,
    ADA,
    ALGO,
    ATOM,
    AUDIO,
    AVAX,
    AXS,
    BAL,
    BAT,
    BCH,
    BNB,
    BORA,
    BTC,
    BTT,
    CAKE,
    CELO,
    COMP,
    CRO,
    CRV,
    DGB,
    DOGE,
    DOT,
    DYDX,
    EGLD,
    ENJ,
    EOS,
    ETH,
    FIL,
    FTM,
    FTT,
    GALA,
    HT,
    ICX,
    ILV,
    IMX,
    KLAY,
    KNC,
    KSM,
    LEO,
    LINK,
    LRC,
    LTC,
    LUNA,
    MANA,
    MATIC,
    MIOTA,
    MKR,
    MLN,
    MTL,
    NEAR,
    NEO,
    OKB,
    OMG,
    ONT,
    PNT,
    QTUM,
    REN,
    ROSE,
    SAND,
    SKL,
    SNX,
    SOL,
    SRM,
    STX,
    SUSHI,
    SXP,
    THETA,
    TRX,
    UMA,
    UNI,
    VET,
    WEMIX,
    XEM,
    XLM,
    XPR,
    XRP,
    XTZ,
    YFI,
    YGG,
    ZIL,
    ZRX,
}

// Special cases for Tokens starting with number that cannot be directly assigned to enum
impl Token {
    fn to_token_string(self: Token) -> String {
        match self {
            _ => self.to_string(),
        }
    }
    fn from_token_string(symbol: &str) -> Result<Token, ParseError> {
        match symbol {
            _ => Token::from_str(symbol),
        }
    }
}

#[derive(ToString, EnumString, EnumIter, EnumPropertyTrait, Debug, Copy, Clone, PartialEq)]
enum Exchange {
    #[strum(props(data_source_id = "55"))]
    BIBOX = 0,
    #[strum(props(data_source_id = "54"))]
    BINANCE = 1,
    #[strum(props(data_source_id = "53"))]
    BITFINEX = 2,
    #[strum(props(data_source_id = "57"))]
    BITTREX = 3,
    #[strum(props(data_source_id = "78"))]
    BRAVENEWCOIN = 4,
    #[strum(props(data_source_id = "73"))]
    COINBASEPRO = 5,
    #[strum(props(data_source_id = "74"))]
    COINGECKO = 6,
    #[strum(props(data_source_id = "72"))]
    COINMARKETCAP = 7,
    #[strum(props(data_source_id = "71"))]
    CRYPTOCOMPARE = 8,
    #[strum(props(data_source_id = "76"))]
    HITBTC = 9,
    #[strum(props(data_source_id = "59"))]
    HUOBIPRO = 10,
    #[strum(props(data_source_id = "58"))]
    KRAKEN = 11,
    #[strum(props(data_source_id = "56"))]
    OKX = 12,
}

impl Exchange {
    fn from_u64(value: u64) -> Option<Exchange> {
        Exchange::iter().nth(value as usize)
    }
}

macro_rules! token_to_exchange_list {
    ($data:expr) => {
        match $data {
            Token::AAVE => "0100001110000",
            Token::ADA => "0100001110110",
            Token::ALGO => "0100001110100",
            Token::ATOM => "0100011110100",
            Token::AUDIO => "0100001110000",
            Token::AVAX => "0100011110101",
            Token::AXS => "0100001110100",
            Token::BAL => "0100001110100",
            Token::BAT => "0101001110110",
            Token::BCH => "0100001110100",
            Token::BNB => "0100001110000",
            Token::BORA => "0000001110000",
            Token::BTC => "0111111111110",
            Token::BTT => "0000001110100",
            Token::CAKE => "0000001110000",
            Token::CELO => "0100001110000",
            Token::COMP => "0101011110010",
            Token::CRO => "0000001110100",
            Token::CRV => "0100001110100",
            Token::DGB => "0000001110000",
            Token::DOGE => "0100001110100",
            Token::DOT => "0100001110100",
            Token::DYDX => "0100001110100",
            Token::EGLD => "0100001110000",
            Token::ENJ => "0100001110000",
            Token::EOS => "0110001110100",
            Token::ETH => "0111111111110",
            Token::FIL => "1100011101100",
            Token::FTM => "0100001110000",
            Token::FTT => "0100001110100",
            Token::GALA => "0000001110000",
            Token::HT => "0000001110100",
            Token::ICX => "0100001110100",
            Token::ILV => "0100001110000",
            Token::IMX => "0100001110100",
            Token::KLAY => "0100001100000",
            Token::KNC => "0100000000100",
            Token::KSM => "0000001110100",
            Token::LEO => "0000001110000",
            Token::LINK => "0101011110110",
            Token::LRC => "0100001110000",
            Token::LTC => "0110001110100",
            Token::LUNA => "0100001110100",
            Token::MANA => "0100001110100",
            Token::MATIC => "0100001110000",
            Token::MIOTA => "0000001110000",
            Token::MKR => "0100001110100",
            Token::MLN => "0000001110100",
            Token::MTL => "0100001110000",
            Token::NEAR => "0100001111101",
            Token::NEO => "0000001110000",
            Token::OKB => "0000001110000",
            Token::OMG => "0100001110100",
            Token::ONT => "0100001110100",
            Token::PNT => "0100001010000",
            Token::QTUM => "0100001110000",
            Token::REN => "0100001110100",
            Token::ROSE => "0100001110000",
            Token::SAND => "0100001110001",
            Token::SKL => "0100011110000",
            Token::SNX => "0100001110100",
            Token::SOL => "0100001100000",
            Token::SRM => "0100001110000",
            Token::STX => "0100001100000",
            Token::SUSHI => "0100001110100",
            Token::SXP => "0100001110000",
            Token::THETA => "0100001110100",
            Token::TRX => "0100001110100",
            Token::UMA => "0100001110000",
            Token::UNI => "0100001110000",
            Token::VET => "0100001110100",
            Token::WEMIX => "0000001110000",
            Token::XEM => "0000001110100",
            Token::XLM => "0100011110110",
            Token::XPR => "0000001110000",
            Token::XRP => "0110001110100",
            Token::XTZ => "0111001110100",
            Token::YFI => "0100001110100",
            Token::YGG => "0100001110000",
            Token::ZIL => "0100001110000",
            Token::ZRX => "0100001110100",
        }
    };
}

fn get_ds_input(exchange_id: u64, symbols: Vec<Token>) -> String {
    let exchange = Exchange::from_u64(exchange_id).unwrap();
    if API_SOURCE.contains(&exchange) {
        format!(
            "{}",
            symbols
                .iter()
                .map(|&x| x.to_token_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        format!(
            "{} {}",
            exchange.to_string().to_ascii_lowercase(),
            symbols
                .iter()
                .map(|&x| x.to_token_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

fn get_ds_from_exchange(exchange_id: u64) -> i64 {
    let exchange = match Exchange::from_u64(exchange_id) {
        Some(data) => data,
        None => panic!("Unsupported Exchange ID"),
    };
    if API_SOURCE.contains(&exchange) {
        i64::from_str(exchange.get_str("data_source_id").unwrap()).unwrap()
    } else {
        3i64 // CCXT Data source id
    }
}

fn get_symbols_from_input(exchange_id: u64, input: String) -> Vec<String> {
    let exchange = Exchange::from_u64(exchange_id).unwrap();
    if API_SOURCE.contains(&exchange) {
        input.split(" ").map(|x| x.to_string()).collect()
    } else {
        let mut v: Vec<String> = input.split(" ").map(|x| x.to_string()).collect();
        v.drain(0..1);
        v
    }
}

// Get list of exchange that needs to be called along with the symbols to call
// given a list of input symbols
fn get_exchange_map(symbols: Vec<String>) -> HashMap<u64, Vec<Token>> {
    let mut exchange_map = HashMap::new();
    for symbol in symbols {
        let symbol_token = Token::from_token_string(symbol.as_str()).unwrap();
        let mut exchange_binary = token_to_exchange_list!(symbol_token).chars();
        for i in 0..(EXCHANGE_COUNT as usize) {
            if exchange_binary.next() == Some('1') {
                match exchange_map.entry(i as u64) {
                    Entry::Vacant(e) => {
                        e.insert(vec![symbol_token]);
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(symbol_token);
                    }
                }
            }
        }
    }
    exchange_map
}

fn median(arr: &mut Vec<f64>) -> f64 {
    let len_arr = arr.len() as f64;
    if len_arr > 0f64 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = len_arr / 2f64;
        if len_arr as u64 % 2 == 0 {
            (arr[(mid - 1f64) as usize] + arr[mid as usize]) / 2f64
        } else {
            arr[mid as usize]
        }
    } else {
        0f64
    }
}

fn prepare_impl(input: Input) {
    let exchange_map = get_exchange_map(input.symbols);
    for (exchange_id, symbols) in exchange_map.iter() {
        oei::ask_external_data(
            *exchange_id as i64,
            get_ds_from_exchange(*exchange_id),
            get_ds_input(*exchange_id, symbols.to_vec()).as_bytes(),
        )
    }
}

#[no_mangle]
fn execute_impl(input: Input) -> Output {
    // Get the required exchange and associated symbols to query
    let exchange_map = get_exchange_map((*input.symbols).to_vec());
    // store the median price of each token requested from an exchange
    let mut exchange_medians: Vec<Option<Vec<f64>>> = vec![Some(vec![]); EXCHANGE_COUNT as usize];
    for (exchange_id, _symbols) in exchange_map.iter() {
        // Get the data source calldata for a given external ID
        let raw_input = ext::load_input::<String>(*exchange_id as i64);
        let mut prices = vec![vec![]; exchange_map[exchange_id].len()];
        let inputs: Vec<String> = raw_input.collect();
        if inputs.len() == 0 {
            exchange_medians[*exchange_id as usize] = None;
            continue;
        }
        // for each validator response for the exchange,
        // split the response into individual prices
        for raw in inputs {
            let px_list: Vec<f64> = raw
                .split(",")
                .filter_map(|x| x.parse::<f64>().ok())
                .collect();
            // for each token price, add it to the list of validator responses
            // for that token and exchange
            for (idx, &px) in px_list.iter().enumerate() {
                prices[idx].push(px);
            }
        }
        let mut median_prices = vec![0f64; prices.len()];
        for (idx, price) in prices.iter().enumerate() {
            median_prices[idx] = median(&mut price.to_vec());
        }
        exchange_medians[*exchange_id as usize] = Some(median_prices);
    }

    let mut symbol_pxs = HashMap::new();
    for (exchange_id, symbols) in exchange_map.iter() {
        let exchange_median = exchange_medians[*exchange_id as usize].as_ref();
        if exchange_median.is_none() {
            continue;
        }
        let exchange_median = exchange_median.unwrap();
        let symbols_vec =
            get_symbols_from_input(*exchange_id, get_ds_input(*exchange_id, symbols.to_vec()));

        for (symbol_id, symbol) in symbols_vec.iter().enumerate() {
            match symbol_pxs.entry(symbol.clone()) {
                Entry::Vacant(e) => {
                    e.insert(vec![exchange_median[symbol_id]]);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().push(exchange_median[symbol_id]);
                }
            }
        }
    }

    let mut rates = Vec::new();
    for symbol in input.symbols.iter() {
        rates.push((median(symbol_pxs.get_mut(*&symbol).unwrap()) * (input.multiplier as f64)) as u64)
    }
    Output { rates }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
