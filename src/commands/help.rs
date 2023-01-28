use color_print::cprintln;
use crate::io::files::Paths;

pub fn help_command_impl() {
	cprintln!("Mcvm: <i>A Minecraft launcher for the future");
}

pub fn help_command(_argc: usize, _argv: &[String], _paths: &Paths) {
	help_command_impl();
}
