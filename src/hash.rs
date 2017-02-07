use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use ring::digest::{digest, Context, Digest, SHA256};

const BUF_SIZE: usize = 512 * 1024;

pub fn dummy_digest() -> Digest {
    digest(&SHA256, &[])
}

pub fn hash_file(path: &Path) -> io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = vec![0; BUF_SIZE];
    let mut file = File::open(path)?;

    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        context.update(&buffer[..bytes_read]);
    }

    Ok(context.finish())
}

/// Combine digests by hashing them together, like in a Merkle tree.
pub fn combine_digests(digests: Vec<&Digest>) -> Digest {
    let mut context = Context::new(&SHA256);

    for digest in digests {
        context.update(digest.as_ref());
    }

    context.finish()
}
