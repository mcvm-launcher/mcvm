use std::{
	ops::Deref,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
};

use anyhow::{Context, bail};
use crossterm::event::{KeyCode, KeyEvent};
use dashmap::DashMap;
use image::DynamicImage;
use itertools::Itertools;
use nitrolaunch::{
	config::{
		Config,
		modifications::{ConfigModification, apply_modifications_and_write},
	},
	config_crate::{
		instance::is_valid_instance_id,
		package::{PackageConfigDeser, add_or_update_package_config},
	},
	core::{NitroCore, util::versions::MinecraftVersion},
	instance::update::manager::UpdateSettings,
	io::paths::Paths,
	pkg::search::{PackageMultiSearchResults, PackageSearchSession},
	pkg_crate::{PackageMetaAndProps, declarative::DeclarativeAddonVersion},
	plugin_crate::hook::hooks::{
		AddCustomPackageRepositories, AddCustomPackageRepositoriesResult, AddSupportedLoaders,
	},
	shared::{
		Side, UpdateDepth,
		id::{InstanceID, TemplateID},
		loaders::Loader,
		output::NoOp,
		pkg::{ArcPkgReq, PackageCategory, PackageKind, PackageSearchParameters, PackageStability},
		util::to_string_json,
		versions::{VersionPattern, parse_single_versioned_string},
	},
};
use ratatui::{
	DefaultTerminal, Frame,
	layout::{Constraint, HorizontalAlignment, Layout, Margin, Rect},
	style::Style,
	symbols,
	text::{Line, Span},
	widgets::{
		Block, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
	},
};
use ratatui_image::{Image, Resize, picker::Picker};
use ratatui_textarea::TextArea;
use reqwest::Client;
use tokio::{
	sync::mpsc::{Receiver, Sender},
	task::JoinHandle,
};

use crate::{
	commands::{CmdData, modpack::install_into_config},
	image_cache::{ImageCache, crop_image_to_ratio},
	output::fit_message_width,
	prompt::get_key,
};

const PAGE_SIZE: u8 = 35;

pub async fn run(
	mut data: CmdData<'_>,
	instance: Option<String>,
	template: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(false).await?;

	let repos = data
		.config
		.get_mut()
		.plugins
		.call_hook(AddCustomPackageRepositories, &(), &data.paths, &mut NoOp)
		.await?
		.flatten_all_results(&mut NoOp)
		.await?;

	let loaders = data
		.config
		.get_mut()
		.plugins
		.call_hook(AddSupportedLoaders, &(), &data.paths, &mut NoOp)
		.await?
		.flatten_all_results(&mut NoOp)
		.await?;

	let client = Client::new();
	let core = data
		.config
		.get()
		.get_core(
			None,
			&UpdateSettings {
				depth: UpdateDepth::Shallow,
				offline_auth: false,
			},
			&client,
			&data.config.get().plugins,
			&data.paths,
			&mut NoOp,
		)
		.await?;
	let versions = core
		.get_version_manifest(None, UpdateDepth::Shallow, &mut NoOp)
		.await?
		.get_releases();

	let instances: Vec<_> = data.config.get().instances.keys().cloned().collect();
	let templates: Vec<_> = data.config.get().templates.keys().cloned().collect();

	let filters = create_filters(instance, template, data.config.get(), &core)
		.await
		.context("Failed to create initial filters")?;

	let state = State::new(
		data.config.take().unwrap(),
		data.paths.clone(),
		filters,
		repos,
		versions,
		loaders,
		instances,
		templates,
	)?;

	ratatui::run(move |terminal| renderer(terminal, state)).context("Failed to run app")
}

/// Create initial filters
async fn create_filters(
	instance: Option<String>,
	template: Option<String>,
	config: &Config,
	core: &NitroCore,
) -> anyhow::Result<PackageSearchParameters> {
	let mut params = PackageSearchParameters {
		count: PAGE_SIZE,
		skip: 0,
		..Default::default()
	};

	if let Some(instance) = instance {
		let Some(instance) = config.instances.get(&InstanceID::from(instance)) else {
			bail!("Instance does not exist");
		};

		if instance.loader() != &Loader::Vanilla {
			params.loaders = vec![instance.loader().clone()];
		}
		let version = core.resolve_version(instance.version()).await?;
		params.minecraft_versions = vec![version.to_string()];
	} else if let Some(template) = template {
		let Some(template) = config.templates.get(&TemplateID::from(template)) else {
			bail!("Template does not exist");
		};

		if let Some(loader) = &template.instance.loader {
			let (loader, _) = parse_single_versioned_string(loader);
			let loader = Loader::parse_from_str(loader);
			if loader != Loader::Vanilla {
				params.loaders = vec![loader];
			}
		}
		if let Some(version) = &template.instance.version {
			let version = MinecraftVersion::from_deser(version);
			let version = core.resolve_version(&version).await?;
			params.minecraft_versions = vec![version.to_string()];
		}
	}

	Ok(params)
}

/// State for the application
struct State<'a> {
	/// Handle for worker thread
	worker: JoinHandle<()>,
	/// Current state of the worker thread
	worker_state: WorkerState,
	/// Receiver for worker state updates
	worker_state_rx: Receiver<WorkerState>,
	/// Sender for worker thread tasks
	task_tx: Sender<Task>,
	/// Receiver for search results
	results_rx: Receiver<PackageMultiSearchResults>,
	/// Package previews map
	previews: Arc<DashMap<ArcPkgReq, PackageMetaAndProps>>,
	/// Finalized search results
	results: PackageMultiSearchResults,
	/// Receiver for package info
	package_info_rx: Receiver<PackageInfo>,
	/// Finalized package info
	package_info: Option<PackageInfo>,
	/// Image cache
	image_cache: ImageCache,
	/// Set to true when a new image was loaded and we should re-render
	image_available: Arc<AtomicBool>,
	/// Current focus state
	focus: FocusState,
	/// Search parameters
	search_params: SearchParams,
	/// Search bar
	search: TextArea<'a>,
	/// Popup list state
	popup_list_state: ListState,
	/// List of packages
	package_list: List<'a>,
	/// List state for package list
	package_list_state: ListState,
	/// Last selected package
	last_selected_package: Option<ArcPkgReq>,
	/// Available repositories
	repositories: Vec<AddCustomPackageRepositoriesResult>,
	/// Available Minecraft versions
	versions: Vec<String>,
	/// Available loaders
	loaders: Vec<Loader>,
	/// Available instances
	instances: Vec<InstanceID>,
	/// Available templates
	templates: Vec<TemplateID>,
	/// Current scroll of preview pane body
	preview_scroll: u16,
	/// Max height scroll of preview pane body
	preview_scroll_height: u16,
	/// Current tab of preview pane
	preview_tab: PreviewTab,
	/// State for install prompt
	install_prompt: InstallPromptState<'a>,
}

