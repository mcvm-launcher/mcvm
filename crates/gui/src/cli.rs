use nitrolaunch::core::QuickPlayType;

#[derive(clap::Parser)]
pub struct Cli {
	#[arg(long)]
	pub launch: Option<String>,
	#[arg(long)]
	pub account: Option<String>,
	#[arg(long)]
	pub quick_play: Option<QuickPlayType>,
}
