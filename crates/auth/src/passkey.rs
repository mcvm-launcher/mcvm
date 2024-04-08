use anyhow::Context;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rsa::{rand_core::CryptoRngCore, traits::PaddingScheme, BigUint, RsaPrivateKey, RsaPublicKey};

/// Create the RSA public and private key from a passkey
pub fn generate_keys(passkey: &str) -> anyhow::Result<RsaPrivateKey> {
	// FIXME: With this method, we just start overwriting the seed once we get
	// to a large enough index in the passkey. This is not very secure.
	let mut seed = [0u8; 32];
	for (i, byte) in passkey.bytes().enumerate() {
		seed[i % 32] = byte;
	}
	let mut rng = ChaCha8Rng::from_seed(seed);
	RsaPrivateKey::new(&mut rng, 1024).context("Failed to create RSA private key")
}

/// Create a public key from it's stored n and the default exponent
pub fn recreate_public_key(n: &[u64]) -> anyhow::Result<RsaPublicKey> {
	let n = BigUint::from_slice_native(n);
	let e = BigUint::from_slice_native(&[65537]);
	RsaPublicKey::new(n, e).context("Failed to recreate public key")
}

/// Create a public key from it's stored n and the default exponent
pub fn recreate_public_key_bytes(n: &[u8]) -> anyhow::Result<RsaPublicKey> {
	let n = BigUint::from_bytes_le(n);
	let e = BigUint::from_slice_native(&[65537]);
	RsaPublicKey::new(n, e).context("Failed to recreate public key")
}

/// Encrypt a string in chunks
pub fn encrypt_chunks<R: CryptoRngCore, P: PaddingScheme + Copy>(
	data: &[u8],
	public_key: &RsaPublicKey,
	rng: &mut R,
	padding: P,
	key_size: usize,
) -> anyhow::Result<Vec<Vec<u8>>> {
	let mut out = Vec::with_capacity(data.len());
	for chunk in data.chunks(key_size / 2) {
		let data = public_key
			.encrypt(rng, padding, chunk)
			.context("Failed to encrypt data chunk")?;
		out.push(data);
	}

	Ok(out)
}

/// Decrypt a string in chunks
pub fn decrypt_chunks<P: PaddingScheme + Copy>(
	data: &[Vec<u8>],
	private_key: &RsaPrivateKey,
	padding: P,
) -> anyhow::Result<Vec<u8>> {
	let mut out = Vec::with_capacity(data.len());
	for (i, chunk) in data.iter().enumerate() {
		let data = private_key
			.decrypt(padding, chunk)
			.with_context(|| format!("Failed to decrypt data chunk {i}"))?;
		out.extend(data);
	}

	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests that passkey generation works and produces the same result every time
	#[test]
	fn test_passkey_generation() {
		let passkey = "foo bar baz";
		let key = generate_keys(passkey).expect("Failed to generate passkey");
		key.validate().expect("Private key validation failed");

		let expected_n = &[
			13215879769031435363,
			3042605133295529232,
			4999223922724939278,
			16422739374832148648,
			7624678381681168461,
			4552657176200489251,
			4979487057228561449,
			7670538712497119850,
			11538492019645330197,
			17973144035373568151,
			6538386160387571421,
			10444279286813265960,
			4980591232309381569,
			11486140692866479892,
			11182720846508821729,
			15336914180461716952,
		];
		let expected =
			recreate_public_key(expected_n).expect("Failed to create expected public key");

		assert_eq!(key.to_public_key(), expected);
	}
}
