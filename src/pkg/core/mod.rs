use std::collections::HashSet;

use mcvm_pkg::{repo::RepoPkgEntry, PackageContentType};

const ALL_CORE_PACKAGE_IDS: [&str; 25] = [
	"animated-textures-support",
	"cem-support",
	"cit-support",
	"ctm-support",
	"custom-colors-support",
	"custom-gui-support",
	"custom-sky-support",
	"emissive-blocks-support",
	"emissive-entities-support",
	"fabric-rendering-api",
	"fabriclike-api",
	"fail",
	"hd-fonts-support",
	"kotlin-support",
	"kotlin-support-forgelin",
	"kubejs-script-support",
	"natural-textures-support",
	"none",
	"optifine-resource-packs",
	"optifine-support",
	"quilted-fabric-api",
	"quilt-standard-libraries",
	"random-entities-support",
	"shader-support",
	"splash-screen-support",
];

const ANIMATED_TEXTURES_SUPPORT: &str = include_str!("animated-textures-support.pkg.txt");
const CEM_SUPPORT: &str = include_str!("cem-support.pkg.txt");
const CIT_SUPPORT: &str = include_str!("cit-support.pkg.txt");
const CTM_SUPPORT: &str = include_str!("ctm-support.pkg.txt");
const CUSTOM_COLORS_SUPPORT: &str = include_str!("custom-colors-support.pkg.txt");
const CUSTOM_GUI_SUPPORT: &str = include_str!("custom-gui-support.pkg.txt");
const CUSTOM_SKY_SUPPORT: &str = include_str!("custom-sky-support.pkg.txt");
const EMISSIVE_BLOCKS_SUPPORT: &str = include_str!("emissive-blocks-support.pkg.txt");
const EMISSIVE_ENTITIES_SUPPORT: &str = include_str!("emissive-entities-support.pkg.txt");
const FABRIC_RENDERING_API: &str = include_str!("fabric-rendering-api.json");
const FABRICLIKE_API: &str = include_str!("fabriclike-api.pkg.txt");
const FAIL: &str = include_str!("fail.pkg.txt");
const HD_FONTS_SUPPORT: &str = include_str!("hd-fonts-support.pkg.txt");
const KOTLIN_SUPPORT: &str = include_str!("kotlin-support.pkg.txt");
const KOTLIN_SUPPORT_FORGELIN: &str = include_str!("kotlin-support-forgelin.pkg.txt");
const KUBEJS_SCRIPT_SUPPORT: &str = include_str!("kubejs-script-support.pkg.txt");
const NATURAL_TEXTURES_SUPPORT: &str = include_str!("natural-textures-support.pkg.txt");
const NONE: &str = include_str!("none.pkg.txt");
const OPTIFINE_RESOURCE_PACKS: &str = include_str!("optifine-resource-packs.pkg.txt");
const OPTIFINE_SUPPORT: &str = include_str!("optifine-support.pkg.txt");
const QUILTED_FABRIC_API: &str = include_str!("quilted-fabric-api.json");
const QUILT_STANDARD_LIBRARIES: &str = include_str!("quilt-standard-libraries.json");
const RANDOM_ENTITIES_SUPPORT: &str = include_str!("random-entities-support.pkg.txt");
const SHADER_SUPPORT: &str = include_str!("shader-support.pkg.txt");
const SPLASH_SCREEN_SUPPORT: &str = include_str!("splash-screen-support.pkg.txt");