impl<'a> State<'a> {
	/// Initialize state with widgets and worker thread
	fn new(
		config: Config,
		paths: Paths,
		filters: PackageSearchParameters,
		repositories: Vec<AddCustomPackageRepositoriesResult>,
		versions: Vec<String>,
		loaders: Vec<Loader>,
		instances: Vec<InstanceID>,
		templates: Vec<TemplateID>,
	) -> anyhow::Result<Self> {
		// Get info

		// Setup worker
		let session = PackageSearchSession::new(PAGE_SIZE);
		let (state_tx, state_rx) = tokio::sync::mpsc::channel(4);
		let (task_tx, task_rx) = tokio::sync::mpsc::channel(5);
		let (results_tx, results_rx) = tokio::sync::mpsc::channel(2);
		let (package_info_tx, package_info_rx) = tokio::sync::mpsc::channel(2);
		let handle = tokio::spawn(worker_thread(
			config,
			paths,
			session.clone(),
			state_tx,
			task_rx,
			results_tx,
			package_info_tx,
		));

		// Search bar
		let mut search = TextArea::new(Vec::new());
		search.set_style(Style::new().white());
		search.set_placeholder_text("Enter search...");
		let search_block = rounded_block().title("Search");
		search.set_block(search_block);

		// Package list
		let package_list = List::default()
			.highlight_style(Style::new().green())
			.highlight_symbol(">");
		let mut package_list_state = ListState::default();
		package_list_state.select_first();

		let mut out = Self {
			worker: handle,
			worker_state: WorkerState::Idle,
			worker_state_rx: state_rx,
			task_tx,
			results_rx,
			package_info_rx,
			package_info: None,
			image_cache: ImageCache::new(Client::new()),
			image_available: Arc::new(AtomicBool::new(false)),
			results: PackageMultiSearchResults::default(),
			previews: session.previews().clone(),
			search_params: SearchParams {
				inner: filters,
				repo: None,
			},
			popup_list_state: ListState::default(),
			search,
			package_list,
			package_list_state,
			last_selected_package: None,
			repositories,
			versions,
			loaders,
			instances,
			templates,
			focus: FocusState::None,
			preview_scroll: 0,
			preview_scroll_height: 0,
			preview_tab: PreviewTab::Description,
			install_prompt: InstallPromptState {
				location: InstallLocation::Instance,
				list_state: ListState::default(),
				new_id_state: TextArea::default(),
				new_id_focused: false,
			},
		};

		out.search();

		Ok(out)
	}

	/// Gets the currently selected package
	fn get_selected_package(&self) -> Option<ArcPkgReq> {
		let pos = self.package_list_state.selected()?;
		let pkg = self.results.results.get(pos)?;

		Some(pkg.clone())
	}

	/// Gets info for the currently selected repository
	fn get_selected_repo_info(&self) -> Option<&AddCustomPackageRepositoriesResult> {
		self.repositories
			.iter()
			.find(|x| Some(&x.id) == self.search_params.repo.as_ref())
	}

	/// Gets the request for the package that will be installed
	fn get_installed_package(&self) -> Option<ArcPkgReq> {
		let info = self.package_info.as_ref()?;
		let req = self.get_selected_package()?;

		let content_version = if self.preview_tab == PreviewTab::Versions {
			if let Some(version) = info.versions.get(self.preview_scroll as usize) {
				if let Some(content_versions) = &version.conditional_properties.content_versions {
					content_versions.first()
				} else {
					None
				}
			} else {
				None
			}
		} else {
			None
		};

		let req = req.with_slug(info.preview.meta.slug.clone());
		let req = if let Some(content_version) = content_version {
			req.with_content_version(VersionPattern::Single(content_version.clone()))
		} else {
			req
		};

		Some(req.arc())
	}

	fn get_available_package_types(&self) -> Vec<PackageKind> {
		if let Some(repo) = self.get_selected_repo_info() {
			repo.metadata.package_types.clone()
		} else {
			self.repositories
				.iter()
				.flat_map(|x| x.metadata.package_types.clone())
				.unique()
				.collect()
		}
	}

	fn get_available_package_categories(&self) -> Vec<PackageCategory> {
		if let Some(repo) = self.get_selected_repo_info() {
			repo.metadata.package_categories.clone()
		} else {
			self.repositories
				.iter()
				.flat_map(|x| x.metadata.package_categories.clone())
				.unique()
				.collect()
		}
	}

	/// Returns focus to the element above
	fn focus_out(&mut self) {
		match self.focus {
			FocusState::InstallPrompt => {
				if self.install_prompt.new_id_focused {
					self.install_prompt.new_id_focused = false;
				} else {
					self.focus_preview()
				}
			}
			_ => self.focus_none(),
		}
	}

	/// Returns focus to the default
	fn focus_none(&mut self) {
		// Search when a multi-select closes
		if let FocusState::Popup(Popup::Version | Popup::Loader | Popup::Category) = self.focus {
			self.search();
		}
		self.focus = FocusState::None;
	}

	/// Focuses a popup
	fn focus_popup(&mut self, popup: Popup) {
		self.popup_list_state.select_first();
		self.focus = FocusState::Popup(popup);
	}

	/// Focuses the preview
	fn focus_preview(&mut self) {
		self.select_package();
		self.focus = FocusState::Preview;
	}

	/// Focuses the install prompt
	fn focus_install(&mut self) {
		self.install_prompt.list_state.select_first();
		self.install_prompt.location = InstallLocation::Instance;
		self.focus = FocusState::InstallPrompt;
	}

	/// Focuses a different preview tab
	fn focus_preview_tab(&mut self, tab: PreviewTab) {
		self.preview_tab = tab;
		self.preview_scroll = 0;
	}

	/// Resets package filters
	fn reset_filters(&mut self) {
		self.search_params.inner.types = Vec::new();
		self.search_params.inner.minecraft_versions = Vec::new();
		self.search_params.inner.loaders = Vec::new();
		self.search_params.inner.categories = Vec::new();
		self.search();
	}

	/// Sends a request to search for packages given the current parameters
	fn search(&mut self) {
		let _ = self
			.task_tx
			.try_send(Task::FetchPackages(self.search_params.clone()));
	}

