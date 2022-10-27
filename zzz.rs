use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};

#[derive(OBIDecode, OBISchema)]
struct Input {
    base: String,
    quote: String,
    timestamp: u64,
    multiplier: u64,
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    price: u64,
}

const D1: i64 = 486;
const D2: i64 = 487;
const D3: i64 = 488;

fn prepare_impl(input: Input) {
    let calldata = format!("{} {} {}", input.quote, input.base, input.multiplier);
    oei::ask_external_data(D1, D1, calldata.as_bytes());
    oei::ask_external_data(D2, D2, calldata.as_bytes());
    oei::ask_external_data(D3, D3, calldata.as_bytes());
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

fn execute_impl(input: Input) -> Output {
    let a: f64 = median(ext::load_input::<String>(D1).map(|s| s.parse().unwrap()).collect());
    let b: f64 = median(ext::load_input::<String>(D2).map(|s| s.parse().unwrap()).collect());
    let c: f64 = median(ext::load_input::<String>(D3).map(|s| s.parse().unwrap()).collect());

    Output {
        price: (f64::max(f64::min(a, b), f64::min(f64::max(a, b), c)) * input.multiplier as f64) as u64
    }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
