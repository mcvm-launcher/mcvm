/// Gets a core package that is included with the binary
pub fn get_core_package(package: &str) -> Option<&'static str> {
	match package {
		"animated-textures-support" => Some(include_str!("animated-textures-support.pkg.txt")),
		"cem-support" => Some(include_str!("cem-support.pkg.txt")),
		"cit-support" => Some(include_str!("cit-support.pkg.txt")),
		"ctm-support" => Some(include_str!("ctm-support.pkg.txt")),
		"custom-colors-support" => Some(include_str!("custom-colors-support.pkg.txt")),
		"custom-gui-support" => Some(include_str!("custom-gui-support.pkg.txt")),
		"custom-sky-support" => Some(include_str!("custom-sky-support.pkg.txt")),
		"emissive-blocks-support" => Some(include_str!("emissive-blocks-support.pkg.txt")),
		"emissive-entities-support" => Some(include_str!("emissive-entities-support.pkg.txt")),
		"fabric-rendering-api" => Some(include_str!("fabric-rendering-api.pkg.txt")),
		"fabriclike-api" => Some(include_str!("fabriclike-api.pkg.txt")),
		"kotlin-support" => Some(include_str!("kotlin-support.pkg.txt")),
		"optifine-resource-packs" => Some(include_str!("optifine-resource-packs.pkg.txt")),
		"random-entities-support" => Some(include_str!("random-entities-support.pkg.txt")),
		"shader-support" => Some(include_str!("shader-support.pkg.txt")),
		"splash-screen-support" => Some(include_str!("splash-screen-support.pkg.txt")),
		_ => None,
	}
}
