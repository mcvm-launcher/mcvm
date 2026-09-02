#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use clap::Parser;
use freya::radio::use_init_radio_station;
use nitrolaunch::shared::nitro_executable::{NitroClientId, NitroExecutableRegistry};
use tokio::runtime::Builder;
use tokio::sync::broadcast;

use crate::cli::Cli;
use crate::components::dialog::tip::Tips;
use crate::components::footer::Footer;
use crate::components::global::Global;
use crate::ops::launch::{LaunchInstance, LaunchInstanceParams};
use crate::ops::task::Task;
use crate::prelude::*;

use crate::components::nav::{NavBar, router::Router};
use crate::state::{BackEvent, BackState, FrontChannel, FrontState};
use crate::util::Shared;

mod cli;
mod components;
mod data;
mod dependency;
mod icons;
mod instance_manager;
mod ops;
mod output;
mod pages;
mod prelude;
mod routing;
/// :O
mod secrets;
mod state;
mod theme;
mod util;

fn main() {
	let rt = Builder::new_multi_thread().enable_all().build().unwrap();
	let _rt = rt.enter();

	let (event_tx, mut event_rx) = broadcast::channel(100);
	let back_state = rt
		.block_on(BackState::new(event_tx, event_rx.resubscribe()))
		.unwrap();

	// CLI
	if let Ok(mut exec_registry) = NitroExecutableRegistry::open(&back_state.paths.internal) {
		let _ = exec_registry.add_this(NitroClientId::Gui);
	}
	let cli = Cli::parse();
	if let Some(instance) = cli.launch {
		let mutation = LaunchInstance {
			back_state: Captured(back_state.clone()),
		};
		let result = rt.block_on(mutation.run(&LaunchInstanceParams {
			id: instance,
			offline: false,
			quick_play: cli.quick_play.unwrap_or_default(),
		}));
		if let Err(e) = result {
			eprintln!("Failed to launch instance: {e:?}");
		}

		while let Some(ev) = rt.block_on(event_rx.recv()).ok() {
			if let BackEvent::OutputEndTask {
				task: Task::LaunchInstance(..),
				..
			} = ev
			{
				break;
			}
		}

		return;
	}

	let window = WindowConfig::new(move || app(back_state.clone(), event_rx.resubscribe()))
		.with_size(1400.0, 900.0)
		.with_title("Nitrolaunch")
		.with_app_id("Nitrolaunch");
	let config = LaunchConfig::new().with_window(window);
	#[cfg(feature = "profiler")]
	let config = config
		.with_plugin(freya::performance::PerformanceOverlayPlugin::default().with_visible(true));

	launch(config);
}

fn app(back_state: BackState, event_rx: broadcast::Receiver<BackEvent>) -> impl IntoElement {
	let station = use_init_radio_station::<(), FrontChannel>(|| ());
	use_provide_context(|| Shared::new(FrontState::new(station, event_rx)));
	use_provide_context(|| back_state);

	App
}

#[derive(PartialEq)]
struct App;

impl Component for App {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let show_sidebar = use_state(|| false);

		let router = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.child(Router::new());

		let sidebar = if *show_sidebar.read() {
			rect()
				.width(Size::px(theme.sidebar_width))
				.height(Size::fill())
				.background(theme.sidebar)
		} else {
			rect()
		};

		let view = rect()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.flex()
			.horizontal()
			.child(sidebar)
			.child(router);

		rect()
			.width(Size::fill())
			.height(Size::fill())
			.flex()
			.background(theme.bg)
			.color(theme.fg)
			.font_size(theme.font)
			.child(NavBar { show_sidebar })
			.child(Tips)
			.child(view)
			.child(Footer)
			.child(Global)
	}
}
