use crate::io::files::Paths;

use color_print::cprintln;

static UPDATE_HELP: &'static str = "Update the packages and instances of a profile";

pub fn help() {
	cprintln!("Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<command>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Commands:");
	cprintln!("\t <i>update:</i> {}", UPDATE_HELP);
}

pub fn run(argc: usize, _argv: &[String], _paths: &Paths)
-> Result<(), Box<dyn std::error::Error>> {
	if argc == 0 {
		help();
		return Ok(());
	}

	Ok(())
}
