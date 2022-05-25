use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};
use sha3::{Digest, Sha3_256};

#[derive(OBIDecode, OBISchema)]
struct Input {
    seed: String,
    time: u64,
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    hash: Vec<u8>,
}

const NUM_DS: u8 = 4;

const VRF_1: i64 = 82;
const VRF_2: i64 = 83;
const VRF_3: i64 = 84;
const VRF_4: i64 = 85;

fn decode_hex(input: &str) -> Vec<u8> {
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..i + 2], 16).unwrap())
        .collect()
}

fn get_pubkey_by_id(i: u8) -> String {
    match i {
        0 => "dca4c01a68b79c82ef53a4c400b7020a92afa58c2c2a514f33d1153e577ad3b7".into(),
        1 => "6bf857a5e0a33655707e764bd6a40896e1bfaec9d520cd87a17051606fe96fc7".into(),
        2 => "66217ae09f2a8ff5f0a74b69aab1a45b3713559d65013ec52c6a76e0f2c18378".into(),
        3 => "0b6ebe53e0e8665f43a6836fedacf22fb0b19f1136e90bf0e1705c5a1cf06460".into(),
        _ => panic!("Unknown index"),
    }
}

fn get_hash(x: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(x);
    let mut output = vec![0u8; 32];
    output.copy_from_slice(&hasher.finalize());
    output
}

fn get_random_ds_index_from_seed(s: &str, num_data_source: u8) -> u8 {
    let nds = num_data_source as u16;

    // compute remainders for [2^0,2^1,2^2,...,2^31] mod nds
    let pre_compute_remainders = (0..32).fold(vec![0u16; 32], |mut s, i| {
        s[31 - i] = match i {
            0 => 1,
            _ => (s[31 + 1 - i] * 256) % nds,
        };
        s
    });

    // selected data source index = random_hash_from_seed % NUM_DS
    get_hash(s.as_bytes())
        .iter()
        .map(|&x| x as u16)
        .zip(pre_compute_remainders.iter())
        .fold(0u16, |selected_ds_i, (h, r)| {
            (((h * r) % nds) + selected_ds_i) % nds
        }) as u8
}

fn mod_index_to_ds_id(i: u8) -> i64 {
    match i {
        0 => VRF_1,
        1 => VRF_2,
        2 => VRF_3,
        3 => VRF_4,
        _ => panic!("Unknown index"),
    }
}

#[no_mangle]
fn prepare_impl(input: Input) {
    let s = format!("{} {}", input.seed, input.time);
    oei::ask_external_data(
        1,
        mod_index_to_ds_id(get_random_ds_index_from_seed(&s, NUM_DS)),
        s.as_bytes(),
    );
}

fn str_to_vec(input: &String) -> Vec<u8> {
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..i + 2], 16).unwrap())
        .collect()
}

