pub use shaman;
pub use shaman::digest::Digest;
pub use shaman::sha1::Sha1;
pub use shaman::sha2::Sha256;

/// Calculates SHA-256 hash from list of strigns and returns hex representation.
pub fn hex_digest<'i>(parts: impl IntoIterator<Item = &'i str, IntoIter = impl Iterator<Item = &'i str>>) -> String {
    parts.into_iter().fold(Sha256::new(), |mut digest, part| { digest.input_str(part); digest }).result_str()
}
