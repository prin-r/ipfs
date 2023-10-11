use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};

#[derive(OBIDecode, OBISchema)]
struct Input { word: String }

#[derive(OBIEncode, OBISchema)]
struct Output { result: String }

fn prepare_impl(input: Input) {
    match input.word.len() {
        0 => panic!("Error: word.len() must be > 0"),
        _ => oei::ask_external_data(6, 1, input.word.as_bytes())
    }
}

fn execute_impl(_: Input) -> Output {
    Output { result: ext::load_majority::<String>(1).unwrap() }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