#[no_mangle]
fn execute_impl(input: Input) -> Output {
    let x = ext::load_majority::<String>(1).unwrap();

    assert!(x.len() == 288);
    // The first 160 characters is the proof
    let proof = decode_hex(&x[0..160]);

    let pubkey = get_pubkey_by_id(get_random_ds_index_from_seed(&format!("{} {}", input.seed, input.time), NUM_DS));
    let alpha = format!("{}:{}", input.seed, input.time);

    if !oei::ecvrf_verify(&decode_hex(&pubkey), &proof, alpha.as_bytes()) {
        panic!("Invalid result")
    }

    Output {
        hash: str_to_vec(&x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obi::get_schema;
    use std::collections::*;

    #[test]
    fn test_get_schema() {
        let mut schema = HashMap::new();
        Input::add_definitions_recursively(&mut schema);
        Output::add_definitions_recursively(&mut schema);
        let input_schema = get_schema(String::from("Input"), &schema);
        let output_schema = get_schema(String::from("Output"), &schema);
        println!("{}/{}", input_schema, output_schema);
        // assert_eq!(
        //     "{public_key:string,seed:string,time:u64}/{hash:bytes}",
        //     format!("{}/{}", input_schema, output_schema),
        // );
    }

    fn assert_all(input: String, expected_outputs: Vec<u8>) {
        for (i, n_ds) in expected_outputs.iter().zip(2..255) {
            assert_eq!(*i, get_random_ds_index_from_seed(&input, n_ds));
        }
    }

    #[test]
    fn test_get_random_ds_index_from_seed_1() {
        assert_all(
            format!("{} {}", String::from("mumu"), 1234u64),
            vec![
                1, 1, 1, 3, 1, 4, 1, 1, 3, 5, 1, 3, 11, 13, 9, 1, 1, 16, 13, 4, 5, 6, 1, 8, 3, 1,
                25, 20, 13, 3, 25, 16, 1, 18, 1, 19, 35, 16, 33, 9, 25, 14, 5, 28, 29, 23, 25, 4,
                33, 1, 29, 32, 1, 38, 25, 16, 49, 42, 13, 22, 3, 46, 25, 3, 49, 5, 1, 52, 53, 5, 1,
                37, 19, 58, 73, 60, 55, 65, 73, 1, 9, 16, 25, 18, 57, 49, 49, 18, 73, 81, 29, 34,
                23, 73, 25, 39, 53, 82, 33, 66, 1, 100, 81, 88, 85, 23, 1, 66, 93, 19, 25, 37, 73,
                98, 49, 55, 101, 18, 73, 82, 83, 91, 65, 8, 109, 54, 25, 100, 3, 117, 49, 130, 5,
                28, 1, 131, 121, 123, 53, 70, 5, 16, 73, 78, 37, 4, 93, 92, 133, 72, 73, 1, 137, 3,
                133, 123, 65, 85, 153, 144, 1, 156, 9, 148, 99, 86, 25, 16, 103, 73, 57, 70, 49,
                158, 137, 160, 107, 171, 73, 128, 81, 22, 121, 93, 127, 137, 117, 109, 73, 98, 25,
                169, 39, 133, 53, 196, 181, 74, 33, 139, 167, 165, 1, 173, 203, 190, 185, 16, 193,
                25, 85, 76, 23, 143, 1, 158, 175, 37, 93, 120, 19, 19, 25, 208, 37, 174, 73, 196,
                213, 214, 49, 135, 55, 23, 101, 223, 137, 198, 73, 80, 203, 163, 205, 53, 91, 16,
                65, 16, 133, 160, 109, 236, 181,
            ],
        );
    }

    #[test]
    fn test_get_random_ds_index_from_seed_2() {
        assert_all(
            format!("{} {}", String::from("lulu"), 4321u64),
            vec![
                0, 2, 2, 1, 2, 5, 6, 5, 6, 9, 2, 11, 12, 11, 6, 1, 14, 9, 6, 5, 20, 18, 14, 21, 24,
                23, 26, 10, 26, 7, 6, 20, 18, 26, 14, 34, 28, 11, 6, 11, 26, 35, 42, 41, 18, 8, 38,
                5, 46, 35, 50, 7, 50, 31, 54, 47, 10, 38, 26, 28, 38, 5, 6, 11, 20, 22, 18, 41, 26,
                31, 14, 61, 34, 71, 66, 75, 50, 67, 6, 50, 52, 81, 26, 1, 78, 68, 86, 74, 86, 89,
                18, 38, 8, 66, 38, 10, 54, 86, 46, 56, 86, 5, 102, 26, 60, 68, 50, 1, 86, 71, 54,
                100, 104, 41, 10, 50, 38, 103, 86, 108, 28, 11, 38, 121, 68, 14, 70, 35, 76, 4, 86,
                47, 22, 131, 86, 39, 110, 123, 26, 8, 102, 141, 86, 126, 134, 5, 34, 3, 146, 119,
                142, 86, 152, 131, 50, 18, 146, 113, 6, 110, 50, 58, 134, 86, 164, 53, 110, 102,
                86, 104, 78, 94, 68, 96, 86, 38, 74, 53, 86, 70, 180, 89, 110, 71, 38, 86, 102,
                131, 66, 19, 134, 161, 10, 11, 54, 160, 86, 197, 46, 89, 56, 68, 86, 11, 108, 41,
                102, 9, 26, 58, 166, 173, 68, 121, 158, 131, 110, 134, 86, 154, 182, 201, 166, 221,
                100, 76, 218, 122, 156, 152, 126, 171, 50, 196, 38, 146, 222, 234, 86, 1, 108, 212,
                150, 201, 134, 180, 38, 164, 246, 240, 194, 64, 14,
            ],
        );
    }

    #[test]
    fn test_get_random_ds_index_from_seed_3() {
        assert_all(
            format!(
                "{} {}",
                String::from("scaleremembernorthdeleteneighborhood"),
                1624128435u64
            ),
            vec![
                0, 2, 2, 4, 2, 1, 2, 5, 4, 2, 2, 4, 8, 14, 10, 10, 14, 4, 14, 8, 2, 4, 2, 9, 4, 23,
                22, 15, 14, 23, 10, 2, 10, 29, 14, 7, 4, 17, 34, 18, 8, 4, 2, 14, 4, 19, 26, 29,
                34, 44, 30, 30, 50, 24, 50, 23, 44, 8, 14, 27, 54, 50, 42, 4, 2, 55, 10, 50, 64,
                60, 50, 34, 44, 59, 42, 57, 56, 4, 74, 50, 18, 2, 50, 44, 4, 44, 2, 70, 14, 43, 50,
                23, 66, 4, 74, 25, 78, 68, 34, 9, 44, 19, 82, 29, 30, 24, 50, 70, 24, 44, 106, 112,
                80, 4, 102, 95, 8, 78, 74, 13, 88, 59, 54, 9, 50, 115, 42, 47, 4, 129, 2, 99, 122,
                104, 10, 117, 50, 17, 134, 113, 60, 134, 122, 44, 34, 29, 118, 50, 134, 125, 42,
                95, 134, 54, 134, 148, 4, 83, 74, 50, 50, 86, 18, 134, 2, 96, 50, 121, 44, 23, 90,
                3, 44, 134, 90, 8, 70, 53, 14, 106, 134, 149, 50, 44, 116, 112, 66, 50, 4, 165,
                170, 120, 122, 134, 78, 174, 68, 30, 34, 122, 110, 15, 146, 59, 122, 50, 186, 156,
                134, 80, 30, 131, 24, 4, 50, 85, 70, 107, 134, 95, 44, 118, 106, 59, 112, 94, 194,
                16, 4, 134, 218, 91, 212, 19, 126, 83, 78, 147, 74, 18, 134, 131, 210, 29, 182, 4,
                178, 2, 134, 11, 50, 211, 242,
            ],
        );
    }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
