use std::collections::HashSet;

use mcvm_pkg::{repo::RepoPkgEntry, PackageContentType};

macro_rules! define_core_packages {
	($($pkg:literal,$ext:literal,$id:ident,$content:ident);*$(;)?) => {
		const ALL_CORE_PACKAGE_IDS: &[&str] = &[
			$(
				$pkg,
			)*
		];

		$(
			const $id: &str = include_str!(concat!($pkg, ".", $ext));
		)*

		pub fn get_core_package(package: &str) -> Option<&'static str> {
			match package {
				$(
					$pkg => Some($id),
				)*
				_ => None,
			}
		}

		pub fn get_core_package_content_type(package: &str) -> Option<PackageContentType> {
			match package {
				$(
					$pkg => Some(PackageContentType::$content),
				)*
				_ => None,
			}
		}
	};
}

define_core_packages! {
	"animated-textures-support", "pkg.txt", ANIMATED_TEXTURES_SUPPORT, Script;
	"cem-support", "pkg.txt", CEM_SUPPORT, Script;
	"cit-support", "pkg.txt", CIT_SUPPORT, Script;
	"ctm-support", "pkg.txt", CTM_SUPPORT, Script;
	"custom-colors-support", "pkg.txt", CUSTOM_COLORS_SUPPORT, Script;
	"custom-gui-support", "pkg.txt", CUSTOM_GUI_SUPPORT, Script;
	"custom-sky-support", "pkg.txt", CUSTOM_SKY_SUPPORT, Script;
	"emissive-blocks-support", "pkg.txt", EMISSIVE_BLOCKS_SUPPORT, Script;
	"emissive-entities-support", "pkg.txt", EMISSIVE_ENTITIES_SUPPORT, Script;
	"fabric-rendering-api", "json", FABRIC_RENDERING_API, Declarative;
	"fabriclike-api", "pkg.txt", FABRICLIKE_API, Script;
	"fail", "pkg.txt", FAIL, Script;
	"hd-fonts-support", "pkg.txt", HD_FONTS_SUPPORT, Script;
	"kotlin-support", "pkg.txt", KOTLIN_SUPPORT, Script;
	"kotlin-support-forgelin", "pkg.txt", KOTLIN_SUPPORT_FORGELIN, Script;
	"kubejs-script-support", "pkg.txt", KUBEJS_SCRIPT_SUPPORT, Script;
	"natural-textures-support", "pkg.txt", NATURAL_TEXTURES_SUPPORT, Script;
	"none", "pkg.txt", NONE, Script;
	"optifine-resource-packs", "pkg.txt", OPTIFINE_RESOURCE_PACKS, Script;
	"optifine-support", "pkg.txt", OPTIFINE_SUPPORT, Script;
	"quilt-standard-libraries", "json", QUILT_STANDARD_LIBRARIES, Declarative;
	"quilted-fabric-api", "json", QUILTED_FABRIC_API, Declarative;
	"random-entities-support", "pkg.txt", RANDOM_ENTITIES_SUPPORT, Script;
	"shader-support", "pkg.txt", SHADER_SUPPORT, Script;
	"splash-screen-support", "pkg.txt", SPLASH_SCREEN_SUPPORT, Script;
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
	use mcvm_parse::{parse::lex_and_parse, routine::INSTALL_ROUTINE};
	use mcvm_pkg::parse_and_validate;

	#[test]
	fn test_core_package_parse() {
		for package in ALL_CORE_PACKAGE_IDS {
			println!("Package: {package}");
			let contents = get_core_package(package).unwrap();
			let content_type = get_core_package_content_type(package).unwrap();
			parse_and_validate(contents, content_type).unwrap();

			if let PackageContentType::Script = content_type {
				let parsed = lex_and_parse(contents).unwrap();
				if *package != "none" {
					assert!(parsed.routines.contains_key(INSTALL_ROUTINE));
				}
			}
		}
	}
}
