use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;
use std::fmt::{self, Display};
use std::error::Error;

pub use sha2::{Digest as DigestTrait, Sha256};
use sha2::digest::generic_array::GenericArray;
use hex::{self, FromHexError};
use digest::OutputSizeUser;

#[derive(Debug)]
pub enum DigestError {
    FromHexError(FromHexError),
    LengthMissmatch { got: usize , expected: usize },
}

impl From<FromHexError> for DigestError {
    fn from(err: FromHexError) -> DigestError {
        DigestError::FromHexError(err)
    }
}

impl Display for DigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigestError::FromHexError(_) => write!(f, "error converting hex string to digest"),
            DigestError::LengthMissmatch { got, expected } => write!(f, "digest length missmmatch, got {} bytes, expected {} bytes", got, expected),
        }
    }
}

impl Error for DigestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DigestError::FromHexError(err) => Some(err),
            DigestError::LengthMissmatch { .. } => None,
        }
    }
}

/// Represents SHA2-256 hash value
#[derive(PartialEq, Eq, Clone)]
pub struct Digest(GenericArray<u8, <Sha256 as OutputSizeUser>::OutputSize>);

impl Digest {
    /// Create new Digest from give bytes as is.
    pub fn new(value: &[u8]) -> Result<Digest, DigestError> {
        if value.len() != <Sha256 as OutputSizeUser>::output_size() {
            Err(DigestError::LengthMissmatch { got: value.len(), expected: <Sha256 as OutputSizeUser>::output_size() })
        } else {
            Ok(Digest(GenericArray::clone_from_slice(&value)))
        }
    }

    /// Create new Digest from give hex encoded bytes as is.
    pub fn from_hex(hex: &str) -> Result<Digest, DigestError> {
        Digest::new(&hex::decode(hex)?)
    }

    /// Calculate digest from content read from a reader.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Digest, io::Error> {
        let mut digest = Sha256::new();
        std::io::copy(reader, &mut digest)?;
        Ok(Digest(digest.finalize()))
    }

    /// Calculate digest from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Digest, io::Error> {
        let mut file = BufReader::new(File::open(path)?);
        Digest::from_reader(&mut file)
    }

    /// Calculate digest from a stream of byte buffers.
    pub fn from_buffers<S: AsRef<[u8]>>(buffers: impl IntoIterator<Item = S, IntoIter = impl Iterator<Item = S>>) -> Digest {
        let mut hash = Sha256::new();
        for buffer in buffers {
            hash.update(buffer);
        }
        Digest(hash.finalize())
    }

    /// Calculate digest from bytes.
    pub fn from_bytes<S: AsRef<[u8]>>(bytes: S) -> Digest {
        Digest::from_buffers(Some(bytes))
    }

    /// Encode digest value as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Returns digest value as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Unwraps digest value GenericArray.
    pub fn unwrap(&self) -> GenericArray<u8, <Sha256 as OutputSizeUser>::OutputSize> {
        self.0
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DigestSha256")
            .field(&format_args!("{:X}", &self.0))
            .finish()
    }
}

#[derive(Debug)]
pub enum FileDigestError {
    IoError(io::Error),
}

impl fmt::Display for FileDigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileDigestError::IoError(_) => write!(f, "failed to digest a file due to IO error"),
        }
    }
}

impl Error for FileDigestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FileDigestError::IoError(err) => Some(err),
        }
    }
}

impl From<io::Error> for FileDigestError {
    fn from(err: io::Error) -> FileDigestError {
        FileDigestError::IoError(err)
    }
}

/// Calculates SHA2-256 hash from list of strings and returns hex representation.
pub fn hex_digest<S: AsRef<[u8]>>(
    parts: impl IntoIterator<Item = S, IntoIter = impl Iterator<Item = S>>,
) -> String {
    Digest::from_buffers(parts).to_hex()
}

/// Calculates SHA2-256 hash from contents of a (potentially large) file and returns hex
/// representation.
pub fn hex_digest_file(path: impl AsRef<Path>) -> Result<String, FileDigestError> {
    Ok(Digest::from_file(path)?.to_hex())
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
