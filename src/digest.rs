use problem::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub use shaman::digest::Digest;
pub use shaman::sha1::Sha1;
pub use shaman::sha2::Sha256;

/// Calculates SHA-256 hash from list of strings and returns hex representation.
pub fn hex_digest<'i, S>(
    parts: impl IntoIterator<Item = &'i S, IntoIter = impl Iterator<Item = &'i S>>,
) -> String
where S: AsRef<[u8]> + 'i
{
    parts
        .into_iter()
        .fold(Sha256::new(), |mut digest, part| {
            digest.input(part.as_ref());
            digest
        })
        .result_str()
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
            break;
        }
        hash.input(buf);
        file.consume(len);
    }

    Ok(hash.result_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_digest() {
        assert_eq!(hex_digest(&["foo", "bar"]), "c3ab8ff13720e8ad9047dd39466b3c8974e592c2fa383d4a3960714caef0c4f2".to_owned());
        assert_eq!(hex_digest(&[b"foo", b"bar"]), "c3ab8ff13720e8ad9047dd39466b3c8974e592c2fa383d4a3960714caef0c4f2".to_owned());
    }
}
