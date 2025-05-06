use crate::output::LauncherOutput;
use anyhow::Context;
use mcvm::config::Config;
use mcvm::io::paths::Paths;
use mcvm::plugin::PluginManager;
use std::fmt::Debug;

pub mod instance;
pub mod launch;

fn load_config(paths: &Paths, o: &mut LauncherOutput) -> anyhow::Result<Config> {
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
