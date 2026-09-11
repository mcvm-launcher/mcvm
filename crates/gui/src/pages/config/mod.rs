use std::sync::Arc;

use crate::{
	components::{
		dialog::modal::{Modal, ModalButton},
		input::{
			control::{ControlSection, ControlledConfig, Controls, filter_control},
			final_value_owned,
			tabs::SideTabs,
		},
	},
	ops::{
		ToastedMutation,
		instance::{
			FetchInstanceOrTemplateConfig, FetchParentConfigs, SaveConfig, SaveConfigParams,
		},
		plugin_results::{FetchInstanceControls, FetchInstanceControlsKeys},
	},
	pages::config::{content::ContentConfig, general::GeneralTab, launch::LaunchConfigPage},
	prelude::*,
	state::{FrontState, ModalType},
	util::{PtrEq, Shared},
};
use freya::query::UseMutation;
use itertools::Itertools;
use nitrolaunch::{
	config_crate::{
		ConfigKind,
		template::{TemplateConfig, TemplateLoaderConfiguration, TemplatePackageConfiguration},
	},
	core::util::versions::MinecraftVersion,
	shared::{
		Side,
		loaders::Loader,
		util::{DeserListOrSingle, to_string_json},
		versions::{
			MinecraftLatestVersion, MinecraftVersionDeser, VersionPattern, format_versioned_string,
		},
	},
};

pub mod content;
mod general;
mod launch;

#[derive(PartialEq)]
pub struct ConfigPage;

impl Component for ConfigPage {
	fn render(&self) -> impl IntoElement {
		let front_state = use_front_state();
		front_state.read().subscribe(FrontChannel::Modal);
		let back_state = use_consume::<BackState>();
		let item = front_state.read().modal().and_then(|x| {
			if let ModalType::Configuration(item) = x {
				Some(item.clone())
			} else {
				None
			}
		});
		let save_config = use_mutation(Mutation::new(SaveConfig::new(back_state.clone()).toast(
			&back_state,
			Some("Saved"),
			"Failed to save config",
		)));
		let config_state = ConfigState::new(
			item.as_ref().map(|x| x.ty).unwrap_or_default(),
			item.as_ref().map(|x| x.is_new).unwrap_or(false),
		);

		let save_fn = config_state.save_fn(front_state.clone(), save_config);
		let front_state2 = front_state.clone();
		let on_submit = move |_: ()| {
			let successful = save_fn();

			if successful {
				front_state2.write().set_modal(None);
			}
		};

		let title = match &item {
			Some(item) => match item.ty {
				ConfigKind::Instance => match &item.id {
					Some(id) => format!("Configuring instance {id}"),
					None => "Creating new instance".into(),
				},
				ConfigKind::Template => match &item.id {
					Some(id) => format!("Configuring template {id}"),
					None => "Creating new template".into(),
				},
				ConfigKind::BaseTemplate => "Configuring base template".into(),
			},
			None => "".into(),
		};

		Modal::new(title, "gear".into())
			.maybe_child(item.is_some(), || ConfigModal {
				item: item.clone().unwrap(),
				config_state: config_state.clone(),
			})
			.size_large()
			.on_close(move |_| front_state.write().set_modal(None))
			.cancel_button()
			.button(ModalButton {
				title: "Save".into(),
				icon: "check".into(),
				on_click: on_submit.into(),
				active: *config_state.is_dirty.read(),
			})
	}
}

#[derive(PartialEq)]
struct ConfigModal {
	item: ConfiguredItem,
	config_state: ConfigState,
}

