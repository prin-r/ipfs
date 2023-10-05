use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};
use phf::phf_map;

#[derive(OBIDecode, OBISchema)]
struct Input {
    symbols: Vec<String>,
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    rates: Vec<u64>,
}

const DS_ID: i64 = 1;

static SYMBOLS: phf::Map<&'static str, bool> = phf_map! {
    "1INCH" => true,
    "AAVE" => true,
    "ADA" => true,
    "ALCX" => true,
    "ALGO" => true,
    "ALPHA" => true,
    "ARB" => true,
    "ASTR" => true,
    "ATOM" => true,
    "AVAX" => true,
    "BAND" => true,
    "BAT" => true,
    "BETA" => true,
    "BNB" => true,
    "BTC" => true,
    "BTT" => true,
    "BUSD" => true,
    "C98" => true,
    "CAKE" => true,
    "CELO" => true,
    "CKB" => true,
    "CLV" => true,
    "COMP" => true,
    "CREAM" => true,
    "CRO" => true,
    "CRV" => true,
    "CUSD" => true,
    "DAI" => true,
    "DOGE" => true,
    "DOT" => true,
    "ETH" => true,
    "FRAX" => true,
    "FTM" => true,
    "GLMR" => true,
    "HEGIC" => true,
    "ICX" => true,
    "INJ" => true,
    "JOE" => true,
    "JST" => true,
    "KNC" => true,
    "KP3R" => true,
    "KSM" => true,
    "LINK" => true,
    "LTC" => true,
    "MANA" => true,
    "MATIC" => true,
    "MIM" => true,
    "MKR" => true,
    "MOVR" => true,
    "NFT" => true,
    "ONE" => true,
    "OP" => true,
    "OSMO" => true,
    "PERP" => true,
    "ROSE" => true,
    "SCRT" => true,
    "SFI" => true,
    "SHIB" => true,
    "SNX" => true,
    "SOL" => true,
    "SPELL" => true,
    "STRK" => true,
    "SUN" => true,
    "SUSD" => true,
    "SUSHI" => true,
    "TRX" => true,
    "TUSD" => true,
    "UNI" => true,
    "USDC" => true,
    "USDT" => true,
    "WBTC" => true,
    "XMR" => true,
    "XRP" => true,
    "YFI" => true,
};

fn prepare_impl(input: Input) {
    if input.symbols.is_empty()
        || input
            .symbols
            .iter()
            .any(|s| !SYMBOLS.contains_key(s.as_str()))
    {
        panic!("Either symbols are empty or a symbol is not a member of the symbol map");
    }

    oei::ask_external_data(1, DS_ID, input.symbols.join(" ").as_bytes())
}

fn aggregate<I>(strings: I, input_len: usize) -> Vec<u64>
where
    I: Iterator<Item = String>,
{
    strings
        .filter_map(|s| {
            let nums: Vec<u64> = s
                .split_whitespace()
                .filter_map(|n| n.parse().ok())
                .collect();
            nums.len().eq(&input_len).then(|| nums)
        })
        .fold(vec![Vec::new(); input_len], |mut acc, v| {
            for (vec, &num) in acc.iter_mut().zip(&v) {
                vec.push(num);
            }
            acc
        })
        .into_iter()
        .map(|mut nums| match nums.is_empty() {
            true => None,
            _ => {
                let mid = nums.len() / 2;
                Some(*nums.as_mut_slice().select_nth_unstable(mid).1)
            }
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

fn execute_impl(input: Input) -> Output {
    let rates: Vec<u64> = aggregate(ext::load_input::<String>(1), input.symbols.len());
    if rates.len() != input.symbols.len() {
        panic!("Invalid length");
    }
    Output { rates }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let r = aggregate(
            vec![
                "12 45 78".to_string(),
                "32 67 89".to_string(),
                "54 23 91".to_string(),
            ]
            .into_iter(),
            3,
        );
        assert_eq!(r, vec![32, 45, 89])
    }

    #[test]
    fn test_2() {
        let r = aggregate(
            vec![
                "xcjkzjkxkx".to_string(),
                "32 67 89".to_string(),
                "54 23 91".to_string(),
            ]
            .into_iter(),
            3,
        );
        assert_eq!(r, vec![54, 67, 91])
    }

    #[test]
    fn test_3() {
        let r = aggregate(
            vec![
                "xcjkzjkxkx".to_string(),
                "32 67 89".to_string(),
                "reqfgwegdfdsfs".to_string(),
            ]
            .into_iter(),
            3,
        );
        assert_eq!(r, vec![32, 67, 89])
    }

    #[test]
    fn test_4() {
        let r = aggregate(
            vec![
                "xcjkzjkxkx".to_string(),
                "".to_string(),
                "reqfgwegdfdsfs".to_string(),
            ]
            .into_iter(),
            3,
        );
        assert_eq!(r, vec![])
    }

    #[test]
    fn test_5() {
        let r = aggregate(vec![].into_iter(), 3);
        assert_eq!(r, vec![])
    }

    #[test]
    fn test_6() {
        let r = aggregate(vec!["4 1 2 3 5 6".to_string()].into_iter(), 6);
        assert_eq!(r, vec![4, 1, 2, 3, 5, 6])
    }
}