	/// Previews the currently highlighted package in the list
	fn preview_package(&mut self) {
		let Some(req) = self.get_selected_package() else {
			return;
		};

		if let Some(preview) = self.previews.get(&req) {
			self.package_info = Some(PackageInfo {
				req,
				preview: preview.clone(),
				versions: Vec::new(),
			});
			self.preview_scroll = 0;
		}
	}

	/// Selects the currently highlighted package in the list and sends a request to fetch it
	fn select_package(&mut self) {
		let Some(req) = self.get_selected_package() else {
			return;
		};

		if Some(req.clone()) == self.last_selected_package {
			return;
		}

		self.last_selected_package = Some(req.clone());
		let _ = self.task_tx.try_send(Task::FetchPackageInfo(req));
	}

	/// Installs the current package
	fn install_package(&self) {
		let Some(req) = self.get_installed_package() else {
			return;
		};

		let id = match self.install_prompt.location {
			InstallLocation::Instance => self
				.install_prompt
				.list_state
				.selected()
				.and_then(|i| self.instances.get(i))
				.map(|x| x.to_string()),
			InstallLocation::Template => self
				.install_prompt
				.list_state
				.selected()
				.and_then(|i| self.templates.get(i))
				.map(|x| x.to_string()),
			InstallLocation::NewInstance => {
				Some(self.install_prompt.new_id_state.lines()[0].to_string())
					.filter(|x| is_valid_instance_id(x))
			}
		};
		let Some(id) = id else {
			return;
		};

		let _ = self.task_tx.try_send(Task::InstallPackage {
			location: self.install_prompt.location,
			id,
			req,
		});
	}

	/// Requests an image to be downloaded
	fn request_image(&self, url: &str) {
		let url = url.to_string();
		let cache = self.image_cache.clone();
		let signal = self.image_available.clone();

		tokio::spawn(async move {
			let _ = cache.get(&url).await;
			signal.store(true, Ordering::Relaxed);
		});
	}
}

impl<'a> Drop for State<'a> {
	fn drop(&mut self) {
		self.worker.abort();
	}
}

/// Main event loop
fn renderer(terminal: &mut DefaultTerminal, mut state: State<'_>) -> anyhow::Result<()> {
	// Initial draw
	terminal.draw(|frame| render(frame, &mut state))?;

	loop {
		let mut should_render = false;

		// Check for results or state updates
		if let Ok(update) = state.worker_state_rx.try_recv() {
			state.worker_state = update;
			should_render = true;
		}

		if let Ok(results) = state.results_rx.try_recv() {
			state.results = results;
			state.package_list_state.select_first();
			state.select_package();
			should_render = true;
		}

		if let Ok(info) = state.package_info_rx.try_recv() {
			// Only update the preview if we are still actually looking at that package
			if Some(info.req.clone()) == state.get_selected_package() {
				state.package_info = Some(info);
				state.preview_scroll = 0;
				should_render = true;
			}
		}

		if state.image_available.swap(false, Ordering::Relaxed) {
			should_render = true;
		}

		// Key checks. If there is no key event, no need to re-render
		let key = get_key()?;
		if let Some(key) = key {
			should_render = true;

			match state.focus {
				_ if key.code == KeyCode::Esc => state.focus_out(),
				_ if key.code == KeyCode::Char('q') && state.focus != FocusState::Search => break,
				FocusState::None => match key.code {
					KeyCode::Char('s') | KeyCode::Char('/') => state.focus = FocusState::Search,
					KeyCode::Char('x') => state.reset_filters(),
					KeyCode::Char('r') => state.focus_popup(Popup::Repository),
					KeyCode::Char('t') => state.focus_popup(Popup::PackageType),
					KeyCode::Char('v') => state.focus_popup(Popup::Version),
					KeyCode::Char('l') => state.focus_popup(Popup::Loader),
					KeyCode::Char('c') => state.focus_popup(Popup::Category),
					KeyCode::Up | KeyCode::Char('k') => {
						state.package_list_state.select_previous();
						state.preview_package();
					}
					KeyCode::Down | KeyCode::Char('j') => {
						state.package_list_state.select_next();
						state.preview_package();
					}
					KeyCode::Enter | KeyCode::Tab | KeyCode::Char('p') => {
						state.focus_preview();
					}
					_ => {}
				},
				FocusState::Search => match key.code {
					KeyCode::Enter => {
						let line = state.search.lines().first().unwrap();
						if line.is_empty() {
							state.search_params.inner.search = None;
						} else {
							state.search_params.inner.search = Some(line.clone());
						}
						state.search();
						state.focus_none();
					}
					_ => {
						state.search.input(key);
					}
				},
				FocusState::Popup(popup) => popup.input(&mut state, key),
				FocusState::Preview => match key.code {
					KeyCode::Char('p') | KeyCode::Tab => state.focus_none(),
					KeyCode::Char('d') => state.focus_preview_tab(PreviewTab::Description),
					KeyCode::Char('v') => state.focus_preview_tab(PreviewTab::Versions),
					KeyCode::Char('g') => state.focus_preview_tab(PreviewTab::Gallery),
					KeyCode::Char('i') => state.focus_install(),
					KeyCode::Up | KeyCode::Char('k') if state.preview_scroll > 0 => {
						state.preview_scroll -= 1
					}
					KeyCode::Down | KeyCode::Char('j')
						if state.preview_scroll < state.preview_scroll_height =>
					{
						state.preview_scroll += 1
					}
					_ => {}
				},
				FocusState::InstallPrompt => match key.code {
					KeyCode::Enter => {
						state.install_package();
						state.focus_none();
					}
					_ if state.install_prompt.new_id_focused => {
						state.install_prompt.new_id_state.input(key);
					}
					KeyCode::Char('i') => state.install_prompt.location = InstallLocation::Instance,
					KeyCode::Char('t') => state.install_prompt.location = InstallLocation::Template,
					KeyCode::Char('n') => {
						if let Some(info) = &state.package_info
							&& info.is_modpack()
						{
							state.install_prompt.location = InstallLocation::NewInstance;
						}
					}
					KeyCode::Char('e')
						if state.install_prompt.location == InstallLocation::NewInstance =>
					{
						state.install_prompt.new_id_focused = true
					}
					KeyCode::Up | KeyCode::Char('k') => {
						state.install_prompt.list_state.select_previous()
					}
					KeyCode::Down | KeyCode::Char('j') => {
						state.install_prompt.list_state.select_next()
					}
					_ => {}
				},
			}
		};

		// Re-render
		if should_render {
			terminal.draw(|frame| render(frame, &mut state))?;
		}
	}
	Ok(())
}