impl Component for ConfigModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let config_query = use_query(FetchInstanceOrTemplateConfig::new(
			self.item.clone(),
			back_state.clone(),
		));

		let parent_configs = use_query(FetchParentConfigs::new(
			self.config_state.from.read().cloned(),
			back_state.clone(),
		));
		let parent_configs = parent_configs
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		self.config_state
			.parent_configs
			.clone()
			.set_if_modified(PtrEq(parent_configs.clone()));

		let controls = use_query(Query::new(
			FetchInstanceControlsKeys {
				id: self.item.id.clone(),
				ty: self.item.ty,
				config_plugin: None,
			},
			FetchInstanceControls::new(back_state.clone()),
		));

		let controls = use_memo(move || {
			let controls = controls.read();
			let controls = controls.state();
			let controls = controls.ok().cloned().unwrap_or(PtrEq(Arc::default()));
			NotEq(ControlSection::sectionize(
				&controls.0,
				ControlSection {
					id: "plugins".into(),
					name: "Plugins".into(),
					icon: "jigsaw".into(),
					..Default::default()
				},
			))
		});

		let id = self.item.id.clone();
		let mut config_state = self.config_state.clone();
		use_side_effect(move || {
			let config = config_query
				.read()
				.state()
				.ok()
				.cloned()
				.flatten()
				.unwrap_or_default();

			config_state.update(id.clone(), config.no_templates);
		});

		let tab = use_state(|| Tab::General);
		let final_side = final_value_owned(
			self.config_state.side.read().cloned(),
			&parent_configs,
			|x| x.instance.side,
		);
		let left_panel = rect()
			.width(Size::flex(1.0))
			.border(border_right(theme.border, theme.panel_border))
			.child(
				SideTabs::from_state(tab)
					.child(SelectOption::new(Tab::General, "General", Some("gear")))
					.child(SelectOption::new(
						Tab::Content,
						"Content",
						Some("honeycomb"),
					))
					.child(SelectOption::new(
						Tab::Launch,
						"Launch Settings",
						Some("play"),
					))
					.children(
						controls
							.read()
							.0
							.values()
							.filter(|x| {
								x.controls
									.iter()
									.filter(|x| filter_control(x, final_side))
									.count() > 0
							})
							.sorted_by_cached_key(|x| x.name.clone())
							.map(|section| {
								SelectOption::new(
									Tab::Custom(section.id.clone()),
									&section.name,
									Some(&section.icon),
								)
							}),
					),
			);

		let tab_contents = match &*tab.read() {
			Tab::General => GeneralTab {
				config_state: self.config_state.clone(),
				parent_configs: PtrEq(parent_configs.clone()),
			}
			.into_element(),
			Tab::Content => ContentConfig {
				config_state: self.config_state.clone(),
				parent_configs: PtrEq(parent_configs.clone()),
				on_edit: None,
			}
			.into_element(),
			Tab::Launch => LaunchConfigPage {
				config_state: self.config_state.clone(),
				parent_configs: PtrEq(parent_configs.clone()),
			}
			.into_element(),
			Tab::Custom(id) => {
				let controls = controls.read();
				let default = Arc::default();
				let children = controls.0.get(id).map(|x| &x.controls).unwrap_or(&default);
				Controls {
					controls: PtrEq(children.clone()),
					values: self.config_state.plugin_config,
					side: final_side,
				}
				.into_element()
			}
		};

		let right_panel = rect().width(Size::flex(4.25)).child(tab_contents);

		rect()
			.horizontal()
			.flex()
			.maybe(!self.config_state.is_new, |this| this.child(left_panel))
			.child(right_panel)
	}
}

#[derive(PartialEq, Clone)]
enum Tab {
	General,
	Content,
	Launch,
	Custom(String),
}

/// State objects for the config
#[derive(Clone, PartialEq)]
pub struct ConfigState {
	pub ty: ConfigKind,
	pub is_new: bool,
	/// Whether any of the config fields have been edited
	pub is_dirty: State<bool>,
	/// Whether we can propagate the name to the ID
	pub is_id_dirty: State<bool>,
	pub original_config: State<PtrEq<TemplateConfig>>,
	pub parent_configs: State<PtrEq<[TemplateConfig]>>,
	pub id: State<String>,
	pub original_id: State<String>,
	pub from: State<Vec<String>>,
	pub name: State<Option<String>>,
	pub icon: State<Option<String>>,
	pub side: State<Option<Side>>,
	pub version: State<Option<String>>,
	pub client_loader: State<Option<Loader>>,
	pub server_loader: State<Option<Loader>>,
	pub client_loader_version: State<VersionPattern>,
	pub server_loader_version: State<VersionPattern>,
	pub packages: State<TemplatePackageConfiguration>,
	pub modpack: State<Option<String>>,
	pub datapack_folder: State<Option<String>>,
	pub java: State<Option<String>>,
	pub plugin: State<Option<String>>,
	pub plugin_config: State<ControlledConfig>,
}

