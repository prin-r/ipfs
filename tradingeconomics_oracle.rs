use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, prepare_entry_point, oei, ext};

#[derive(OBIDecode, OBISchema)]
struct Input {
    _null: u8,
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    keys: Vec<String>,
    values: Vec<u64>,
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

    let majority = ext::stats::majority(results).unwrap();

    let mut keys = Vec::new();
    let mut values = Vec::new();

    for pair in majority.split(',') {
        let mut parts = pair.split(':');
        if let (Some(key), Some(value_str)) = (parts.next(), parts.next()) {
            keys.push(key.to_string());
            if let Ok(value) = value_str.parse::<u64>() {
                values.push(value);
            }
        }
    }

    Output { keys, values }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
