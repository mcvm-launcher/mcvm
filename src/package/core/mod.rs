static ANIMATED_TEXTURES_SUPPORT: &str = include_str!("animated-textures-support.pkg.txt");
static CEM_SUPPORT: &str = include_str!("cem-support.pkg.txt");
static CIT_SUPPORT: &str = include_str!("cit-support.pkg.txt");
static CTM_SUPPORT: &str = include_str!("ctm-support.pkg.txt");
static CUSTOM_COLORS_SUPPORT: &str = include_str!("custom-colors-support.pkg.txt");
static CUSTOM_GUI_SUPPORT: &str = include_str!("custom-gui-support.pkg.txt");
static CUSTOM_SKY_SUPPORT: &str = include_str!("custom-sky-support.pkg.txt");
static EMISSIVE_BLOCKS_SUPPORT: &str = include_str!("emissive-blocks-support.pkg.txt");
static EMISSIVE_ENTITIES_SUPPORT: &str = include_str!("emissive-entities-support.pkg.txt");
static FABRIC_RENDERING_API: &str = include_str!("fabric-rendering-api.pkg.txt");
static FABRICLIKE_API: &str = include_str!("fabriclike-api.pkg.txt");
static KOTLIN_SUPPORT: &str = include_str!("kotlin-support.pkg.txt");
static OPTIFINE_RESOURCE_PACKS: &str = include_str!("optifine-resource-packs.pkg.txt");
static RANDOM_ENTITIES_SUPPORT: &str = include_str!("random-entities-support.pkg.txt");
static SHADER_SUPPORT: &str = include_str!("shader-support.pkg.txt");
static SPLASH_SCREEN_SUPPORT: &str = include_str!("splash-screen-support.pkg.txt");

static ALL_CORE_PACKAGES: [&str; 16] = [
	ANIMATED_TEXTURES_SUPPORT,
	CEM_SUPPORT,
	CIT_SUPPORT,
	CTM_SUPPORT,
	CUSTOM_COLORS_SUPPORT,
	CUSTOM_GUI_SUPPORT,
	CUSTOM_SKY_SUPPORT,
	EMISSIVE_BLOCKS_SUPPORT,
	EMISSIVE_ENTITIES_SUPPORT,
	FABRIC_RENDERING_API,
	FABRICLIKE_API,
	KOTLIN_SUPPORT,
	OPTIFINE_RESOURCE_PACKS,
	RANDOM_ENTITIES_SUPPORT,
	SHADER_SUPPORT,
	SPLASH_SCREEN_SUPPORT,
];

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
		"kotlin-support" => Some(KOTLIN_SUPPORT),
		"optifine-resource-packs" => Some(OPTIFINE_RESOURCE_PACKS),
		"random-entities-support" => Some(RANDOM_ENTITIES_SUPPORT),
		"shader-support" => Some(SHADER_SUPPORT),
		"splash-screen-support" => Some(SPLASH_SCREEN_SUPPORT),
		_ => None,
	}
}

pub fn is_core_package(package: &str) -> bool {
	ALL_CORE_PACKAGES.contains(&package)
}

#[cfg(test)]
mod tests {
	use super::*;
	use mcvm_parse::parse::lex_and_parse;

	#[test]
	fn test_core_package_parse() {
		for package in ALL_CORE_PACKAGES {
			lex_and_parse(package).unwrap();
		}
	}
}