/// Main render
fn render(frame: &mut Frame, state: &mut State) {
	let layout = Layout::vertical([
		Constraint::Length(1),
		Constraint::Fill(1),
		Constraint::Length(3),
		Constraint::Length(1),
	]);
	let layout = frame.area().layout::<4>(&layout);
	let filters_pane = layout[0];
	let preview_pane = layout[1];
	let search_pane = layout[2];
	let status_pane = layout[3];

	// Draw panes

	// Status bar
	let status_layout = Layout::horizontal([Constraint::Fill(3), Constraint::Fill(1)]);
	let status_layout = status_pane.layout::<2>(&status_layout);
	let keybinds_pane = status_layout[0];
	let state_pane = status_layout[1];

	let keybinds_text = match state.focus {
		FocusState::None => {
			"q to quit; s to search; k/j for up/down; enter to select; tab/p to focus package"
		}
		FocusState::Search => "esc to exit search; enter to submit",
		FocusState::Popup(..) => "esc to exit popup; k/j for up/down; enter to select",
		FocusState::Preview => "esc to exit preview; k/j for scroll; i to install",
		FocusState::InstallPrompt => "esc to exit prompt; k/j for up/down; enter to install",
	};

	let keybinds = Paragraph::new(keybinds_text);
	frame.render_widget(keybinds, keybinds_pane);

	let worker_state = match &state.worker_state {
		WorkerState::Idle => Paragraph::new("Idle").style(Style::new().gray()),
		WorkerState::Running => Paragraph::new("Running"),
		WorkerState::Success => Paragraph::new("Success").style(Style::new().light_green()),
		WorkerState::Error(error) => Paragraph::new(error.as_str()).style(Style::new().red()),
	};
	frame.render_widget(worker_state, state_pane);

	// Search bar
	let search_block = state.search.block().unwrap();
	let search_block_style = if state.focus == FocusState::Search {
		Style::new().green()
	} else {
		Style::new().gray()
	};
	let search_block = search_block.clone().border_style(search_block_style);
	state.search.set_block(search_block);
	frame.render_widget(&state.search, search_pane);

	// Package preview
	let preview_layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)]);
	let preview_layout = preview_pane.layout::<2>(&preview_layout);
	let list_pane = preview_layout[0];
	let preview_pane = preview_layout[1];

	// Package list
	let package_items = state.results.results.iter().map(|x| {
		if let Some(preview) = state.previews.get(x) {
			if let Some(name) = &preview.meta.name {
				name.clone()
			} else {
				x.to_string()
			}
		} else {
			x.to_string()
		}
	});
	state.package_list = state.package_list.clone().items(package_items);
	let mut block = rounded_block().title("Packages");
	if state.focus == FocusState::None {
		block = block.border_style(Style::new().green());
	}
	let inner = block.inner(list_pane);
	frame.render_widget(block, list_pane);
	frame.render_stateful_widget(&state.package_list, inner, &mut state.package_list_state);

	// Preview pane
	let mut block = rounded_block().title("Preview");
	if state.focus == FocusState::Preview {
		block = block.border_style(Style::new().green());
	}
	let inner_area = block.inner(preview_pane);
	frame.render_widget(block, preview_pane);
	if let Some(req) = state.get_selected_package() {
		let mut scroll_height = 0;
		let widget = PackageInfoWidget {
			req,
			info: state.package_info.as_ref(),
			state,
			scroll_height: &mut scroll_height,
		};

		frame.render_widget(widget, inner_area);
		state.preview_scroll_height = scroll_height;
	} else {
		frame.render_widget(Clear, inner_area);
	};

	// Filters
	let filter_layout = Layout::horizontal(Constraint::from_fills([1, 1, 1, 1, 1]));
	let filter_layout = filters_pane.layout::<5>(&filter_layout);
	let repo_pane = filter_layout[0];
	let type_pane = filter_layout[1];
	let version_pane = filter_layout[2];
	let loader_pane = filter_layout[3];
	let category_pane = filter_layout[4];

	let repo = format!(
		"[r] Repository: {}",
		state.search_params.repo.as_deref().unwrap_or("All")
	);
	let repo = Paragraph::new(repo).style(Style::new().bold().light_blue());
	frame.render_widget(repo, repo_pane);

	let ty = format!(
		"[t] Package type: {}",
		state
			.search_params
			.inner
			.types
			.first()
			.map(|x| x.to_string())
			.unwrap_or("Any".into())
	);
	let ty = Paragraph::new(ty).style(Style::new().bold().light_blue());
	frame.render_widget(ty, type_pane);

	let version = match state.search_params.inner.minecraft_versions.len() {
		0 => "Any",
		1 => state
			.search_params
			.inner
			.minecraft_versions
			.first()
			.unwrap()
			.as_str(),
		_ => "Multiple",
	};
	let version =
		Paragraph::new(format!("[v] Version: {version}")).style(Style::new().bold().light_blue());
	frame.render_widget(version, version_pane);

	let loader = match state.search_params.inner.loaders.len() {
		0 => "Any".to_string(),
		1 => state
			.search_params
			.inner
			.loaders
			.first()
			.unwrap()
			.to_string(),
		_ => "Multiple".to_string(),
	};
	let loader =
		Paragraph::new(format!("[l] Loader: {loader}")).style(Style::new().bold().light_blue());
	frame.render_widget(loader, loader_pane);

	let category = match state.search_params.inner.categories.len() {
		0 => "Any".to_string(),
		1 => to_string_json(state.search_params.inner.categories.first().unwrap()),
		_ => "Multiple".to_string(),
	};
	let category =
		Paragraph::new(format!("[c] Category: {category}")).style(Style::new().bold().light_blue());
	frame.render_widget(category, category_pane);

	// Popup
	if let FocusState::Popup(popup) = state.focus {
		let mut base_area = frame.area().inner(Margin::new(1, 1));
		base_area.height = 10;

		popup.render(state, frame, base_area);
	}

	// Install prompt
	if state.focus == FocusState::InstallPrompt {
		let area = frame.area().inner(Margin::new(15, 10));
		InstallPromptState::render(frame, area, state);
	}
}

/// Focus state for the TUI
#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusState {
	None,
	Search,
	Popup(Popup),
	Preview,
	InstallPrompt,
}

/// Different selection popups
#[derive(Clone, Copy, PartialEq, Eq)]
enum Popup {
	Repository,
	PackageType,
	Version,
	Loader,
	Category,
}

