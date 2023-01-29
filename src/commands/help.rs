use crate::io::files::Paths;

use color_print::cprintln;

pub fn help() {
	cprintln!("<s>Usage:</s> mcvm <k!><<subcommand>> [...]</k!>");
	cprintln!();
	cprintln!("<s>Commands:");
	cprintln!("\t<i>help:</i> show this message");
	cprintln!("\t<i>profile:</i> modify profiles");
}

pub fn run(_argc: usize, _argv: &[String], _paths: &Paths)
-> Result<(), Box<dyn std::error::Error>> {
	cprintln!("Mcvm: <i>A Minecraft launcher for the future");
	help();
	Ok(())
}
