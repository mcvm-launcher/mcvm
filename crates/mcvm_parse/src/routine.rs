pub static METADATA_ROUTINE: &str = "meta";
pub static PROPERTIES_ROUTINE: &str = "properties";
pub static INSTALL_ROUTINE: &str = "install";
pub static UNINSTALL_ROUTINE: &str = "uninstall";

/// Returns if a routine name is reserved for use by mcvm
pub fn is_reserved(routine: &str) -> bool {
	routine == METADATA_ROUTINE
		|| routine == PROPERTIES_ROUTINE
		|| routine == INSTALL_ROUTINE
		|| routine == UNINSTALL_ROUTINE
}
