//! Example program to test out whole-file hashing

extern crate antifa;
extern crate data_encoding;

use std::env;
use std::path::Path;
use data_encoding::hex;

use antifa::hash::hash_file;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let filename = Path::new(&args[1]);

    let digest = hash_file(filename.into()).unwrap();
    println!("{}", hex::encode(digest.as_ref()).to_lowercase());
}
