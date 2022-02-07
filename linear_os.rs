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

// Add non CCXT data source to this array
const API_SOURCE: [Exchange; 5] = [
  Exchange::CRYPTOCOMPARE,
  Exchange::COINGECKO,
  Exchange::COINBASEPRO,
  Exchange::COINMARKETCAP,
  Exchange::BRAVENEWCOIN,
];

#[derive(ToString, EnumString, EnumIter, PartialEq, Debug, Copy, Clone)]
enum Token {
  BTC,
  ETH,
  USDT,
  XRP,
  LINK,
  DOT,
  BCH,
  LTC,
  ADA,
  BSV,
  CRO,
  BNB,
  EOS,
  XTZ,
  TRX,
  XLM,
  ATOM,
  XMR,
  OKB,
  USDC,
  NEO,
  XEM,
  LEO,
  HT,
  VET,
  YFI,
  MIOTA,
  LEND,
  SNX,
  DASH,
  COMP,
  ZEC,
  ETC,
  OMG,
  MKR,
  ONT,
  NXM,
  AMPL,
  BAT,
  THETA,
  DAI,
  REN,
  ZRX,
  ALGO,
  FTT,
  DOGE,
  KSM,
  WAVES,
  EWT,
  DGB,
  KNC,
  ICX,
  TUSD,
  SUSHI,
  BTT,
  BAND,
  EGLD,
  ANT,
  NMR,
  USDP,
  LSK,
  LRC,
  HBAR,
  BAL,
  RUNE,
  YFII,
  LUNA,
  DCR,
  SC,
  STX,
  ENJ,
  BUSD,
  OCEAN,
  RSR,
  SXP,
  BTG,
  BZRX,
  SRM,
  SNT,
  SOL,
  CKB,
  BNT,
  CRV,
  MANA,
  KAVA,
  MATIC,
  TRB,
  REP,
  FTM,
  TOMO,
  ONE,
  WNXM,
  PAXG,
  WAN,
  SUSD,
  RLC,
  OXT,
  RVN,
  FNX,
  RENBTC,
  WBTC,
  DIA,
  BTM,
  IOTX,
  FET,
  JST,
  MCO,
  KMD,
  BTS,
  QKC,
  YAMV2,
  XZC,
  UOS,
  AKRO,
  HNT,
  HOT,
  KAI,
  OGN,
  WRX,
  KDA,
  ORN,
  FOR,
  AST,
  STORJ,
  TWOKEY,
  ABYSS,
  BLZ,
  BTU,
  CND,
  CVC,
  DGX,
  ELF,
  EQUAD,
  EURS,
  FXC,
  GDC,
  GEN,
  GHT,
  GNO,
  GVT,
  IOST,
  KEY,
  LOOM,
  MET,
  MFG,
  MLN,
  MTL,
  MYB,
  NEXXO,
  NPXS,
  OST,
  PAY,
  PBTC,
  PLR,
  PLTC,
  PNK,
  PNT,
  POLY,
  POWR,
  QNT,
  RAE,
  REQ,
  RSV,
  SAN,
  SPIKE,
  SPN,
  STMX,
  TKN,
  TKX,
  TRYB,
  UBT,
  UPP,
  USDS,
  VIDT,
  XHV,
  CREAM,
  UNI,
  LINA,
  XVS,
  UMA,
  CELO,
  QTUM,
  HYN,
  ZIL,
  ZB,
  FIL,
  ALPHA,
  TWT,
  PERP,
  DPI,
  MTA,
  AAVE,
  GRT,
  KP3R,
  YAM,
  PICKLE,
  SFI,
  BOR,
  OBTC,
  CAKE,
  HEGIC,
  FRAX,
  SCRT,
  MVL,
  STRK,
  MIR,
  ANC,
  INDEX,
  ARPA,
  AUTO,
  UST,
  AETH,
  ALCX,
  OHM,
  MIM,
  MOVR,
  AVAX,
  INJ,
  JOE,
  ORCA,
  BEL,
  ORC,
  SHIB,
  AXS,
  ROSE,
  C98,
  CUSD,
  DYDX,
  IMX,
  BORA,
  SKL,
}