impl ConfigState {
	/// Must be called from component render scope
	pub fn new(ty: ConfigKind, is_new: bool) -> Self {
		let out = Self {
			ty,
			is_new,
			is_dirty: use_state(|| false),
			is_id_dirty: use_state(|| false),
			original_config: use_state(|| PtrEq(Arc::new(TemplateConfig::default()))),
			parent_configs: use_state(|| PtrEq(Arc::default())),
			id: use_state(String::new),
			original_id: use_state(String::new),
			from: use_state(Vec::new),
			name: use_state(|| None),
			icon: use_state(|| None),
			side: use_state(|| None),
			version: use_state(|| None),
			client_loader: use_state(|| None),
			server_loader: use_state(|| None),
			client_loader_version: use_state(|| VersionPattern::Any),
			server_loader_version: use_state(|| VersionPattern::Any),
			packages: use_state(TemplatePackageConfiguration::default),
			modpack: use_state(|| None),
			datapack_folder: use_state(|| None),
			java: use_state(|| None),
			plugin: use_state(|| None),
			plugin_config: use_state(ControlledConfig::default),
		};

		let out2 = out.clone();
		use_side_effect(move || {
			out2.id.read();
			out2.from.read();
			out2.name.read();
			out2.icon.read();
			out2.side.read();
			out2.version.read();
			out2.client_loader.read();
			out2.server_loader.read();
			out2.client_loader_version.read();
			out2.server_loader_version.read();
			out2.packages.read();
			out2.modpack.read();
			out2.datapack_folder.read();
			out2.java.read();
			out2.plugin.read();
			out2.plugin_config.read();

			out2.is_dirty.clone().set(out2.has_changed());
		});

		out
	}

	pub fn update(&mut self, id: Option<String>, config: TemplateConfig) {
		self.original_config
			.set_if_modified(PtrEq(Arc::new(config.clone())));

		if let Some(id) = id {
			self.id.set_if_modified(id.clone());
			self.original_id.set(id);
		}

		self.from.set_if_modified(config.instance.from.get_vec());

		let (loader, loader_version) = if let Some(loader) = config.client_loader() {
			(Some(loader.0), loader.1)
		} else {
			(None, VersionPattern::Any)
		};
		self.client_loader.set_if_modified(loader);
		self.client_loader_version.set_if_modified(loader_version);

		let (loader, loader_version) = if let Some(loader) = config.server_loader() {
			(Some(loader.0), loader.1)
		} else {
			(None, VersionPattern::Any)
		};
		self.server_loader.set_if_modified(loader);
		self.server_loader_version.set_if_modified(loader_version);

		self.name.set_if_modified(config.instance.name);
		self.icon.set_if_modified(config.instance.icon);
		self.side.set_if_modified(config.instance.side);
		self.version.set_if_modified(
			config
				.instance
				.version
				.map(|x| MinecraftVersion::from_deser(&x).to_string()),
		);
		self.packages.set(match self.ty {
			ConfigKind::Instance => TemplatePackageConfiguration::Simple(config.instance.packages),
			ConfigKind::Template | ConfigKind::BaseTemplate => config.packages,
		});
		self.modpack.set_if_modified(config.instance.modpack);

		self.datapack_folder
			.set_if_modified(config.instance.datapack_folder);
		self.java.set_if_modified(config.instance.launch.java);

		self.plugin
			.set_if_modified(config.instance.source_plugin.clone());
		self.plugin_config
			.write()
			.update(config.instance.plugin_config.clone());

		// Set default side for new instances
		if self.ty == ConfigKind::Instance && self.is_new {
			self.side.set_if_modified(Some(Side::Client));
		}

		self.is_dirty.set_if_modified(self.is_new);
		self.is_id_dirty.set_if_modified(!self.is_new);
	}

