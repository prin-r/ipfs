use obi::{OBIDecode, OBIEncode, OBISchema};
use owasm_kit::{execute_entry_point, ext, oei, prepare_entry_point};
use strum::{EnumCount, EnumProperty};
use strum_macros::{FromRepr, EnumCount as EnumCountMacro, EnumProperty as EnumPropertyMacro, Display};
use sha3::{Digest, Sha3_256};
use std::str::FromStr;
use hex;

#[derive(OBIDecode, OBISchema)]
struct Input {
    seed: Vec<u8>,
    time: u64, // In Unix time
    worker_address: Vec<u8>, // The worker should use this field to prevent front-running.
}

#[derive(OBIEncode, OBISchema)]
struct Output {
    result: Vec<u8>
}

#[derive(Debug, Copy, Clone, PartialEq, Display, EnumPropertyMacro, EnumCountMacro, FromRepr)]
#[repr(usize)]
enum DataSources {
    #[strum(props(pubkey="99812aab99423aa8033d8a6990993f31046c403997e83efd621421a166229a0e", ds_id="515"))]
    VRF1,
    #[strum(props(pubkey="8deed23561be3733009d05ae678a5d7ce80304373564acbe24875966bc58e5e9", ds_id="516"))]
    VRF2,
    #[strum(props(pubkey="7291bbcc98cf0c55aff388b9a06fb2090f992bd0380f1d888bd804d0d25321cd", ds_id="517"))]
    VRF3,
    #[strum(props(pubkey="726678872aa98bd078faf0c513908e6c8dd5ce841e937d8c1e6e11e1822e2720", ds_id="518"))]
    VRF4,
}

fn get_hash(x: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(x);
    let mut output = vec![0u8; 32];
    output.copy_from_slice(&hasher.finalize());
    output
}

fn get_random_ds_index_from_seed(s: &str, num_data_source: u8) -> usize {
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
        }) as usize
}

fn get_ds_from_input(ds_input: &str) -> DataSources {
    DataSources::from_repr(
        get_random_ds_index_from_seed(ds_input, DataSources::COUNT as u8)
    ).unwrap()
}

fn prepare_impl(input: Input) {
    if input.seed.len() != 32 {
        panic!("Error seed must be bytes32");
    }

    if input.time < 1 {
        panic!("Error time must be > 0");
    }

    let ds_input = format!("{} {}", hex::encode(input.seed), input.time);
    oei::ask_external_data(
        1,
        i64::from_str( get_ds_from_input(&ds_input).get_str("ds_id").unwrap()).unwrap(),
        ds_input.as_bytes(),
    );
}

fn execute_impl(input: Input) -> Output {
    let concat_data = ext::load_majority::<String>(1).unwrap();
    assert_eq!(concat_data.len(), 288);

    let ds_input = format!("{} {}", hex::encode(input.seed), input.time);
    let verification_result = oei::ecvrf_verify(
        &hex::decode(get_ds_from_input(&ds_input).get_str("pubkey").unwrap()).unwrap(),
        // The first 160 characters is the proof
        &hex::decode(&concat_data[0..160]).unwrap(),
        ds_input.as_bytes(),
    );

    match verification_result {
        Ok(true) => Output { result: get_hash(&hex::decode(&concat_data[160..288]).unwrap()) },
        Ok(false) => panic!("verification result is false"),
        Err(err_code) => panic!("verification error with code {}", err_code),
    }
}

prepare_entry_point!(prepare_impl);
execute_entry_point!(execute_impl);
