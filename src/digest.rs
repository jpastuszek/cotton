use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;
use problem::*;
pub use shaman;
pub use shaman::digest::Digest;
pub use shaman::sha1::Sha1;
pub use shaman::sha2::Sha256;

/// Calculates SHA-256 hash from list of strings and returns hex representation.
pub fn hex_digest<'i>(parts: impl IntoIterator<Item = &'i str, IntoIter = impl Iterator<Item = &'i str>>) -> String {
    parts.into_iter().fold(Sha256::new(), |mut digest, part| { digest.input_str(part); digest }).result_str()
}

/// Calculates SHA-256 hash from contents of a (potentially large) file and returns hex
/// representation.
pub fn hex_digest_file(path: impl AsRef<Path>) -> Result<String, Problem> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();

    loop {
        let buf = file.fill_buf()?;
        let len = buf.len();

        if len == 0 {
            break
        }
        hash.input(buf);
        file.consume(len);
    }

    Ok(hash.result_str())
}
