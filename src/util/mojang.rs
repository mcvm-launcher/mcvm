use cfg_match::cfg_match;

cfg_match! {
	target_os = "linux" => {
		pub static OS_STRING: &str = "linux";
	}
	target_os = "windows" => {
		pub static OS_STRING: &str = "windows";
	}
	_ => {
		pub static OS_STRING: &str = "";
		compile_error!("Target operating system is unsupported")
	}
}

cfg_match! {
	target_arch = "x86" => {
		pub static ARCH_STRING: &str = "x86";
	}
	target_arch = "x86_64" => {
		pub static ARCH_STRING: &str = "x64";
	}
	target_arch = "arm" => {
		pub static ARCH_STRING: &str = "arm";
	}
	_ => {
		pub static ARCH_STRING: &str = "";
		compile_error!("Target architecture is unsupported")
	}
}

// For checking rule actions in Mojang json files
pub fn is_allowed(action: &str) -> bool {
	action == "allow"
}
