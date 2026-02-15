use sha2::{Sha256, Digest};
use sha1::Sha1;
use md5::Md5;

pub fn sha256(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

pub fn sha1(input: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

pub fn md5(input: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

pub fn crc32(input: &[u8]) -> String {
    let checksum = crc32fast::hash(input);
    format!("{:08x}", checksum)
}
