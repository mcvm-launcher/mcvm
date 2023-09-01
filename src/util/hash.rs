use std::{
	fs::File,
	io::{BufReader, Read},
	path::Path,
};

use anyhow::Context;
use mcvm_shared::pkg::PackageAddonOptionalHashes;
use sha2::{Digest, Sha256, Sha512};

/// Length in bytes of a SHA-256 hash
pub const HASH_SHA256_RESULT_LENGTH: usize = 32;
/// Length in bytes of a SHA-512 hash
pub const HASH_SHA512_RESULT_LENGTH: usize = 64;

/// Digest a reader into a hasher
pub fn digest_reader<D: Digest, R: Read>(mut reader: R) -> anyhow::Result<Vec<u8>> {
	let mut digest = D::new();
	let mut buf = [0; 1024];

	loop {
		let count = reader.read(&mut buf)?;
		if count == 0 {
			break;
		}
		digest.update(&buf[..count]);
	}

	Ok(digest.finalize().to_vec())
}

/// Get a hash string as a hex
pub fn get_hash_str_as_hex(hash: &str) -> anyhow::Result<Vec<u8>> {
	Ok(hex::decode(hash)?)
}

/// The different hash types used by addons
#[derive(Copy, Clone)]
pub enum AddonHashType {
	/// SHA-256 hash
	SHA256,
	/// SHA-512 hash
	SHA512,
}

/// Best hash value and hash type from the get_best_hash function
pub struct BestHashResult(String, AddonHashType);

/// Get the best hash from a set of addon hashes
pub fn get_best_hash(hashes: &PackageAddonOptionalHashes) -> Option<BestHashResult> {
	let mut best = None;

	if let Some(hash) = &hashes.sha256 {
		best = Some(BestHashResult(hash.clone(), AddonHashType::SHA256));
	}

	if let Some(hash) = &hashes.sha512 {
		best = Some(BestHashResult(hash.clone(), AddonHashType::SHA512));
	}

	best
}

/// Get a reader's hash based on the get_best_hash function
pub fn get_reader_best_hash<R: Read>(
	reader: R,
	hash_type: AddonHashType,
) -> anyhow::Result<Vec<u8>> {
	match hash_type {
		AddonHashType::SHA256 => digest_reader::<Sha256, _>(reader),
		AddonHashType::SHA512 => digest_reader::<Sha512, _>(reader),
	}
}

/// Check the hash of a file based on the get_best_hash function. Returns true if the hash matches
pub fn hash_file_with_best_hash(path: &Path, result: BestHashResult) -> anyhow::Result<bool> {
	let BestHashResult(expected_hash, hash_type) = result;

	let file = File::open(path).context("Failed to open file for checksum")?;
	let mut file = BufReader::new(file);

	let actual_hash =
		get_reader_best_hash(&mut file, hash_type).context("Failed to compute file hash")?;

	let matches = actual_hash
		== get_hash_str_as_hex(&expected_hash).context("Failed to parse provided hash")?;

	Ok(matches)
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use sha2::Sha512;

	use super::*;

	#[test]
	fn test_digest_reader() {
		let text = "Hello";
		let cursor = Cursor::new(text);
		let hash = digest_reader::<Sha512, _>(cursor).unwrap();
		assert_eq!(
			hash,
			hex::decode("3615f80c9d293ed7402687f94b22d58e529b8cc7916f8fac7fddf7fbd5af4cf777d3d795a7a00a16bf7e7f3fb9561ee9baae480da9fe7a18769e71886b03f315")
				.unwrap()
		);
	}
}
