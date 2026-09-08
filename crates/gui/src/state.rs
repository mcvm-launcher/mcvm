use std::{
	collections::HashMap,
	rc::Rc,
	sync::{Arc, OnceLock},
};

use anyhow::Context;
use freya::{
	prelude::use_consume,
	radio::{Radio, RadioChannel, RadioStation, use_radio},
};
use freya_core::{
	integration::{State, WritableUtils},
	lifecycle::{effect::use_side_effect, state::use_state},
};
use nitrolaunch::{
	config::Config,
	config_crate::{ConfigDeser, ConfigKind},
	core::net::game_files::version_manifest::VersionManifestAndList,
	instance::update::manager::UpdateSettings,
	io::{logging::Logger, paths::Paths},
	pkg_crate::repo::RepoMetadata,
	plugin::PluginManager,
	plugin_crate::hook::hooks::{self, AddCustomPackageRepositories, AddThemes},
	shared::{
		UpdateDepth,
		manual_files::ManualFile,
		output::{Message, MessageContents, MessageLevel, NitroOutput, NoOp},
		pkg::PackageDiff,
		versions::{MinecraftLatestVersion, MinecraftVersionDeser},
	},
};
use reqwest::Client;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{
	components::{
		dialog::{tip::Tip, toast::Toast},
		footer::FooterItem,
		instance::transfer::InstanceTransferMode,
	},
	data::LauncherData,
	dependency::BackDependency,
	instance_manager::RunningInstanceManager,
	ops::task::{Task, TaskManager},
	output::{LauncherOutput, OutputInner, SerializableResolutionError},
	pages::{config::ConfiguredItem, settings},
	routing::{Navigator, Page},
	secrets::get_ms_client_id,
	theme::Theme,
	util::{PtrEq, Shared},
};

/// Global state for frontend / UI related things. Only usable on the freya thread.
#[derive(Clone)]
pub struct FrontState {
	theme: Arc<Theme>,
	navigator: Navigator,
	radio: RadioStation<(), FrontChannel>,
	footer: FooterItem,
	modal: Option<ModalType>,
	toasts: Rc<[Toast]>,
	toast_id_counter: u32,
	tip: Option<Tip>,
	event_rx: Rc<broadcast::Receiver<BackEvent>>,
}

/// Different "channels" for listening to changes in parts of the global frontend state
#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
pub enum FrontChannel {
	/// Changes to the route
	Route,
	/// Changes to the footer item
	FooterItem,
	/// Changes to toasts
	Toast,
	/// Changes to the tip
	Tip,
	/// Changes to the visible modal
	Modal,
	/// Changes to launcher data
	Data,
	/// Changes to the theme
	Theme,
	/// Changes to the configured theme, which then updates the theme
	ThemeConfig,
	/// Changes to the zoom level
	Zoom,
}

impl RadioChannel<()> for FrontChannel {}

impl FrontState {
	pub fn new(
		radio: RadioStation<(), FrontChannel>,
		event_rx: broadcast::Receiver<BackEvent>,
	) -> Self {
		Self {
			theme: Arc::new(Theme::dark()),
			navigator: Navigator::new(),
			radio,
			footer: FooterItem::None,
			modal: None,
			toasts: Rc::default(),
			toast_id_counter: 0,
			tip: None,
			event_rx: Rc::new(event_rx),
		}
	}

	/// Subscribe to changes in the front state on the given channel for this component, re-rendering when it updates
	pub fn subscribe(&self, channel: FrontChannel) {
		use_radio(channel).read();
	}

	pub fn subscribe_events(&self) -> broadcast::Receiver<BackEvent> {
		self.event_rx.resubscribe()
	}

	pub fn invalidate(&self, channel: FrontChannel) {
		self.radio.clone().write_channel(channel);
	}

	pub fn theme(&self) -> Arc<Theme> {
		self.theme.clone()
	}

	pub fn set_theme(&mut self, theme: Theme) {
		self.theme = Arc::new(theme);
		self.invalidate(FrontChannel::Theme);
	}

	pub fn route(&self) -> &Page {
		self.navigator.route()
	}

	pub fn navigate(&mut self, route: Page) {
		let prev_route = self.navigator.route().clone();
		self.navigator.navigate(route);
		self.check_route_change(prev_route);
		self.invalidate(FrontChannel::Route);
		self.set_modal(None);
	}

	pub fn forward(&mut self) {
		let prev_route = self.navigator.route().clone();
		self.navigator.forward();
		self.check_route_change(prev_route);
		self.invalidate(FrontChannel::Route);
	}