impl Popup {
	fn render(&self, state: &mut State, frame: &mut Frame, area: Rect) {
		let layout = Layout::horizontal(Constraint::from_fills([1, 1, 1, 1, 1]));
		let layout = area.layout::<5>(&layout);

		let area = match self {
			Self::Repository => layout[0],
			Self::PackageType => layout[1],
			Self::Version => layout[2],
			Self::Loader => layout[3],
			Self::Category => layout[4],
		};

		let block = rounded_block()
			.border_style(Style::new().green())
			.title(self.title());

		let selected = self.get_selected(state);
		let items = self.get_items(state);
		let items = items.into_iter().enumerate().map(|(i, x)| {
			if selected.contains(&i) {
				format!("{x} (Selected)")
			} else {
				x
			}
		});

		let list = List::default()
			.block(block)
			.items(items)
			.highlight_symbol(">")
			.highlight_style(Style::new().green());

		frame.render_widget(Clear, area);
		frame.render_stateful_widget(list, area, &mut state.popup_list_state);
	}

	fn input(&self, state: &mut State, input: KeyEvent) {
		match input.code {
			KeyCode::Char('r') if *self == Popup::Repository => state.focus_none(),
			KeyCode::Char('t') if *self == Popup::PackageType => state.focus_none(),
			KeyCode::Char('v') if *self == Popup::Version => state.focus_none(),
			KeyCode::Char('l') if *self == Popup::Loader => state.focus_none(),
			KeyCode::Char('c') if *self == Popup::Category => state.focus_none(),
			KeyCode::Up | KeyCode::Char('k') => state.popup_list_state.select_previous(),
			KeyCode::Down | KeyCode::Char('j') => state.popup_list_state.select_next(),
			KeyCode::Enter => match self {
				// Single select
				Self::Repository | Self::PackageType => {
					if let Some(selected) = state.popup_list_state.selected() {
						self.select(state, selected);
						state.search();
						state.focus = FocusState::None;
					}
				}
				// Multi select
				Self::Version | Self::Loader | Self::Category => {
					if let Some(selected) = state.popup_list_state.selected() {
						self.select(state, selected);
						state.search();
					}
				}
			},
			_ => {}
		}
	}

	fn select(&self, state: &mut State, pos: usize) {
		match self {
			Self::Repository => {
				if pos == 0 {
					state.search_params.repo = None;
					return;
				}

				let Some(repo) = state.repositories.get(pos - 1) else {
					return;
				};

				state.search_params.repo = Some(repo.id.clone());

				// Handle package types and categories that are no longer supported by this repo
				let available_pkg_types = state.get_available_package_types();
				if !state
					.search_params
					.inner
					.types
					.iter()
					.all(|x| available_pkg_types.contains(x))
				{
					state.search_params.inner.types =
						available_pkg_types.first().into_iter().cloned().collect();
				}

				let available_categories = state.get_available_package_categories();
				state
					.search_params
					.inner
					.categories
					.retain(|x| available_categories.contains(x));
			}
			Self::PackageType => {
				let types = state.get_available_package_types();
				let Some(ty) = types.get(pos) else {
					return;
				};

				state.search_params.inner.types = vec![*ty];
			}
			Self::Version => {
				let Some(version) = state.versions.get(pos) else {
					return;
				};

				if state
					.search_params
					.inner
					.minecraft_versions
					.contains(version)
				{
					state
						.search_params
						.inner
						.minecraft_versions
						.retain(|x| x != version);
				} else {
					state
						.search_params
						.inner
						.minecraft_versions
						.push(version.clone());
				}
			}
			Self::Loader => {
				let Some(loader) = state.loaders.get(pos) else {
					return;
				};

				if state.search_params.inner.loaders.contains(loader) {
					state.search_params.inner.loaders.retain(|x| x != loader);
				} else {
					state.search_params.inner.loaders.push(loader.clone());
				}
			}
			Self::Category => {
				let categories = state.get_available_package_categories();
				let Some(category) = categories.get(pos) else {
					return;
				};
				let category = *category;

				if state.search_params.inner.categories.contains(&category) {
					state
						.search_params
						.inner
						.categories
						.retain(|x| x != &category);
				} else {
					state.search_params.inner.categories.push(category);
				}
			}
		}
	}

	fn get_selected(&self, state: &State) -> Vec<usize> {
		match self {
			Self::Repository => match &state.search_params.repo {
				Some(repo) => state
					.repositories
					.iter()
					.position(|y| y.id == *repo)
					.map(|x| x + 1)
					.into_iter()
					.collect(),
				None => vec![0],
			},
			Self::PackageType => {
				let Some(ty) = state.search_params.inner.types.first() else {
					return Vec::new();
				};

				let types = state.get_available_package_types();

				types.iter().position(|x| x == ty).into_iter().collect()
			}
			Self::Version => state
				.search_params
				.inner
				.minecraft_versions
				.iter()
				.filter_map(|x| state.versions.iter().position(|y| y == x))
				.collect(),
			Self::Loader => state
				.search_params
				.inner
				.loaders
				.iter()
				.filter_map(|x| state.loaders.iter().position(|y| y == x))
				.collect(),
			Self::Category => {
				let categories = state.get_available_package_categories();

				state
					.search_params
					.inner
					.categories
					.iter()
					.filter_map(|x| categories.iter().position(|y| y == x))
					.collect()
			}
		}
	}

	fn get_items(&self, state: &State) -> Vec<String> {
		match self {
			Self::Repository => {
				let mut items = vec!["All".to_string()];
				items.extend(state.repositories.iter().map(|x| x.id.clone()));
				items
			}
			Self::PackageType => state
				.get_available_package_types()
				.iter()
				.map(|x| x.to_string())
				.collect(),
			Self::Version => state.versions.clone(),
			Self::Loader => state.loaders.iter().map(|x| x.to_string()).collect(),
			Self::Category => state
				.get_available_package_categories()
				.iter()
				.map(to_string_json)
				.collect(),
		}
	}

	fn title(&self) -> &'static str {
		match self {
			Self::Repository => "Select repository",
			Self::PackageType => "Select package type",
			Self::Version => "Select Minecraft versions",
			Self::Loader => "Select loaders",
			Self::Category => "Select categories",
		}
	}
}

/// Search parameters including repository
#[derive(Clone)]
struct SearchParams {
	inner: PackageSearchParameters,
	repo: Option<String>,
}

/// Info about a package
struct PackageInfo {
	req: ArcPkgReq,
	preview: PackageMetaAndProps,
	versions: Vec<DeclarativeAddonVersion>,
}

