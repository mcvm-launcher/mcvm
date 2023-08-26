pub static METADATA_ROUTINE: &str = "meta";
pub static PROPERTIES_ROUTINE: &str = "properties";
pub static INSTALL_ROUTINE: &str = "install";
pub static UNINSTALL_ROUTINE: &str = "uninstall";

/// The list of reserved routines
pub static RESERVED_ROUTINES: [&str; 4] = [
	METADATA_ROUTINE,
	PROPERTIES_ROUTINE,
	INSTALL_ROUTINE,
	UNINSTALL_ROUTINE,
];

/// Returns if a routine name is reserved for use by mcvm
pub fn is_reserved(routine: &str) -> bool {
	RESERVED_ROUTINES.contains(&routine)
}

/// Returns if a routine can call other routines
pub fn can_call_routines(routine: &str) -> bool {
	routine != METADATA_ROUTINE && routine != PROPERTIES_ROUTINE
}
