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

const EXCHANGE_COUNT: u64 = 19;

const CCXT_DS_ID: i64 = 207;

// Add non CCXT data source to this array
const API_SOURCE: [Exchange; 6] = [
    Exchange::CRYPTOCOMPARE,
    Exchange::COINGECKO,
    Exchange::COINBASEPRO,
    Exchange::COINMARKETCAP,
    Exchange::BRAVENEWCOIN,
    Exchange::OSMOSIS,
];

#[derive(ToString, EnumString, EnumIter, PartialEq, Debug, Copy, Clone)]
enum Token {
    BTC,
    ETH,
    USDT,
    INJ,
    BNB,
    LUNA,
    UST,
    ANC,
    ATOM,
    CRO,
    MIR,
    SCRT,
    STX,
    OSMO,
    MOVR,
    AVAX,
    SOL,
    FTM,
    NEAR,
    DOGE,
    DOT,
    ADA,
    COMP,
    HT,
    KSM,
    LINK,
    UNI,
    XRP,
    YFI,
    AAVE,
    ALCX,
    ALPHA,
    BAL,
    BCH,
    CAKE,
    CRV,
    EOS,
    HBAR,
    INDEX,
    IOTX,
    LTC,
    MATIC,
    OHM,
    PERP,
    THETA,
    XTZ,
    FTT,
    ZIL,
    EGLD,
    HNT,
    KAI,
    KDA,
    ONE,
    TOMO,
    FIL,
}

// Special cases for Tokens starting with number that cannot be directly assigned to enum
impl Token {
    fn to_token_string(self: Token) -> String {
        self.to_string()
    }
    fn from_token_string(symbol: &str) -> Result<Token, ParseError> {
        Token::from_str(symbol)
    }
}

#[derive(ToString, EnumString, EnumIter, EnumPropertyTrait, Debug, Copy, Clone, PartialEq)]
enum Exchange {
    #[strum(props(data_source_id = "57"))]
    BRAVENEWCOIN = 0,
    #[strum(props(data_source_id = "58"))]
    CRYPTOCOMPARE = 1,
    #[strum(props(data_source_id = "208"))]
    COINGECKO = 2,
    #[strum(props(data_source_id = "62"))]
    COINMARKETCAP = 3,
    BINANCE = 4,
    HUOBIPRO = 5,
    #[strum(props(data_source_id = "119"))]
    COINBASEPRO = 6,
    KRAKEN = 7,
    BITFINEX = 8,
    BITTREX = 9,
    BITSTAMP = 10,
    OKEX = 11,
    FTX = 12,
    HITBTC = 13,
    ITBIT = 14,
    BITHUMB = 15,
    COINONE = 16,
    BIBOX = 17,
    #[strum(props(data_source_id = "241"))]
    OSMOSIS = 18,
}

impl Exchange {
    fn from_u64(value: u64) -> Option<Exchange> {
        Exchange::iter().nth(value as usize)
    }
}

macro_rules! token_to_exchange_list {
    ($data:expr) => {
        match $data {
            Token::BTC => "0111111111000000000",
            Token::ETH => "0111111111000000000",
            Token::USDT => "0111001110001000000",
            Token::INJ => "0111110100000000000",
            Token::BNB => "0111100000000000000",
            Token::LUNA => "0111110000000000000",
            Token::UST => "0111001000010000000",
            Token::ANC => "0111000000000000000",
            Token::ATOM => "0111111000000000000",
            Token::CRO => "0111010000000000000",
            Token::MIR => "0011100000000000000",
            Token::SCRT => "0111000000000000000",
            Token::STX => "0011100000000000000",
            Token::OSMO => "0111000000000000001",
            Token::MOVR => "0111000100000000000",
            Token::AVAX => "0111111000010000000",
            Token::SOL => "0011100000000000000",
            Token::FTM => "0111100000000000000",
            Token::NEAR => "0111110000010100000",
            Token::DOGE => "0111110000000000000",
            Token::DOT => "0111110000000000000",
            Token::ADA => "0111110100000000000",
            Token::COMP => "0111101101000000000",
            Token::HT => "0111010000000000000",
            Token::KSM => "0111010000000000000",
            Token::LINK => "0111111101000000000",
            Token::UNI => "0111100000000000000",
            Token::XRP => "0111110010000000000",
            Token::YFI => "0111110000000000000",
            Token::AAVE => "0111100000000000000",
            Token::ALCX => "0111000000001000010",
            Token::ALPHA => "0111100000010000000",
            Token::BAL => "0111110000000000000",
            Token::BCH => "0111110000000000000",
            Token::CAKE => "0111000000000000000",
            Token::CRV => "0111110000000000000",
            Token::EOS => "0111110010000000000",
            Token::HBAR => "0111100000000000000",
            Token::INDEX => "0111000000000000000",
            Token::IOTX => "0111000000000000000",
            Token::LTC => "0111110010000000000",
            Token::MATIC => "0111100000000000000",
            Token::OHM => "0111000000000000000",
            Token::PERP => "0111000000000000000",
            Token::THETA => "0111110000000000000",
            Token::XTZ => "0111110011000000000",
            Token::FTT => "0111110000000000000",
            Token::ZIL => "0111100000000000000",
            Token::EGLD => "0111100000000000000",
            Token::HNT => "0111100000000000000",
            Token::KAI => "0111000000000000000",
            Token::KDA => "0111000000000000000",
            Token::ONE => "0111110000000000000",
            Token::TOMO => "0111100000000000000",
            Token::FIL => "0111111000010010000",
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
        CCXT_DS_ID // CCXT Data source id
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
        rates.push(
            (median(symbol_pxs.get_mut(*&symbol).unwrap()) * (input.multiplier as f64)) as u64,
        )
    }
    Output { rates }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
