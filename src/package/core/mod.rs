/// Gets a core package that is included with the binary
pub fn get_core_package(package: &str) -> Option<&'static str> {
	match package {
		"fabriclike-api" => Some(include_str!("fabriclike-api.pkg.txt")),
		"fabric-rendering-api" => Some(include_str!("fabric-rendering-api.pkg.txt")),
		"cit-support" => Some(include_str!("cit-support.pkg.txt")),
		"cem-support" => Some(include_str!("cem-support.pkg.txt")),
		"connected-textures-support" => Some(include_str!("connected-textures-support.pkg.txt")),
		"shader-support" => Some(include_str!("shader-support.pkg.txt")),
		_ => None,
	}
}
