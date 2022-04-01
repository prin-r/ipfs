// This file was automatically generated
// This file was automatically generated on 2022-03-28 13:55:41.928679+00:00
// VERSION = 1

use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm::{execute_entry_point, ext, oei, prepare_entry_point};
use std::collections::hash_map::*;
use std::collections::HashMap;
use std::str::FromStr;
use strum::{EnumProperty, IntoEnumIterator};
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

const EXCHANGE_COUNT: u64 = 2;

const API_SOURCE: [Exchange; 2] = [
    Exchange::TERRASWAP,
    Exchange::ASTROPORT,
];

#[derive(ToString, EnumString, EnumIter, PartialEq, Debug, Copy, Clone)]
enum Token {
    ABR,
    ANC,
    APOLLO,
    ASTRO,
    ATLO,
    BRO,
    BTL,
    DPH,
    GLOW,
    HALO,
    KUJI,
    LOCAL,
    LOOP,
    LOOPR,
    LOTA,
    LUART,
    LUNI,
    LUV,
    LunaX,
    MARS,
    MIAW,
    MINE,
    MINT,
    MIR,
    MOON,
    ORION,
    ORNE,
    PLY,
    PRISM,
    Psi,
    ROBO,
    SAYVE,
    SDOLLAR,
    SITY,
    SPEC,
    STT,
    TFLOKI,
    TFTICII,
    TFTICIII,
    TLAND,
    TNS,
    TWD,
    VKR,
    WHALE,
    XDEFI,
    XRUNE,
    XTRA,
    aUST,
    bETH,
    bLuna,
    cLuna,
    mAAPL,
    mABNB,
    mAMD,
    mAMZN,
    mARKK,
    mBABA,
    mBTC,
    mCOIN,
    mDIS,
    mDOT,
    mETH,
    mFB,
    mGLXY,
    mGOOGL,
    mGS,
    mHOOD,
    mIAU,
    mJNJ,
    mKO,
    mMSFT,
    mNFLX,
    mNIO,
    mNKE,
    mNVDA,
    mPYPL,
    mQQQ,
    mSBUX,
    mSLV,
    mSPY,
    mSQ,
    mTSLA,
    mTWTR,
    mUSO,
    mVIXY,
    pLuna,
    vUST,
    wasAVAX,
    wbWBNB,
    weUSDC,
    wewstETH,
    whSD,
    wsSOL,
    wsstSOL,
}

#[derive(ToString, EnumString, EnumIter, EnumPropertyTrait, Debug, Copy, Clone, PartialEq)]
enum Exchange {
    #[strum(props(data_source_id = "289"))]
    TERRASWAP = 0,
    #[strum(props(data_source_id = "290"))]
    ASTROPORT = 1,
}

impl Exchange {
    fn from_u64(value: u64) -> Option<Exchange> {
        Exchange::iter().nth(value as usize)
    }
}

macro_rules! token_to_exchange_list {
    ($data:expr) => {
        match $data {
            Token::ABR => "10",
            Token::ANC => "11",
            Token::APOLLO => "11",
            Token::ASTRO => "11",
            Token::ATLO => "10",
            Token::BRO => "01",
            Token::BTL => "10",
            Token::DPH => "10",
            Token::GLOW => "10",
            Token::HALO => "10",
            Token::KUJI => "11",
            Token::LOCAL => "11",
            Token::LOOP => "10",
            Token::LOOPR => "10",
            Token::LOTA => "11",
            Token::LUART => "11",
            Token::LUNI => "10",
            Token::LUV => "10",
            Token::LunaX => "10",
            Token::MARS => "11",
            Token::MIAW => "10",
            Token::MINE => "11",
            Token::MINT => "10",
            Token::MIR => "11",
            Token::MOON => "10",
            Token::ORION => "11",
            Token::ORNE => "11",
            Token::PLY => "10",
            Token::PRISM => "11",
            Token::Psi => "11",
            Token::ROBO => "10",
            Token::SAYVE => "11",
            Token::SDOLLAR => "10",
            Token::SITY => "10",
            Token::SPEC => "10",
            Token::STT => "11",
            Token::TFLOKI => "11",
            Token::TFTICII => "10",
            Token::TFTICIII => "10",
            Token::TLAND => "10",
            Token::TNS => "10",
            Token::TWD => "11",
            Token::VKR => "11",
            Token::WHALE => "10",
            Token::XDEFI => "11",
            Token::XRUNE => "10",
            Token::XTRA => "10",
            Token::aUST => "11",
            Token::bETH => "11",
            Token::bLuna => "11",
            Token::cLuna => "01",
            Token::mAAPL => "10",
            Token::mABNB => "10",
            Token::mAMD => "10",
            Token::mAMZN => "10",
            Token::mARKK => "10",
            Token::mBABA => "10",
            Token::mBTC => "10",
            Token::mCOIN => "10",
            Token::mDIS => "10",
            Token::mDOT => "10",
            Token::mETH => "10",
            Token::mFB => "10",
            Token::mGLXY => "10",
            Token::mGOOGL => "10",
            Token::mGS => "10",
            Token::mHOOD => "10",
            Token::mIAU => "10",
            Token::mJNJ => "10",
            Token::mKO => "10",
            Token::mMSFT => "10",
            Token::mNFLX => "10",
            Token::mNIO => "10",
            Token::mNKE => "10",
            Token::mNVDA => "10",
            Token::mPYPL => "10",
            Token::mQQQ => "10",
            Token::mSBUX => "10",
            Token::mSLV => "10",
            Token::mSPY => "10",
            Token::mSQ => "10",
            Token::mTSLA => "10",
            Token::mTWTR => "10",
            Token::mUSO => "10",
            Token::mVIXY => "10",
            Token::pLuna => "10",
            Token::vUST => "10",
            Token::wasAVAX => "01",
            Token::wbWBNB => "10",
            Token::weUSDC => "01",
            Token::wewstETH => "11",
            Token::whSD => "10",
            Token::wsSOL => "10",
            Token::wsstSOL => "01",
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
                .map(|&x| x.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        format!(
            "{} {}",
            exchange.to_string().to_ascii_lowercase(),
            symbols
                .iter()
                .map(|&x| x.to_string())
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
        let symbol_token = Token::from_str(symbol.as_str()).unwrap();
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
