use anyhow::ensure;

/// Converts a hexadecimal uuid to the hyphenated form
pub fn hyphenate_uuid(uuid: &str) -> anyhow::Result<String> {
	ensure!(uuid.len() == 32, "UUID is not the correct length of 32");

	let p1 = &uuid[..8];
	let p2 = &uuid[8..12];
	let p3 = &uuid[12..16];
	let p4 = &uuid[16..20];
	let p5 = &uuid[20..];

	let out = format!("{p1}-{p2}-{p3}-{p4}-{p5}");
	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_uuid_hyphenation() {
		let uuid = "8b447756e18044d3bfbfdbc8df722db4";
		assert_eq!(
			hyphenate_uuid(uuid).unwrap(),
			"8b447756-e180-44d3-bfbf-dbc8df722db4".to_string()
		);
	}
}