	pub fn back(&mut self) {
		let prev_route = self.navigator.route().clone();
		self.navigator.back();
		self.check_route_change(prev_route);
		self.invalidate(FrontChannel::Route);
	}

	pub fn can_go_forward(&self) -> bool {
		self.navigator.can_go_forward()
	}

	pub fn can_go_back(&self) -> bool {
		self.navigator.can_go_back()
	}

	fn check_route_change(&mut self, prev_route: Page) {
		if self.navigator.route() != &prev_route {
			self.footer = FooterItem::None;
			self.invalidate(FrontChannel::FooterItem);
		}
	}

	pub fn set_footer(&mut self, item: FooterItem) {
		self.footer = item;
		self.invalidate(FrontChannel::FooterItem);
	}

	pub fn footer(&self) -> &FooterItem {
		&self.footer
	}

	pub fn set_modal(&mut self, modal: Option<ModalType>) {
		self.modal = modal;
		self.invalidate(FrontChannel::Modal);
	}

	pub fn modal(&self) -> Option<&ModalType> {
		self.modal.as_ref()
	}

	pub fn toast(&mut self, mut toast: Toast) {
		toast.set_id(self.toast_id_counter);
		self.toast_id_counter += 1;

		self.toasts = self
			.toasts
			.iter()
			.cloned()
			.chain(std::iter::once(toast))
			.collect();
		self.invalidate(FrontChannel::Toast);
	}

	pub fn toasts(&self) -> &[Toast] {
		&self.toasts
	}

	pub fn remove_toast(&mut self, id: u32) {
		self.toasts = self
			.toasts
			.iter()
			.filter(|x| x.id() != id)
			.cloned()
			.collect();
		self.invalidate(FrontChannel::Toast);
	}

	pub fn tip(&self) -> Option<&Tip> {
		self.tip.as_ref()
	}

	pub fn set_tip(&mut self, tip: Option<Tip>) {
		self.tip = tip;
		self.invalidate(FrontChannel::Tip);
	}
}

/// Gives access to front state
pub fn use_front_state() -> Shared<FrontState> {
	use_consume()
}

#[derive(Clone, PartialEq)]
pub enum ModalType {
	Configuration(ConfiguredItem),
	Settings(settings::Tab),
	DeleteInstance(String),
	DeleteTemplate(String),
	PackageDiffs(Arc<[PackageDiff]>),
	ManualFiles(PtrEq<[ManualFile]>),
	MicrosoftAuth { url: String, device_code: String },
	Transfer(InstanceTransferMode, Option<String>),
	Migrate,
	Onboarding,
	CustomPopup(PtrEq<hooks::Popup>),
}

/// Global state for Nitrolaunch-related things. Thread-safe, can be passed to tokio tasks.
#[derive(Clone)]
pub struct BackState {
	pub event_tx: broadcast::Sender<BackEvent>,
	pub paths: Arc<Paths>,
	pub client: Client,
	pub plugins: PluginManager,
	pub running_instances: RunningInstanceManager,
	output_inner: OutputInner,
	task_manager: Arc<Mutex<TaskManager>>,
	cached_info: Arc<CachedInfo>,
}

impl BackState {
	pub async fn new(
		event_tx: broadcast::Sender<BackEvent>,
		event_rx: broadcast::Receiver<BackEvent>,
	) -> anyhow::Result<Self> {
		let event_rx = Arc::new(event_rx);
		let paths = Arc::new(Paths::new().await?);
		let plugins = PluginManager::load(&paths, &mut NoOp).await?;

		let running_instances = RunningInstanceManager::new(&paths, event_tx.clone())
			.context("Failed to create running instance manager")?;

		tokio::spawn(running_instances.clone().get_run_task());

		let (logger_tx, mut logger_rx) = mpsc::channel::<Message>(25);
		let mut logger = Logger::new(&paths, "gui").context("Failed to set up logger")?;
		tokio::spawn(async move {
			while let Some(message) = logger_rx.recv().await {
				println!("{}", message.contents.clone().default_format());
				let _ = logger.log_message(message.contents, message.level);
			}
		});

		let task_manager = Arc::new(Mutex::new(TaskManager::new(
			event_tx.clone(),
			logger_tx.clone(),
		)));
		tokio::spawn(TaskManager::get_run_task(task_manager.clone()));

		let output_inner = OutputInner {
			event_tx: event_tx.clone(),
			event_rx: event_rx.clone(),
			password_prompt: Arc::new(Mutex::new(None)),
			yes_no_prompt: Arc::new(Mutex::new(None)),
			passkeys: Arc::new(Mutex::new(HashMap::new())),
			logger: logger_tx,
		};

		let mut o = LauncherOutput::new(&output_inner);
		let client = Client::new();
		let cached_info = CachedInfo::new(&paths, &plugins, &mut o).await;

		Ok(Self {
			output_inner,
			task_manager,
			event_tx,
			paths,
			plugins,
			client,
			running_instances,
			cached_info: Arc::new(cached_info),
		})
	}

