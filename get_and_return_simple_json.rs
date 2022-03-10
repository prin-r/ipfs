use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm::{execute_entry_point, ext, oei, prepare_entry_point};

const DS:i64 = 263;

#[derive(OBIDecode, OBISchema)]
struct Input {
    sliced_index_input: i8
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    result: String,
}

fn prepare_impl(input: Input) {
    oei::ask_external_data(1, DS, format!("{}", input.sliced_index_input).as_bytes())
}

#[no_mangle]
fn execute_impl(_input: Input) -> Output {
    Output { result: ext::load_majority::<String>(1).unwrap() }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
