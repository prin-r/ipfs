use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};

#[derive(OBIDecode, OBISchema)]
struct Input {
    _null: u8,
}

/// Each entry is encoded as 20 bytes:
/// - first 12 bytes: ASCII symbol (left-justified, zero-padded)
/// - last   8 bytes: big-endian u64 price
#[derive(OBIEncode, OBISchema)]
struct Output {
    result: Vec<u8>,
}

const DATA_SOURCE_ID: i64 = 24;
const EXTERNAL_ID: i64 = 0;

#[no_mangle]
fn prepare_impl(_input: Input) {
    oei::ask_external_data(EXTERNAL_ID, DATA_SOURCE_ID, b"");
}

#[no_mangle]
fn execute_impl(_input: Input) -> Output {
    let raw_results = ext::load_input::<String>(EXTERNAL_ID);
    let results: Vec<String> = raw_results.collect();

    Output {
        result: prices_to_bytes(&ext::stats::majority(results).unwrap()).unwrap(),
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MalformedPair {
        pair: String,
        index: usize,
    },
    SymbolTooLong {
        symbol: String,
        len: usize,
        index: usize,
    },
    InvalidPrice {
        price: String,
        symbol: String,
        index: usize,
    },
    EmptyResult,
}

pub fn prices_to_bytes(prices_str: &str) -> Result<Vec<u8>, ParseError> {
    if prices_str.is_empty() {
        return Err(ParseError::EmptyResult);
    }

    // Parse each "symbol:price" into a [u8;20] buffer
    let chunks: Vec<[u8; 20]> = prices_str
        .split(',')
        .enumerate()
        .map(|(i, pair)| {
            let mut parts = pair.splitn(2, ':');

            let symbol = parts.next().ok_or(ParseError::MalformedPair {
                pair: pair.into(),
                index: i,
            })?;
            let price_str = parts.next().ok_or(ParseError::MalformedPair {
                pair: pair.into(),
                index: i,
            })?;

            if symbol.is_empty() {
                return Err(ParseError::MalformedPair {
                    pair: pair.into(),
                    index: i,
                });
            }

            // symbol length check
            let sym_bytes = symbol.as_bytes();
            if sym_bytes.len() > 12 {
                return Err(ParseError::SymbolTooLong {
                    symbol: symbol.into(),
                    len: sym_bytes.len(),
                    index: i,
                });
            }

            // parse the price or error
            let price = price_str
                .parse::<u64>()
                .map_err(|_e| ParseError::InvalidPrice {
                    price: price_str.into(),
                    symbol: symbol.into(),
                    index: i,
                })?;

            // build the 20-byte buffer
            let mut buf = [0u8; 20];
            buf[..sym_bytes.len()].copy_from_slice(sym_bytes); // symbol + zero-pad
            buf[12..].copy_from_slice(&price.to_be_bytes()); // big-endian price
            Ok(buf)
        })
        .collect::<Result<_, _>>()?;

    // flatten Vec<[u8;20]> → Vec<u8>
    let mut result = Vec::with_capacity(chunks.len() * 20);
    for chunk in chunks {
        result.extend_from_slice(&chunk);
    }

    Ok(result)
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_prices_to_bytes_valid_input() {
        let prices_input = "Gold:338561000000,Silver:3633800000,Copper:483140000,Platinum:128250000000,HRC Steel:85606000000,Iron Ore:9546000000".to_string();
        let result = prices_to_bytes(&prices_input).unwrap(); // Unwrap here as we expect Ok

        let expected_hex = "476f6c6400000000000000000000004ed3cee24053696c76657200000000000000000000d8976340436f70706572000000000000000000001ccc21a0506c6174696e756d000000000000001ddc4bb28048524320537465656c00000000000013ee83e58049726f6e204f7265000000000000000238fc6680";
        let expected_bytes = hex::decode(expected_hex).unwrap();

        assert_eq!(result, expected_bytes);

        // 6 symbols * 20 bytes/symbol
        assert_eq!(result.len(), 6 * 20);
    }

    #[test]
    fn test_prices_to_bytes_empty_input() {
        let prices_input = "".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(result, Err(ParseError::EmptyResult));
    }

    #[test]
    fn test_prices_to_bytes_single_item() {
        let prices_input = "Test:12345".to_string();
        let result = prices_to_bytes(&prices_input).unwrap();
        // Test (4 bytes) + 8 zeros + 12345 (u64)
        let expected_hex = "5465737400000000000000000000000000003039";
        let expected_bytes = hex::decode(expected_hex).unwrap();
        assert_eq!(result, expected_bytes);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_prices_to_bytes_symbol_exactly_12_bytes() {
        let prices_input = "TwelveChars!:100".to_string();
        let result = prices_to_bytes(&prices_input).unwrap();
        // 12 chars + 0 padding + 100 as u64
        let expected_hex = "5477656c76654368617273210000000000000064";
        let expected_bytes = hex::decode(expected_hex).unwrap();
        assert_eq!(result, expected_bytes);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_prices_to_bytes_price_zero() {
        let prices_input = "ZeroPrice:0".to_string();
        let result = prices_to_bytes(&prices_input).unwrap();
        // ZeroPrice + 3 zeros + 0 as u64
        let expected_hex = "5a65726f50726963650000000000000000000000";
        let expected_bytes = hex::decode(expected_hex).unwrap();
        assert_eq!(result, expected_bytes);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_prices_to_bytes_with_trailing_comma() {
        let prices_input = "Test:100,".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::MalformedPair {
                pair: "".into(),
                index: 1
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_malformed_pair_no_colon() {
        let prices_input = "Gold:100,NoColonHere".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::MalformedPair {
                pair: "NoColonHere".into(),
                index: 1
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_malformed_pair_empty_symbol() {
        let prices_input = ":100".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::MalformedPair {
                pair: ":100".into(),
                index: 0
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_malformed_pair_empty_price() {
        let prices_input = "Gold:".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::InvalidPrice {
                price: "".into(),
                symbol: "Gold".into(),
                index: 0
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_symbol_too_long() {
        let prices_input = "ThisSymbolIsTooLong:100".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::SymbolTooLong {
                symbol: "ThisSymbolIsTooLong".into(),
                len: 19,
                index: 0
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_invalid_price_format() {
        let prices_input = "Invalid:not_a_number".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::InvalidPrice {
                price: "not_a_number".into(),
                symbol: "Invalid".into(),
                index: 0
            })
        );
    }

    #[test]
    fn test_prices_to_bytes_u64_overflow() {
        // This number is 2^64, which is one more than max u64.
        let prices_input = "TooLarge:18446744073709551616".to_string();
        let result = prices_to_bytes(&prices_input);
        assert_eq!(
            result,
            Err(ParseError::InvalidPrice {
                price: "18446744073709551616".into(),
                symbol: "TooLarge".into(),
                index: 0
            })
        );
    }
}