	pub async fn config(&self) -> anyhow::Result<Config> {
		self.config_impl(NoOp).await
	}

	pub async fn config_with_warnings(&self) -> anyhow::Result<Config> {
		let output = self.output();
		self.config_impl(output).await
	}

	async fn config_impl(&self, mut o: impl NitroOutput + 'static) -> anyhow::Result<Config> {
		let paths = self.paths.clone();
		let plugins = self.plugins.clone();

		tokio::spawn(async move {
			Config::load(
				&Config::get_path(&paths),
				plugins,
				false,
				&paths,
				get_ms_client_id(),
				&mut o,
			)
			.await
		})
		.await?
	}

	pub async fn raw_config(&self) -> anyhow::Result<ConfigDeser> {
		let paths = self.paths.clone();

		tokio::spawn(async move { Config::open(&Config::get_path(&paths)) }).await?
	}

	pub fn data(&self) -> LauncherData {
		LauncherData::open(&self.paths).unwrap_or_else(|e| {
			self.output().display(MessageContents::Error(format!(
				"Failed to open launcher data: {e}"
			)));
			LauncherData::default()
		})
	}

	pub fn output(&self) -> LauncherOutput {
		LauncherOutput::new(&self.output_inner)
	}

	pub fn log(&self, message: impl Into<MessageContents>) {
		let _ = self.output_inner.logger.try_send(Message {
			contents: message.into(),
			level: MessageLevel::Debug,
		});
	}

	pub fn register_task(
		&self,
		task: Task,
		task_handle: tokio::task::JoinHandle<anyhow::Result<()>>,
	) {
		let manager = self.task_manager.clone();
		tokio::spawn(async move { manager.lock().await.register_task(task, task_handle) });
	}

	pub async fn kill_task(&self, task: &Task) {
		let mut manager = self.task_manager.lock().await;
		manager.kill(task);
	}

	/// Invalidates a dependency from a tokio task which can't access freya context
	pub fn invalidate(&self, dependency: BackDependency) {
		let _ = self.event_tx.send(BackEvent::Invalidate(dependency));
	}

	/// Invalidates installed plugins
	pub async fn invalidate_plugins(&self) {
		let mut o = self.output();
		self.invalidate(BackDependency::Plugins);
		if let Err(e) = self.plugins.reload(&self.paths, &mut o).await {
			o.display(MessageContents::Error(format!(
				"Failed to reload plugins: {e:?}"
			)));
		}
	}

	pub fn repos(&self) -> &HashMap<String, RepoMetadata> {
		&self.cached_info.repos
	}

	pub fn themes(&self) -> &[nitrolaunch::plugin_crate::hook::hooks::Theme] {
		&self.cached_info.themes
	}

	pub async fn versions(&self) -> anyhow::Result<&Arc<VersionManifestAndList>> {
		self.cached_info
			.versions(
				&self.config().await?,
				&self.client,
				&self.plugins,
				&self.paths,
				&mut self.output(),
			)
			.await
	}

	/// Converts Latest and LatestSnapshot versions to their actual final version strings
	pub async fn canonicalize_version(
		&self,
		id: Option<&str>,
		ty: ConfigKind,
		version: &MinecraftVersionDeser,
	) -> Option<String> {
		if let MinecraftVersionDeser::Version(version) = &version {
			return Some(version.to_string());
		}

		let mut config = self.config().await.ok()?;

		if let Some(id) = id
			&& ty == ConfigKind::Instance
		{
			let Some(instance) = config.instances.get_mut(id) else {
				let _ = self.output_inner.logger.try_send(Message {
					contents: "Canonicalize: Instance does not exist".into(),
					level: MessageLevel::Debug,
				});
				return None;
			};

			let inst_lock = instance.get_lockfile(&self.paths).ok()?;

			if let Some(version) = inst_lock.get_minecraft_version() {
				return Some(version.clone());
			}
		}

		let versions = self.versions().await.ok()?;
		let Some(latest) = &versions.manifest.latest else {
			let _ = self.output_inner.logger.try_send(Message {
				contents: "Canonicalize: No latest versions available".into(),
				level: MessageLevel::Debug,
			});
			return None;
		};

		let version = match version {
			MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release) => {
				latest.release.to_string()
			}
			MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot) => {
				latest.snapshot.to_string()
			}
			MinecraftVersionDeser::Version(..) => unreachable!(),
		};

		Some(version)
	}
}

