use color_print::cprintln;
use crate::io::files::Paths;

pub fn help_command_impl() {
	cprintln!("Mcvm: <i>A Minecraft launcher for the future");
}

pub fn help_command(_argc: u8, _argv: &[String], paths: Paths) {
	println!("{}", paths.internal.to_str().unwrap());
	help_command_impl();
}