impl PackageInfo {
	fn is_modpack(&self) -> bool {
		self.preview.props.kinds.contains(&PackageKind::Modpack)
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreviewTab {
	Description,
	Versions,
	Gallery,
}

impl PreviewTab {
	fn title(&self) -> &'static str {
		match self {
			Self::Description => "Description [d]",
			Self::Versions => "Versions [v]",
			Self::Gallery => "Gallery [g]",
		}
	}
}

struct PackageInfoWidget<'a> {
	req: ArcPkgReq,
	info: Option<&'a PackageInfo>,
	state: &'a State<'a>,
	scroll_height: &'a mut u16,
}

impl<'a> Widget for PackageInfoWidget<'a> {
	fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
		let Some(info) = self.info else {
			Clear.render(area, buf);
			return;
		};

		let is_loaded = Some(self.state.package_info.as_ref().unwrap().req.clone())
			== self.state.last_selected_package;

		let layout = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]);
		let layout = area.layout::<2>(&layout);
		let top_pane = layout[0];
		let bottom_pane = layout[1];

		// Top pane
		let block = Block::new().borders(Borders::BOTTOM);
		let inner_area = block.inner(top_pane);
		block.render(top_pane, buf);

		let layout = Layout::horizontal([Constraint::Length(6), Constraint::Fill(1)]);
		let layout = inner_area.layout::<2>(&layout);
		let icon_pane = layout[0];
		let details_pane = layout[1].inner(Margin::new(1, 1));

		// Icon
		if let Some(icon) = &info.preview.meta.icon {
			if let Some(image) = self.state.image_cache.get_from_cache(icon) {
				let picker = Picker::from_query_stdio().unwrap_or(Picker::halfblocks());
				let image = (*image).clone();
				if let Ok(image) = picker.new_protocol(
					DynamicImage::ImageRgb8(image),
					icon_pane,
					Resize::Scale(None),
				) {
					let image = Image::new(&image);
					image.render(icon_pane, buf);
				}
			} else {
				self.state.request_image(icon);
			}
		}

		// Details
		let layout = Layout::horizontal(Constraint::from_fills([1, 1, 1]));
		let [title_pane, req_pane, not_loaded_pane] = details_pane.layout::<3>(&layout);

		// Title
		let title_name = if let Some(name) = &info.preview.meta.name {
			name.as_str()
		} else if let Some(slug) = &info.preview.meta.slug {
			slug.as_str()
		} else {
			&self.req.id
		};

		let title = if let Some(repo) = &self.req.repository {
			format!("{title_name} - {repo}")
		} else {
			title_name.to_string()
		};

		let title = Paragraph::new(title).style(Style::new().bold());
		title.render(title_pane, buf);

		// Request
		let req = Paragraph::new(
			self.req
				.with_slug(info.preview.meta.slug.clone())
				.to_string_no_version(),
		)
		.style(Style::new().gray());
		req.render(req_pane, buf);

		// Subtitle
		let mut subtitle_area = details_pane;
		subtitle_area.y += 1;

		if let Some(short_description) = &info.preview.meta.description {
			let short_description = Paragraph::new(short_description.as_str());
			short_description
				.style(Style::new().gray())
				.render(subtitle_area, buf);
		}

		// Not loaded indicator
		if !is_loaded {
			Paragraph::new("Press enter to load package")
				.centered()
				.style(Style::new().green().bold())
				.render(not_loaded_pane, buf);
		} else {
			Clear.render(not_loaded_pane, buf);
		}

		// Bottom pane
		let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
		let layout = bottom_pane.layout::<2>(&layout);
		let tabs_pane = layout[0];
		let body_pane = layout[1].inner(Margin::new(1, 1));

		// Tabs
		let layout = Layout::horizontal(Constraint::from_fills([1, 1, 1]));
		let layout = tabs_pane.layout::<3>(&layout);

		for (i, id) in [
			PreviewTab::Description,
			PreviewTab::Versions,
			PreviewTab::Gallery,
		]
		.into_iter()
		.enumerate()
		{
			let mut tab = Paragraph::new(id.title()).alignment(HorizontalAlignment::Center);
			if self.state.preview_tab == id {
				tab = tab.style(Style::new().reversed());
			}
			tab.render(layout[i], buf);
		}

		// Body
		let not_loaded_indicator = Paragraph::new("Press enter to load package").centered();
		let not_loaded_area = body_pane.centered(Constraint::Ratio(1, 3), Constraint::Ratio(1, 2));
		match self.state.preview_tab {
			PreviewTab::Description => {
				if !is_loaded {
					not_loaded_indicator.render(not_loaded_area, buf);
				} else if let Some(body) = &info.preview.meta.long_description {
					let text = tui_markdown::from_str(body);

					let visible_lines = body_pane.height;
					let scroll_height = text.lines.len() as u16;
					let scroll_height = scroll_height.saturating_sub(visible_lines);
					*self.scroll_height = scroll_height;

					let markdown = Paragraph::new(text)
						.scroll((self.state.preview_scroll, 0))
						.wrap(Wrap { trim: false });
					markdown.render(body_pane, buf);
				}
			}
			PreviewTab::Versions => {
				Clear.render(body_pane, buf);
				if !is_loaded {
					not_loaded_indicator.render(not_loaded_area, buf);
				} else {
					*self.scroll_height =
						render_versions(&info.versions, self.state, body_pane, buf);
				}
			}
			PreviewTab::Gallery => {
				Clear.render(body_pane, buf);
				if let Some(gallery) = &info.preview.meta.gallery {
					*self.scroll_height = render_gallery(gallery, self.state, body_pane, buf);
				}
			}
		}
	}
}