/// Events sent from the backend
#[allow(dead_code)]
#[derive(Clone)]
pub enum BackEvent {
	Invalidate(BackDependency),
	SuccessToast(String),
	ErrorToast(String, Option<String>),
	OutputMessage {
		message: MessageContents,
		task: Option<Task>,
	},
	OutputStartTask(Task),
	OutputEndTask {
		task: Task,
		success: bool,
	},
	OutputEndProcess(Option<Task>),
	OutputEndSection(Option<Task>),
	OutputResolutionError {
		error: Arc<SerializableResolutionError>,
		instance_id: String,
	},
	UpdateRunningInstances,
	ShowAuthPrompt {
		url: String,
		device_code: String,
	},
	CloseAuthPrompt,
	ShowYesNoPrompt {
		message: String,
	},
	ConfirmYesNoPrompt {
		yes: bool,
	},
	ShowPasskeyPrompt,
	ShowPackageDiffsPrompt {
		diffs: Arc<[PackageDiff]>,
	},
	ShowManualFilesPrompt {
		files: PtrEq<[ManualFile]>,
	},
	InvalidateData,
}

/// Information from plugins and such that is fetched on startup or reload once and then used
struct CachedInfo {
	repos: HashMap<String, RepoMetadata>,
	themes: Vec<nitrolaunch::plugin_crate::hook::hooks::Theme>,
	versions: OnceLock<Arc<VersionManifestAndList>>,
}

impl CachedInfo {
	async fn new(paths: &Paths, plugins: &PluginManager, o: &mut impl NitroOutput) -> Self {
		let repos = if let Ok(repos) = plugins
			.call_hook(AddCustomPackageRepositories, &(), paths, o)
			.await
		{
			if let Ok(repos) = repos.flatten_all_results(o).await {
				repos
			} else {
				Vec::new()
			}
		} else {
			Vec::new()
		};
		let repos = repos.into_iter().map(|x| (x.id, x.metadata)).collect();

		let themes = if let Ok(themes) = plugins.call_hook(AddThemes, &(), paths, o).await {
			if let Ok(themes) = themes.flatten_all_results(o).await {
				themes
			} else {
				Vec::new()
			}
		} else {
			Vec::new()
		};

		Self {
			repos,
			themes,
			versions: OnceLock::new(),
		}
	}

	async fn versions(
		&self,
		config: &Config,
		client: &Client,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<&Arc<VersionManifestAndList>> {
		if self.versions.get().is_none() {
			let core = config
				.get_core(
					None,
					&UpdateSettings {
						depth: UpdateDepth::Shallow,
						offline_auth: false,
					},
					client,
					plugins,
					paths,
					o,
				)
				.await?;
			let versions = core
				.get_version_manifest(None, UpdateDepth::Shallow, o)
				.await
				.cloned()?;

			let _ = self.versions.set(versions);
		}
		self.versions
			.get()
			.ok_or_else(|| anyhow::anyhow!("Versions not initialized"))
	}
}

pub fn use_launcher_data() -> LauncherDataHook {
	let radio = use_radio(FrontChannel::Data);
	let back_state = use_consume::<BackState>();

	let state = use_state(|| back_state.data());

	let back_state2 = back_state.clone();
	let mut state2 = state;
	let radio2 = radio;
	use_side_effect(move || {
		radio2.read();
		state2.set_if_modified(back_state2.data());
	});

	LauncherDataHook {
		data: state,
		radio,
		back_state,
	}
}

#[derive(Clone)]
pub struct LauncherDataHook {
	pub data: State<LauncherData>,
	radio: Radio<(), FrontChannel>,
	back_state: BackState,
}

impl LauncherDataHook {
	pub fn save(&self) {
		if let Err(e) = self.data.read().write(&self.back_state.paths) {
			self.back_state
				.output()
				.display(MessageContents::Error(format!(
					"Failed to write launcher data: {e}"
				)));
		} else {
			self.radio.clone().write();
		}
	}
}
