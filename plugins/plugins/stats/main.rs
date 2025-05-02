use std::cmp::Reverse;
use std::time::Duration;
use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use color_print::cprintln;
use itertools::Itertools;
use mcvm_core::io::{json_from_file, json_to_file};
use mcvm_plugin::api::{CustomPlugin, HookContext};
use mcvm_plugin::hooks::{Hook, Subcommand};
use mcvm_shared::util::utc_timestamp;
use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("stats", include_str!("plugin.json"))?;
	plugin.subcommand(|ctx, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "stats" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		Cli::parse_from(it);
		print_stats(ctx)?;

		Ok(())
	})?;

	plugin.on_instance_launch(|mut ctx, arg| {
		let mut stats = Stats::open(&ctx).context("Failed to open stats")?;

		// Write launch count
		stats.instances.entry(arg.id.clone()).or_default().launches += 1;
		stats.write(&ctx).context("Failed to write stats")?;

		// Track when the instance started in persistent state to get playtime
		let state = ctx
			.get_persistent_state(HashMap::<String, u64>::new())
			.context("Failed to get persistent state")?;
		let mut state: HashMap<String, u64> = serde_json::from_value(state.clone())?;
		state.insert(arg.id.clone(), utc_timestamp()?);
		ctx.set_persistent_state(state)
			.context("Failed to set persistent state")?;

		Ok(())
	})?;

	plugin.on_instance_stop(|mut ctx, arg| update_playtime(&mut ctx, &arg.id, false))?;

	plugin.while_instance_launch(|mut ctx, arg| {
		let Some(config) = ctx.get_custom_config() else {
			return Ok(());
		};
		let config: Config =
			serde_json::from_str(config).context("Failed to deserialize custom config")?;

		if !config.live_tracking {
			return Ok(());
		}

		loop {
			std::thread::sleep(Duration::from_secs(60));
			update_playtime(&mut ctx, &arg.id, true).context("Failed to update playtime")?;
		}
	})?;

	Ok(())
}

/// Update the playtime. When update_state is true, the persistent state is updated for the next update
/// so that the time delta is correct
fn update_playtime<H: Hook>(
	ctx: &mut HookContext<'_, H>,
	instance: &str,
	update_state: bool,
) -> anyhow::Result<()> {
	let state = ctx
		.get_persistent_state(HashMap::<String, u64>::new())
		.context("Failed to get persistent state")?;
	let mut state: HashMap<String, u64> = serde_json::from_value(state.clone())?;
	let Some(start_time) = state.get_mut(instance) else {
		return Ok(());
	};
	let now = utc_timestamp()?;
	let diff_minutes = (now - *start_time) / 60;

	let mut stats = Stats::open(ctx).context("Failed to open stats")?;
	stats
		.instances
		.entry(instance.to_string())
		.or_default()
		.playtime += diff_minutes;

	// Update start time so that the next update doesn't grow exponentially
	if update_state {
		*start_time = utc_timestamp()?;
		ctx.set_persistent_state(state)?;
	}

	stats.write(ctx).context("Failed to write stats")?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {}

fn print_stats(ctx: HookContext<'_, Subcommand>) -> anyhow::Result<()> {
	let stats = Stats::open(&ctx).context("Failed to open stats")?;

	#[derive(PartialEq, Eq, PartialOrd, Ord)]
	struct Ordering {
		launches: Reverse<u32>,
		playtime: Reverse<u64>,
		instance_id: String,
	}

	let total: u64 = stats
		.instances
		.values()
		.map(|x| x.calculate_playtime())
		.sum();
	let total = format_time(total);
	cprintln!("<s>Total playtime: <m!>{total}");

	for (instance, stats) in stats
		.instances
		.into_iter()
		.sorted_by_key(|(inst_id, stats)| Ordering {
			launches: Reverse(stats.launches),
			playtime: Reverse(stats.calculate_playtime()),
			instance_id: inst_id.clone(),
		}) {
		cprintln!(
			"<k!> - </><b,s>{instance}</> - Launched <m>{}</> times for a total of <m!>{}</>",
			stats.launches,
			format_time(stats.calculate_playtime())
		);
	}

	Ok(())
}

fn format_time(mut time: u64) -> String {
	let mut out = String::new();

	let hours = time / 60;
	time %= 60;

	let minutes = time;

	if hours > 0 {
		out += &format!("{hours}h ");
	}

	out += &format!("{minutes}m");

	out
}

/// The stored stats data
#[derive(Serialize, Deserialize, Clone, Default)]
struct Stats {
	/// The instances with stored stats
	instances: HashMap<String, InstanceStats>,
}

impl Stats {
	fn open<H: Hook>(ctx: &HookContext<'_, H>) -> anyhow::Result<Self> {
		let path = Self::get_path(ctx)?;
		if path.exists() {
			json_from_file(path).context("Failed to open stats file")
		} else {
			let out = Self::default();
			json_to_file(path, &out).context("Failed to write default stats to file")?;
			Ok(out)
		}
	}

	fn write<H: Hook>(self, ctx: &HookContext<'_, H>) -> anyhow::Result<()> {
		let path = Self::get_path(ctx)?;
		json_to_file(path, &self).context("Failed to write stats to file")?;
		Ok(())
	}

	fn get_path<H: Hook>(ctx: &HookContext<'_, H>) -> anyhow::Result<PathBuf> {
		Ok(ctx.get_data_dir()?.join("internal").join("stats.json"))
	}
}

/// Stats for a single instance
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct InstanceStats {
	/// The playtime of the instance in minutes
	playtime: u64,
	/// The number of times the instance has been launched
	launches: u32,
}

impl InstanceStats {
	/// Calculate playtime
	fn calculate_playtime(&self) -> u64 {
		// Due to the minutes always rounding down every time we stop, there will be a constant
		// integration error of on average half a minute every launch. Counteract this by adding back that
		// half a minute for every launch
		self.playtime + self.launches as u64 / 2
	}
}

/// Config for the plugin
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct Config {
	/// Whether to track stats while the instance is running
	live_tracking: bool,
}