/// Renders package versions, returning the scroll height
fn render_versions(
	versions: &[DeclarativeAddonVersion],
	state: &State,
	area: Rect,
	buf: &mut ratatui::prelude::Buffer,
) -> u16 {
	let list = List::default()
		.highlight_symbol(">")
		.highlight_style(Style::new());

	let items = versions.iter().map(|x| {
		let layout = Layout::horizontal([
			Constraint::Length(9),
			Constraint::Fill(1),
			Constraint::Fill(3),
			Constraint::Fill(1),
		]);
		let [_, name_pane, versions_pane, loaders_pane] = area.layout(&layout);

		// Stability
		let stability = x.conditional_properties.stability.unwrap_or_default();
		let stability = match stability {
			PackageStability::Stable => Span::styled("[stable] ", Style::new().green()),
			PackageStability::Latest => Span::styled("[debug]  ", Style::new().yellow()),
		};

		// Version name
		let version_name = x
			.conditional_properties
			.content_versions
			.as_ref()
			.and_then(|x| x.first())
			.map(|x| x.as_str())
			.unwrap_or("Unknown");
		let version_name = fit_message_width(version_name, name_pane.width as usize);
		let version_name = Span::styled(version_name, Style::new().bold());

		// Versions
		let versions = if let Some(versions) = &x.conditional_properties.minecraft_versions {
			let out = versions.iter().take(4).map(|x| x.to_string()).join(" ");
			if versions.len() > 4 { out + "..." } else { out }
		} else {
			String::new()
		};
		let versions = fit_message_width(&versions, versions_pane.width as usize);
		let versions = Span::styled(versions.to_string(), Style::new().light_green());

		// Loaders
		let loaders = if let Some(loaders) = &x.conditional_properties.loaders {
			let out = loaders.iter().take(3).map(|x| x.to_string()).join(" ");
			if loaders.len() > 3 { out + "..." } else { out }
		} else {
			String::new()
		};
		let loaders = fit_message_width(&loaders, loaders_pane.width as usize);
		let loaders = Span::styled(loaders.to_string(), Style::new().blue());

		let line = Line::from(vec![stability, version_name, versions, loaders]);

		ListItem::new(line)
	});
	let list = list.items(items);

	let len = list.len() as u16;

	let mut list_state = ListState::default();
	list_state.select(Some(state.preview_scroll as usize));

	StatefulWidget::render(list, area, buf, &mut list_state);

	len
}

/// Renders a package gallery, returning the scroll height
fn render_gallery(
	gallery: &[String],
	state: &State,
	area: Rect,
	buf: &mut ratatui::prelude::Buffer,
) -> u16 {
	const WIDTH: usize = 3;
	const HEIGHT: usize = 3;

	let picker = Picker::from_query_stdio().unwrap_or(Picker::halfblocks());
	let cell_size = get_cell_size();
	if cell_size.0 == 0 || cell_size.1 == 0 {
		return 0;
	}

	let vertical_layout = Layout::vertical(Constraint::from_fills([1; HEIGHT]));
	for (row_i, row) in vertical_layout.split(area).iter().enumerate() {
		let horizontal_layout = Layout::horizontal(Constraint::from_fills([1; WIDTH]));
		let horizontal_layout = horizontal_layout.split(*row);

		let start = (row_i + state.preview_scroll as usize) * WIDTH;
		let end = start + WIDTH;

		let mut col = 0;
		for i in start..end {
			if i >= gallery.len() {
				continue;
			}

			let area = horizontal_layout[col].inner(Margin::new(1, 1));
			let url = &gallery[i];

			let border = rounded_block();
			border.render(area, buf);

			if let Some(image) = state.image_cache.get_from_cache(url) {
				// Crop to aspect ratio
				let width_pixels = area.width * cell_size.0 as u16;
				let height_pixels = area.height * cell_size.1 as u16;
				let aspect_ratio = width_pixels as f32 / height_pixels as f32;
				let image = crop_image_to_ratio(&image, aspect_ratio).to_image();

				if let Ok(image) = picker.new_protocol(
					DynamicImage::ImageRgb8(image),
					area,
					Resize::Scale(Some(ratatui_image::FilterType::CatmullRom)),
				) {
					let image = Image::new(&image);
					image.render(area, buf);
				}
			} else {
				state.request_image(url);
			}

			col += 1;
		}
	}

	// Account for trailing incomplete rows
	let scroll_height = if gallery.len().is_multiple_of(WIDTH) {
		gallery.len() / WIDTH
	} else {
		gallery.len() / WIDTH + 1
	};

	// Scroll height is equal to how much we need to go beyond the normally visible contents
	if scroll_height > HEIGHT {
		(scroll_height - HEIGHT) as u16
	} else {
		0
	}
}

/// State for the install prompt
struct InstallPromptState<'a> {
	location: InstallLocation,
	list_state: ListState,
	new_id_state: TextArea<'a>,
	new_id_focused: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InstallLocation {
	Instance,
	Template,
	NewInstance,
}

impl InstallLocation {
	fn title(&self) -> &'static str {
		match self {
			Self::Instance => "[i] Instance",
			Self::Template => "[t] Template",
			Self::NewInstance => "[n] New Instance",
		}
	}
}

impl<'a> InstallPromptState<'a> {
	fn render(frame: &mut Frame, area: Rect, state: &mut State) {
		let Some(info) = &state.package_info else {
			state.focus_none();
			return;
		};

		let Some(req) = state.get_selected_package() else {
			state.focus_none();
			return;
		};

		let title = if let Some(name) = &info.preview.meta.name {
			name.as_str()
		} else if let Some(slug) = &info.preview.meta.slug {
			slug.as_str()
		} else {
			&req.id
		};

		let block = rounded_block().title(format!("Install {title}"));
		let inner_area = block.inner(area);
		frame.render_widget(block, area);
		frame.render_widget(Clear, inner_area);

		let layout = Layout::vertical([
			Constraint::Length(3),
			Constraint::Length(1),
			Constraint::Fill(1),
		]);
		let [version_pane, tabs_pane, body_pane] = inner_area.layout(&layout);
		let version_pane = version_pane.inner(Margin::new(1, 1));

		// Version
		let content_version = if state.preview_tab == PreviewTab::Versions {
			if let Some(version) = info.versions.get(state.preview_scroll as usize) {
				if let Some(content_versions) = &version.conditional_properties.content_versions {
					content_versions.first()
				} else {
					None
				}
			} else {
				None
			}
		} else {
			None
		};

		let version = if let Some(content_version) = &content_version {
			Paragraph::new(format!("Selected version: {content_version}"))
				.style(Style::new().green())
		} else {
			Paragraph::new("Best version will be used")
		};
		frame.render_widget(version, version_pane);

		// Tabs
		let tabs = if info.is_modpack() {
			vec![
				InstallLocation::Instance,
				InstallLocation::Template,
				InstallLocation::NewInstance,
			]
		} else {
			vec![InstallLocation::Instance, InstallLocation::Template]
		};

		let layout = Layout::horizontal(Constraint::from_fills(std::iter::repeat_n(1, tabs.len())));
		let layout = tabs_pane.layout_vec(&layout);

		for (i, location) in tabs.into_iter().enumerate() {
			let mut tab = Paragraph::new(location.title()).alignment(HorizontalAlignment::Center);
			if state.install_prompt.location == location {
				tab = tab.style(Style::new().reversed());
			}
			frame.render_widget(tab, layout[i]);
		}

		// List selection
		match state.install_prompt.location {
			InstallLocation::Instance | InstallLocation::Template => {
				let list = List::default().highlight_symbol(">");
				let list = if state.install_prompt.location == InstallLocation::Instance {
					list.items(state.instances.iter().sorted().map(|x| x.deref()))
						.highlight_style(Style::new().green())
				} else {
					list.items(state.templates.iter().sorted().map(|x| x.deref()))
						.highlight_style(Style::new().light_blue())
				};

				frame.render_stateful_widget(list, body_pane, &mut state.install_prompt.list_state);
			}
			InstallLocation::NewInstance => {
				let layout = Layout::vertical([
					Constraint::Fill(1),
					Constraint::Length(1),
					Constraint::Length(3),
				]);
				let [_, hint_pane, input_pane] = body_pane.layout(&layout);

				// Hint
				let hint = if state.install_prompt.new_id_state.is_empty() {
					Paragraph::new("Enter a valid ID for the new instance")
				} else if is_valid_instance_id(&state.install_prompt.new_id_state.lines()[0]) {
					Paragraph::new("ID is valid").style(Style::new().green())
				} else {
					Paragraph::new("ID is invalid").style(Style::new().red())
				};

				frame.render_widget(hint, hint_pane);

				// New ID input
				let block = rounded_block();
				let block_style = if state.install_prompt.new_id_focused {
					Style::new().green()
				} else {
					Style::new().gray()
				};
				let block = block
					.clone()
					.title("[e] to focus")
					.border_style(block_style);
				state.install_prompt.new_id_state.set_block(block);
				frame.render_widget(&state.install_prompt.new_id_state, input_pane);
			}
		}
	}
}

