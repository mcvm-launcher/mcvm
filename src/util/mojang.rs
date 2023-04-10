/// For checking rule actions in Mojang json files
pub fn is_allowed(action: &str) -> bool {
	action == "allow"
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn test_allowed() {
		assert!(is_allowed("allow"));
		assert!(!is_allowed("disallow"));
	}
}