	pub fn apply(&mut self) -> Result<TemplateConfig, ConfigError> {
		let mut config = (*self.original_config.peek().0).clone();
		if self.id.peek().is_empty() && self.ty != ConfigKind::BaseTemplate {
			return Err(ConfigError::IdMissing);
		}
		let final_version = final_value_owned(
			self.version.peek().cloned(),
			&self.parent_configs.peek().0,
			|x| x.instance.version.as_ref().map(|x| to_string_json(&x)),
		);
		if final_version.is_none()
			&& self.ty != ConfigKind::Template
			&& self.ty != ConfigKind::BaseTemplate
		{
			return Err(ConfigError::VersionMissing);
		}

		config.instance.from = DeserListOrSingle::from_iter(self.from.peek().clone());
		config.instance.name = self.name.peek().clone();
		config.instance.icon = self.icon.peek().clone();
		config.instance.side = *self.side.peek();
		config.instance.version = match self.version.peek().as_deref() {
			None => None,
			Some("latest") => Some(MinecraftVersionDeser::Latest(
				MinecraftLatestVersion::Release,
			)),
			Some("latest_snapshot") => Some(MinecraftVersionDeser::Latest(
				MinecraftLatestVersion::Snapshot,
			)),
			Some(other) => Some(MinecraftVersionDeser::Version(other.into())),
		};

		match self.ty {
			ConfigKind::Instance => {
				let side = final_value_owned(
					self.side.peek().cloned(),
					&self.parent_configs.peek().0,
					|x| x.instance.side,
				);
				let Some(side) = side else {
					return Err(ConfigError::SideMissing);
				};
				config.instance.loader = match side {
					Side::Client => format_loader(
						self.client_loader.peek().as_ref(),
						&self.client_loader_version.peek(),
					),
					Side::Server => format_loader(
						self.server_loader.peek().as_ref(),
						&self.server_loader_version.peek(),
					),
				};
			}
			ConfigKind::Template | ConfigKind::BaseTemplate => {
				config.loader = TemplateLoaderConfiguration::default();
				let new_config = TemplateLoaderConfiguration::Full {
					client: format_loader(
						self.client_loader.peek().as_ref(),
						&self.client_loader_version.peek(),
					),
					server: format_loader(
						self.server_loader.peek().as_ref(),
						&self.server_loader_version.peek(),
					),
				};

				// This handles automatic simplification
				config.loader.merge(&new_config);
			}
		}

		config.instance.modpack = self.modpack.peek().clone();

		config.instance.datapack_folder = self.datapack_folder.peek().clone();
		config.instance.launch.java = self.java.peek().clone();

		match self.ty {
			ConfigKind::Instance => {
				config.instance.packages = self.packages.peek().iter_global().cloned().collect();
			}
			ConfigKind::Template | ConfigKind::BaseTemplate => {
				config.packages = self.packages.peek().cloned();
			}
		}

		config.instance.source_plugin = self.plugin.peek().clone();
		let mut plugin_config = self.plugin_config.peek().cloned();
		plugin_config.optimize();
		config.instance.plugin_config = plugin_config.into_data();

		self.is_dirty.set_if_modified(false);
		self.is_id_dirty.set_if_modified(false);

		Ok(config)
	}

	pub fn save_fn(
		&self,
		front_state: Shared<FrontState>,
		save_config: UseMutation<ToastedMutation<SaveConfig>>,
	) -> impl Fn() -> bool + 'static {
		let config_state = self.clone();
		move || {
			let config = match config_state.clone().apply() {
				Ok(config) => config,
				Err(e) => {
					let msg = match e {
						ConfigError::IdMissing => "ID is missing",
						ConfigError::SideMissing => "Side is missing",
						ConfigError::VersionMissing => "Version is missing",
					};
					front_state.write().toast(Toast::error(msg, None));
					return false;
				}
			};

			let id = if config_state.is_new {
				config_state.id.peek().clone()
			} else {
				config_state.original_id.peek().clone()
			};
			let item = ConfiguredItem {
				id: Some(id),
				ty: config_state.ty,
				is_new: config_state.is_new,
			};
			save_config.mutate(SaveConfigParams {
				item: item.clone(),
				config: NotEq(config),
			});

			true
		}
	}

	pub fn has_changed(&self) -> bool {
		if self.is_new {
			return true;
		}

		let Ok(new_config) = self.clone().apply() else {
			return false;
		};

		new_config != *self.original_config.peek().0 || *self.original_id.peek() != *self.id.peek()
	}
}

/// Thing that is being configured
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConfiguredItem {
	/// The ID of what is being configured.
	///
	/// If it is empty, then either we are creating a new instance / template, or we are configuring the base template.
	pub id: Option<String>,
	pub ty: ConfigKind,
	/// Whether this is a new item
	pub is_new: bool,
}

fn format_loader(loader: Option<&Loader>, version: &VersionPattern) -> Option<String> {
	loader.map(|loader| format_versioned_string(&to_string_json(loader), version))
}

pub enum ConfigError {
	IdMissing,
	SideMissing,
	VersionMissing,
}
