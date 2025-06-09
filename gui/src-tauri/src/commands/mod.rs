use anyhow::Context;
use mcvm::config::Config;
use mcvm::io::paths::Paths;
use mcvm::plugin::PluginManager;
use mcvm::shared::output::MCVMOutput;
use std::fmt::Debug;

pub mod instance;
pub mod launch;
pub mod misc;
pub mod package;
pub mod plugin;
pub mod user;

fn load_config(paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<Config> {
	let plugins = PluginManager::load(paths, o).context("Failed to load plugin manager")?;
	Config::load(
		&Config::get_path(paths),
		plugins,
		true,
		paths,
		crate::get_ms_client_id(),
		o,
	)
	.context("Failed to load config")
}

/// Error formatting for results
fn fmt_err<T, E: Debug>(r: Result<T, E>) -> Result<T, String> {
	r.map_err(|x| format!("{x:?}"))
}