// Special cases for Tokens starting with number that cannot be directly assigned to enum
impl Token {
  fn to_token_string(self: Token) -> String {
    match self {
      Token::TWOKEY => "2KEY".to_string(),
      _ => self.to_string(),
    }
  }
  fn from_token_string(symbol: &str) -> Result<Token, ParseError> {
    match symbol {
      "2KEY" => Ok(Token::TWOKEY),
      _ => Token::from_str(symbol),
    }
  }
}

#[derive(ToString, EnumString, EnumIter, EnumPropertyTrait, Debug, Copy, Clone, PartialEq)]
enum Exchange {
  #[strum(props(data_source_id = "57"))]
  BRAVENEWCOIN = 0,
  #[strum(props(data_source_id = "58"))]
  CRYPTOCOMPARE = 1,
  #[strum(props(data_source_id = "112"))]
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
  GATEIO = 18,
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
      Token::XRP => "0111110010000000000",
      Token::LINK => "0111111101000000000",
      Token::DOT => "0111110000000000000",
      Token::BCH => "0111110000000000000",
      Token::LTC => "0111110010000000000",
      Token::ADA => "0111110100000000000",
      Token::BSV => "0111010010010000000",
      Token::CRO => "0111010000000000001",
      Token::BNB => "0111100000000000001",
      Token::EOS => "0111110010000000000",
      Token::XTZ => "0111110011000000000",
      Token::TRX => "0111110000000000000",
      Token::XLM => "0111111100000000000",
      Token::ATOM => "0111111000000000000",
      Token::XMR => "0111110000000000000",
      Token::OKB => "0111000000000000001",
      Token::USDC => "0111100100010000000",
      Token::NEO => "0111000000000000001",
      Token::XEM => "0111010000000000000",
      Token::LEO => "0111000000000000001",
      Token::HT => "0111010000000000000",
      Token::VET => "0111110000000000000",
      Token::YFI => "0111110000000000000",
      Token::MIOTA => "0111000000000000000",
      Token::LEND => "0110000000000000000",
      Token::SNX => "0111110000000000000",
      Token::DASH => "0111110000000000000",
      Token::COMP => "0111101101000000000",
      Token::ZEC => "0111110000000000000",
      Token::ETC => "0111110000000000000",
      Token::OMG => "0111110000000000000",
      Token::MKR => "0111110000000000000",
      Token::ONT => "0111110000000000000",
      Token::NXM => "0011000000000000000",
      Token::AMPL => "0111000000000000001",
      Token::BAT => "0111110101000000000",
      Token::THETA => "0111110000000000000",
      Token::DAI => "0111001110000000000",
      Token::REN => "0111110000000000000",
      Token::ZRX => "0111110000000000000",
      Token::ALGO => "0111110000000000000",
      Token::FTT => "0111110000000000000",
      Token::DOGE => "0111110000000000000",
      Token::KSM => "0111010000000000000",
      Token::WAVES => "0111110000000000000",
      Token::EWT => "0111000000000000001",
      Token::DGB => "0111000000000000000",
      Token::KNC => "0111110000000000001",
      Token::ICX => "0111110000000000000",
      Token::TUSD => "0111100000000000000",
      Token::SUSHI => "0111110000000000000",
      Token::BTT => "0111110000000000000",
      Token::BAND => "0111110000000000000",
      Token::EGLD => "0111100000000000000",
      Token::ANT => "0111110000000000000",
      Token::NMR => "0111100000000000000",
      Token::USDP => "0111100000000000000",
      Token::LSK => "0111100000000000000",
      Token::LRC => "0111100000000000000",
      Token::HBAR => "0111100000000000000",
      Token::BAL => "0111110000000000000",
      Token::RUNE => "0111100000000000001",
      Token::YFII => "0111110000000000000",
      Token::LUNA => "0111110000000000000",
      Token::DCR => "0111110000000000000",
      Token::SC => "0111100000000000000",
      Token::STX => "0011100000000000001",
      Token::ENJ => "0111100000000000000",
      Token::BUSD => "0111100000000000000",
      Token::OCEAN => "0111100000000000000",
      Token::RSR => "0111110000000000000",
      Token::SXP => "0111100000000000000",
      Token::BTG => "0111100000000000000",
      Token::BZRX => "0111100000000000000",
      Token::SRM => "0111100000000000000",
      Token::SNT => "0111010000000000000",
      Token::SOL => "0111110100000000001",
      Token::CKB => "0111010000000000000",
      Token::BNT => "0111110000000000000",
      Token::CRV => "0111110000000000000",
      Token::MANA => "0111110000000000000",
      Token::KAVA => "0111100000000000000",
      Token::MATIC => "0111100000000000000",
      Token::TRB => "0111110000000000000",
      Token::REP => "0111100000000000000",
      Token::FTM => "0111100000000000000",
      Token::TOMO => "0111100000000000000",
      Token::ONE => "0111110000000000000",
      Token::WNXM => "0110110000000000000",
      Token::PAXG => "0111100000000000000",
      Token::WAN => "0111100000000000000",
      Token::SUSD => "0111000000000000000",
      Token::RLC => "0111100000000000000",
      Token::OXT => "0011100000000000001",
      Token::RVN => "0010110000000000000",
      Token::FNX => "0010000000000000000",
      Token::RENBTC => "0111000000000000000",
      Token::WBTC => "0111000000000000000",
      Token::DIA => "0111100000000000000",
      Token::BTM => "0111010000010000000",
      Token::IOTX => "0111101000000000000",
      Token::FET => "0111101000000000000",
      Token::JST => "0111100000010000000",
      Token::MCO => "0101000000000000000",
      Token::KMD => "0111000000000000000",
      Token::BTS => "0111000000000000000",
      Token::QKC => "0111000000000000000",
      Token::YAMV2 => "0111000000000000000",
      Token::XZC => "0100000000000000000",
      Token::UOS => "0011000000000000001",
      Token::AKRO => "0111000000000000000",
      Token::HNT => "0111100000000000000",
      Token::HOT => "0111100000000000000",
      Token::KAI => "0111000000000000000",
      Token::OGN => "0111000000000000000",
      Token::WRX => "0111000000000000000",
      Token::KDA => "0111000000000000000",
      Token::ORN => "0011100000000000000",
      Token::FOR => "0111000000000000000",
      Token::AST => "0111000000000000000",
      Token::STORJ => "0111000000000000000",
      Token::TWOKEY => "0011000000000000000",
      Token::ABYSS => "0111000000000000000",
      Token::BLZ => "0111110000000000000",
      Token::BTU => "0111000000000000000",
      Token::CND => "0111000000000000000",
      Token::CVC => "0111110000000000000",
      Token::DGX => "0111000000000000000",
      Token::ELF => "0111010000000000000",
      Token::EQUAD => "0111000000000000000",
      Token::EURS => "0111000000000000000",
      Token::FXC => "0000000000000000000",
      Token::GDC => "0011000000000000000",
      Token::GEN => "0011000000000000000",
      Token::GHT => "0010000000000000000",
      Token::GNO => "0111000000000000000",
      Token::GVT => "0111000000000000000",
      Token::IOST => "0111110000000000000",
      Token::KEY => "0111100000000000000",
      Token::LOOM => "0111010000000000000",
      Token::MET => "0111000000000000000",
      Token::MFG => "0111000000000000000",
      Token::MLN => "0111010000000000000",
      Token::MTL => "0111100000000000000",
      Token::MYB => "0111000000000000000",
      Token::NEXXO => "0010000000000000000",
      Token::NPXS => "0111000000000000000",
      Token::OST => "0111000000000000000",
      Token::PAY => "0111000000000000000",
      Token::PBTC => "0011000000000000000",
      Token::PLR => "0111000000000000000",
      Token::PLTC => "0100000000000000000",
      Token::PNK => "0111000000000000000",
      Token::PNT => "0110100000000000000",
      Token::POLY => "0111000000000000000",
      Token::POWR => "0111000000000000000",
      Token::QNT => "0111000000000000000",
      Token::RAE => "0011000000000000000",
      Token::REQ => "0111000000000000000",
      Token::RSV => "0111000000000000000",
      Token::SAN => "0111000000000000000",
      Token::SPIKE => "0011000000000000000",
      Token::SPN => "0110000000000000000",
      Token::STMX => "0111000000000000000",
      Token::TKN => "0111000000000000000",
      Token::TKX => "0010000000000000000",
      Token::TRYB => "0111000000000000000",
      Token::UBT => "0111000000000000000",
      Token::UPP => "0111000000000000000",
      Token::USDS => "0110000000000000000",
      Token::VIDT => "0111000000000000000",
      Token::XHV => "0010000000000000000",
      Token::CREAM => "0011000000000000000",
      Token::UNI => "0111100000000000000",
      Token::LINA => "0111000000000000000",
      Token::XVS => "0111100000000000000",
      Token::UMA => "0111100000000000000",
      Token::CELO => "0111100000000000000",
      Token::QTUM => "0111100000000000000",
      Token::HYN => "0110000000000000000",
      Token::ZIL => "0111100000000000000",
      Token::ZB => "0111000000000000000",
      Token::FIL => "0111100000000000001",
      Token::ALPHA => "0111100000000000000",
      Token::TWT => "0111100000000000000",
      Token::PERP => "0111000000000000000",
      Token::DPI => "0111000000000000000",
      Token::MTA => "0111000000000000000",
      Token::AAVE => "0111100000000000000",
      Token::GRT => "0111100000000000000",
      Token::KP3R => "0111000000000000000",
      Token::YAM => "0111000000000000000",
      Token::PICKLE => "0111000000000000000",
      Token::SFI => "0011000000000000000",
      Token::BOR => "0111000000000000000",
      Token::OBTC => "0010000000000000000",
      Token::CAKE => "0111100000000000000",
      Token::HEGIC => "0111000000000000000",
      Token::FRAX => "0111000000000000000",
      Token::SCRT => "0111000000000000000",
      Token::MVL => "0111000000000000000",
      Token::STRK => "0011000000000000000",
      Token::MIR => "0011100000000000000",
      Token::ANC => "0111000000000000000",
      Token::INDEX => "0011000000000000000",
      Token::ARPA => "0111100000000000000",
      Token::AUTO => "0011100000000000000",
      Token::UST => "0111001000010000000",
      Token::AETH => "0101000000000000000",
      Token::ALCX => "0111000000001000010",
      Token::OHM => "0111000000000000000",
      Token::MIM => "0111000000000000000",
      Token::MOVR => "0111000100000000000",
      Token::AVAX => "0111111000010000000",
      Token::INJ => "0111110100000000000",
      Token::JOE => "0111000000000000001",
      Token::ORCA => "0011000000000000000",
      Token::BEL => "0111100000000000000",
      Token::ORC => "0011000000000000000",
      Token::SHIB => "0111111000010000000",
      Token::AXS => "0111110000000000000",
      Token::ROSE => "0111100000000000000",
      Token::C98 => "0111100000000000000",
      Token::CUSD => "0111000000000000000",
      Token::DYDX => "0111110000000000000",
      Token::IMX => "0111011000000000000",
      Token::BORA => "0111000000000001100",
      Token::SKL => "0111101000000000000",
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
