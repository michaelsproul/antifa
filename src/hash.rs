//! Hashing scheme used is:
//!
//! hash(file) = H(0x00 || contents)
//! hash(dir) = H(0x01 || hash(child0) || hash(child1) || ...)
//!
//! This is similar to the Merkle tree used in certificate transparency (but with n-way branching).
//! See: https://tools.ietf.org/html/rfc6962#section-2.1

use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use ring::digest::{Context, Digest, SHA256};

const BUF_SIZE: usize = 4096;

/// Hash a file by computing H(0x00 || file_contents).
pub fn hash_file(path: &Path) -> io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = vec![0; BUF_SIZE];
    let mut file = File::open(path)?;

    context.update(&[0x00]);
    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        context.update(&buffer[..bytes_read]);
    }

    Ok(context.finish())
}

/// Hash a directory, given hashes for its children. Hash is H(0x01 || d0 || d1 || ...).
pub fn combine_digests(digests: Vec<&Digest>) -> Digest {
    let mut context = Context::new(&SHA256);

    context.update(&[0x01]);
    for digest in digests {
        context.update(digest.as_ref());
    }

    context.finish()
}