/// Loop for working tokio task that does stuff like fetch search results
async fn worker_thread(
	config: Config,
	paths: Paths,
	mut session: PackageSearchSession,
	state_tx: Sender<WorkerState>,
	mut task_rx: Receiver<Task>,
	results_tx: Sender<PackageMultiSearchResults>,
	package_info_tx: Sender<PackageInfo>,
) {
	let client = Client::new();

	loop {
		let Some(task) = task_rx.recv().await else {
			continue;
		};

		let _ = state_tx.try_send(WorkerState::Running);
		match task {
			Task::FetchPackages(params) => {
				let results = session
					.search(
						params.inner,
						params.repo.as_deref(),
						config.packages.clone(),
						&paths,
						&client,
						&mut NoOp,
					)
					.await;

				match results {
					Ok(results) => {
						let _ = results_tx.send(results).await;
						let _ = state_tx.try_send(WorkerState::Success);
					}
					Err(e) => {
						let _ = state_tx.try_send(WorkerState::Error(e.to_string()));
					}
				}
			}
			Task::FetchPackageInfo(req) => {
				let pkg = config.packages.get(&req, &paths, &client, &mut NoOp).await;
				let pkg = match pkg {
					Ok(pkg) => pkg,
					Err(e) => {
						let _ = state_tx.try_send(WorkerState::Error(e.to_string()));
						continue;
					}
				};

				let Ok(meta) = pkg.get_metadata(&paths, &client).await else {
					let _ = state_tx.try_send(WorkerState::Error("Failed to get metadata".into()));
					continue;
				};
				let Ok(props) = pkg.get_properties(&paths, &client).await else {
					let _ =
						state_tx.try_send(WorkerState::Error("Failed to get properties".into()));
					continue;
				};

				let versions = match pkg.get_content_versions(&paths, &client).await {
					Ok(versions) => versions.into_iter().map(|x| x.into_owned()).collect(),
					Err(e) => {
						let _ = state_tx.try_send(WorkerState::Error(e.to_string()));
						Vec::new()
					}
				};

				let _ = package_info_tx
					.send(PackageInfo {
						req,
						preview: PackageMetaAndProps { meta, props },
						versions,
					})
					.await;
				let _ = state_tx.try_send(WorkerState::Success);
			}
			Task::InstallPackage { location, id, req } => {
				let modification = match location {
					InstallLocation::Instance => {
						let id = InstanceID::from(id);
						let Some(instance) = config.instances.get(&id) else {
							continue;
						};

						let mut config = instance.original_config().clone();
						add_or_update_package_config(
							&mut config.packages,
							PackageConfigDeser::Basic(req.to_string()),
						);

						ConfigModification::UpdateInstance(id, config)
					}
					InstallLocation::Template => {
						let id = TemplateID::from(id);
						let Some(template) = config.templates.get(&id) else {
							continue;
						};
						let mut config = template.clone();

						config
							.packages
							.add_package(PackageConfigDeser::Basic(req.to_string().into()), None);

						ConfigModification::UpdateTemplate(id, config)
					}
					InstallLocation::NewInstance => {
						let instance_config = install_into_config(
							&req,
							id.clone().into(),
							Some(Side::Client),
							&config,
							&paths,
							&mut NoOp,
						)
						.await;
						let instance_config = match instance_config {
							Ok(instance_config) => instance_config,
							Err(e) => {
								let _ = state_tx.try_send(WorkerState::Error(format!(
									"Failed to import modpack: {e}"
								)));
								continue;
							}
						};

						ConfigModification::AddInstance(id.into(), instance_config)
					}
				};

				let Ok(mut raw_config) = Config::open(&Config::get_path(&paths)) else {
					let _ = state_tx.try_send(WorkerState::Error("Failed to open config".into()));
					continue;
				};

				if let Err(e) = apply_modifications_and_write(
					&mut raw_config,
					vec![modification],
					&paths,
					&config.plugins,
					&mut NoOp,
				)
				.await
				{
					let _ = state_tx
						.try_send(WorkerState::Error(format!("Failed to write config: {e}")));
				}

				let _ = state_tx.try_send(WorkerState::Success);
			}
		}
	}
}

/// Task that the worker thread can run
enum Task {
	FetchPackages(SearchParams),
	FetchPackageInfo(ArcPkgReq),
	InstallPackage {
		location: InstallLocation,
		id: String,
		req: ArcPkgReq,
	},
}

/// State of the worker thread
enum WorkerState {
	Idle,
	Running,
	Success,
	Error(String),
}

fn rounded_block<'a>() -> Block<'a> {
	Block::bordered().border_set(symbols::border::ROUNDED)
}

/// Attempts to get the width and height in pixels of an individual terminal character
fn get_cell_size() -> (u8, u8) {
	let default = (1, 2);
	let Ok(total_size) = crossterm::terminal::window_size() else {
		return default;
	};

	let cell_width = total_size.width as f32 / total_size.columns as f32;
	let cell_height = total_size.height as f32 / total_size.rows as f32;

	(cell_width as u8, cell_height as u8)
}
