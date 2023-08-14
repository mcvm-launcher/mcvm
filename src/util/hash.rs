use std::io::Read;

use sha2::Digest;

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

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use hex_literal::hex;
	use sha2::Sha512;

	use super::*;

	#[test]
	fn test_digest_reader() {
		let text = "Hello";
		let cursor = Cursor::new(text);
		let hash = digest_reader::<Sha512, _>(cursor).unwrap();
		assert_eq!(
			hash,
			hex!("3615f80c9d293ed7402687f94b22d58e529b8cc7916f8fac7fddf7fbd5af4cf777d3d795a7a00a16bf7e7f3fb9561ee9baae480da9fe7a18769e71886b03f315")
		);
	}
}
