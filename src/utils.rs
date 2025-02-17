#![allow(non_snake_case)]
extern crate rand;
extern crate alloc;
extern crate ark_secp256k1;

use ark_ff::Field;
use ark_ff::PrimeField;
use ark_ec::AffineRepr;
use ark_ec::short_weierstrass::SWCurveConfig;
use ark_ec::short_weierstrass::Affine;
use std::fs;
use ark_serialize::CanonicalSerialize;
use relations::curve_tree::SelRerandParameters;

pub fn print_affine_compressed<F: PrimeField,
P0: SWCurveConfig<BaseField = F> + Copy>(pt: Affine<P0>, name: &str) {
    let mut b = Vec::new();
        pt.serialize_compressed(&mut b).expect("Failed to serialize point");
    println!("This is the value of {}: {:#?}", name, hex::encode(&b));
}
// protocol requires three generators G, H, J:
// update: H will be gotten from the CurveTree rerandomization,
// so now only returning G, J
pub fn get_generators<F: PrimeField,
P0: SWCurveConfig<BaseField = F> + Copy>() -> (Affine<P0>, Affine<P0>){
let G = P0::GENERATOR;
//let H = affine_from_bytes_tai::<Affine<P0>>(b"this is H");
let J = affine_from_bytes_tai::<Affine<P0>>(b"this is J");
(G, J)
}

pub fn field_as_bytes<F: Field>(field: &F) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Err(e) = field.serialize_compressed(&mut bytes) {
        panic!("{}", e)
    }
    bytes
}

pub fn read_file_string(filepath: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(filepath)?;
    Ok(data)
}

pub fn write_file_string(filepath: &str, mut buf: Vec<u8>) -> () {
    fs::write(filepath, &mut buf).expect("Failed to write to file");
}

// an effective generation of NUMS deterministically:
pub fn affine_from_bytes_tai<C: AffineRepr>(bytes: &[u8]) -> C {
    extern crate crypto;
    use crypto::digest::Digest;
    use crypto::sha2::Sha256;

    for i in 0..=u8::max_value() {
        let mut sha = Sha256::new();
        sha.input(bytes);
        sha.input(&[i]);
        let mut buf = [0u8; 32];
        sha.result(&mut buf);
        let res = C::from_random_bytes(&buf);
        if let Some(point) = res {
            return point;
        }
    }
    panic!()
}

pub fn get_leaf_commitments<F: PrimeField,
                            P0: SWCurveConfig<BaseField = F>>(pubkey_file_path: &str)
                            -> Vec<Affine<P0>>{
    // this whole section is clunky TODO
    // (need to reverse each binary string, but reverse() is 'in place', hence loop,
    // so the "TODO" is how to do that more elegantly)
    let filestr:String = read_file_string(pubkey_file_path)
    .expect("my failure message");
    let hex_keys_vec = filestr.split_whitespace().collect::<Vec<_>>();
    let hex_keys_vec_count = hex_keys_vec.len();
    println!("Pubkey count: {}", hex_keys_vec_count);
    let hex_keys = hex_keys_vec.into_iter();
    let mut b = Vec::new();
    for s in hex_keys {
        let mut sbin = hex::decode(s).expect("hex decode failed");
        sbin.reverse();
        b.push(sbin.clone());
    }
    let leaf_commitments: Vec<Affine<P0>> = b
            .into_iter()
            .map(|x| <Affine<P0> as AffineRepr>::from_random_bytes(&x[..]).unwrap()).collect();
    leaf_commitments
}

pub fn create_permissible_points_and_randomnesses<
   F: PrimeField,
   P0: SWCurveConfig<BaseField = F> + Copy,
   P1: SWCurveConfig<BaseField = P0::ScalarField, ScalarField = P0::BaseField> + Copy,>(
    leaf_commitments: &[Affine<P0>],
    sr_params: &SelRerandParameters<P0, P1>,
) -> (Vec<Affine<P0>>, Vec<P1::BaseField>) {
    leaf_commitments
        .iter()
        .map(|commitment| {
            sr_params
                .even_parameters
                .uh
                .permissible_commitment(commitment, &sr_params.even_parameters.pc_gens.B_blinding)
        })
        .unzip()
}
