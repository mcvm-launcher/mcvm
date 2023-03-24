use super::lib::CmdData;

use color_print::cprintln;

pub fn help() {
	cprintln!("<i>version:</i> Print the project version");
	cprintln!("<s>Usage:</s> mcvm version");
}

pub fn run(_argc: usize, _argv: &[String], _data: &mut CmdData) -> anyhow::Result<()> {
	cprintln!("mcvm version <g>{}</g>", env!("CARGO_PKG_VERSION"));

	Ok(())
}