/// Gets a core package that is included with the binary
pub fn get_core_package(package: &str) -> Option<&'static str> {
	match package {
		"animated-textures-support" => Some(ANIMATED_TEXTURES_SUPPORT),
		"cem-support" => Some(CEM_SUPPORT),
		"cit-support" => Some(CIT_SUPPORT),
		"ctm-support" => Some(CTM_SUPPORT),
		"custom-colors-support" => Some(CUSTOM_COLORS_SUPPORT),
		"custom-gui-support" => Some(CUSTOM_GUI_SUPPORT),
		"custom-sky-support" => Some(CUSTOM_SKY_SUPPORT),
		"emissive-blocks-support" => Some(EMISSIVE_BLOCKS_SUPPORT),
		"emissive-entities-support" => Some(EMISSIVE_ENTITIES_SUPPORT),
		"fabric-rendering-api" => Some(FABRIC_RENDERING_API),
		"fabriclike-api" => Some(FABRICLIKE_API),
		"fail" => Some(FAIL),
		"hd-fonts-support" => Some(HD_FONTS_SUPPORT),
		"kotlin-support" => Some(KOTLIN_SUPPORT),
		"kotlin-support-forgelin" => Some(KOTLIN_SUPPORT_FORGELIN),
		"kubejs-script-support" => Some(KUBEJS_SCRIPT_SUPPORT),
		"natural-textures-support" => Some(NATURAL_TEXTURES_SUPPORT),
		"none" => Some(NONE),
		"optifine-resource-packs" => Some(OPTIFINE_RESOURCE_PACKS),
		"optifine-support" => Some(OPTIFINE_SUPPORT),
		"quilted-fabric-api" => Some(QUILTED_FABRIC_API),
		"quilt-standard-libraries" => Some(QUILT_STANDARD_LIBRARIES),
		"random-entities-support" => Some(RANDOM_ENTITIES_SUPPORT),
		"shader-support" => Some(SHADER_SUPPORT),
		"splash-screen-support" => Some(SPLASH_SCREEN_SUPPORT),
		_ => None,
	}
}

/// Gets the content type of a core package
pub fn get_core_package_content_type(package: &str) -> Option<PackageContentType> {
	match package {
		"animated-textures-support" => Some(PackageContentType::Script),
		"cem-support" => Some(PackageContentType::Script),
		"cit-support" => Some(PackageContentType::Script),
		"ctm-support" => Some(PackageContentType::Script),
		"custom-colors-support" => Some(PackageContentType::Script),
		"custom-gui-support" => Some(PackageContentType::Script),
		"custom-sky-support" => Some(PackageContentType::Script),
		"emissive-blocks-support" => Some(PackageContentType::Script),
		"emissive-entities-support" => Some(PackageContentType::Script),
		"fabric-rendering-api" => Some(PackageContentType::Declarative),
		"fabriclike-api" => Some(PackageContentType::Script),
		"fail" => Some(PackageContentType::Script),
		"hd-fonts-support" => Some(PackageContentType::Script),
		"kotlin-support" => Some(PackageContentType::Script),
		"kotlin-support-forgelin" => Some(PackageContentType::Script),
		"kubejs-script-support" => Some(PackageContentType::Script),
		"natural-textures-support" => Some(PackageContentType::Script),
		"none" => Some(PackageContentType::Script),
		"optifine-resource-packs" => Some(PackageContentType::Script),
		"optifine-support" => Some(PackageContentType::Script),
		"quilted-fabric-api" => Some(PackageContentType::Declarative),
		"quilt-standard-libraries" => Some(PackageContentType::Declarative),
		"random-entities-support" => Some(PackageContentType::Script),
		"shader-support" => Some(PackageContentType::Script),
		"splash-screen-support" => Some(PackageContentType::Script),
		_ => None,
	}
}

pub fn is_core_package(package: &str) -> bool {
	get_core_package(package).is_some()
}

pub fn get_all_core_packages() -> Vec<(String, RepoPkgEntry)> {
	let mut out = Vec::new();
	for pkg in ALL_CORE_PACKAGE_IDS {
		let content_type = get_core_package_content_type(pkg).expect("Content type should exist");
		out.push((
			pkg.to_string(),
			RepoPkgEntry {
				url: None,
				path: None,
				content_type: Some(content_type),
				flags: HashSet::new(),
			},
		));
	}

	out
}

pub fn get_core_package_count() -> usize {
	ALL_CORE_PACKAGE_IDS.len()
}

#[cfg(test)]
mod tests {
	use super::*;
	use mcvm_pkg::parse_and_validate;

	#[test]
	fn test_core_package_parse() {
		for package in ALL_CORE_PACKAGE_IDS {
			let contents = get_core_package(package).unwrap();
			let content_type = get_core_package_content_type(package).unwrap();
			parse_and_validate(contents, content_type).unwrap();
		}
	}
}
