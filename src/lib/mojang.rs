// TODO: Will add variants for other OS's later
pub static OS_STRING: &'static str = "linux";

pub static ARCH_STRING: &'static str = "x64";

// For checking rule actions in Mojang json files
pub fn is_allowed(action: &str) -> bool {
	action == "allow"
}
