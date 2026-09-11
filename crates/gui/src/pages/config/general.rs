use std::rc::Rc;

use nitrolaunch::{
	config_crate::{ConfigKind, instance::make_valid_instance_id, template::TemplateConfig},
	shared::{Side, util::to_string_json, versions::VersionPattern},
};

use crate::{
	components::input::{
		Derivable, final_value_owned, icon::IconSelector, select::Selected, switch::Switch,
		text::TextInput,
	},
	ops::{
		instance::FetchItems,
		plugin_results::{FetchLoaderVersions, FetchSupportedLoaders},
		plugins::FetchConfigCreationPlugins,
		versions::FetchMinecraftVersions,
	},
	pages::{config::ConfigState, settings::plugins::get_plugin_icon},
	prelude::*,
	util::{PtrEq, assets::get_loader_icon},
};

#[derive(PartialEq)]
pub struct GeneralTab {
	pub config_state: ConfigState,
	pub parent_configs: PtrEq<[TemplateConfig]>,
}

impl Component for GeneralTab {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let mut include_snapshots = use_state(|| false);
		let minecraft_versions = use_query(FetchMinecraftVersions::new(
			back_state.clone(),
			*include_snapshots.read(),
		));
		let plugins_supporting_creation = use_query(Query::new(
			self.config_state.ty,
			FetchConfigCreationPlugins::new(back_state.clone()),
		));
		let items = use_query(FetchItems::new(back_state));

		let show_id_field =
			self.config_state.is_new && self.config_state.ty != ConfigKind::BaseTemplate;
		let show_name_field = self.config_state.ty != ConfigKind::BaseTemplate;

		let name = use_transform_optional_string(self.config_state.name);

		let mut id = self.config_state.id;
		let mut is_id_dirty = self.config_state.is_id_dirty;
		use_side_effect(move || {
			if !*is_id_dirty.peek() {
				id.set(make_valid_instance_id(&name.read()));
			}
		});

		let top_right = rect()
			.maybe(show_name_field, |this| {
				this.child(field(
					"Name",
					"font",
					&theme,
					TextInput::new(name).derived_value(
						self.config_state.name.read().as_ref(),
						&self.parent_configs.0,
						|x| x.instance.name.as_ref(),
					),
				))
			})
			.maybe(show_id_field, |this| {
				this.child(
					field(
						"ID",
						"hashtag",
						&theme,
						TextInput::new(self.config_state.id).on_change(move |_| {
							is_id_dirty.set(true);
						}),
					)
					.tip(&front_state, "A unique ID for this instance"),
				)
			});

		let top = rect()
			.width(Size::fill())
			.height(Size::px(158.0))
			.horizontal()
			.flex()
			.child(
				rect()
					.width(Size::px(158.0))
					.height(Size::fill())
					.center()
					.child(IconSelector {
						icon: self.config_state.icon,
						parent_configs: self.parent_configs.clone(),
					}),
			)
			.child(
				rect()
					.width(Size::flex(1.0))
					.height(Size::fill())
					.padding(Gaps::new(15.0, 15.0, 15.0, 0.0))
					.child(top_right),
			);

		let items = items.read();
		let items = items.state();
		let templates = items.ok().map(|x| {
			x.templates
				.iter()
				.map(|x| SelectOption::new(&x.id, x.name.as_deref().unwrap_or(&x.id), None))
		});
		let from = self.config_state.from;
		let from_field = Dropdown::new(
			Selected::Multi(from.read().clone()),
			Rc::new(move |selected| {
				from.clone().set(selected.multi());
			}),
		)
		.panel_colorway()
		.maybe(templates.is_some(), |this| {
			this.children(templates.unwrap())
		});
		let from_field = field("Parent Templates", "diagram", &theme, from_field).tip(
			&front_state,
			"Templates to derive default configuration from",
		);

		let plugins_supporting_creation = plugins_supporting_creation.read();
		let plugins_supporting_creation = plugins_supporting_creation.state();
		let plugins_supporting_creation = plugins_supporting_creation.ok().map(|x| {
			x.iter().map(|x| {
				SelectOption::new_custom_icon(
					x.id.clone(),
					x.meta.name.as_deref().unwrap_or(&x.id),
					get_plugin_icon(&x.id).into_element(),
				)
			})
		});
		let config_plugin = self.config_state.plugin;
		let config_plugin_field = Dropdown::new(
			Selected::Single(config_plugin.read().clone()),
			Rc::new(move |selected| {
				config_plugin
					.clone()
					.set(selected.single_optional().flatten());
			}),
		)
		.panel_colorway()
		.allow_none()
		.maybe(plugins_supporting_creation.is_some(), |this| {
			this.children(plugins_supporting_creation.unwrap())
		});
		let config_plugin_field = field("Config plugin", "jigsaw", &theme, config_plugin_field)
			.tip(
				&front_state,
				"Plugin to create this instance with. Check the plugin's documentation for more information.",
			);

		let config_settings = rect()
			.width(Size::fill())
			.cont()
			.child(segment(from_field, 1.0))
			.maybe(self.config_state.is_new, |this| {
				this.child(segment(config_plugin_field, 1.0))
			});

		let show_side_field = self.config_state.ty.is_template()
			|| (self.config_state.ty == ConfigKind::Instance && self.config_state.is_new);
		let show_none_option = self.config_state.ty.is_template();
		let side = self.config_state.side;
		let side_field = InlineSelect::new(
			Selected::Single(*side.read()),
			Rc::new(move |value| {
				side.clone().set(value.single_optional().flatten());
			}),
		)
		.derived_value_owned(
			(*self.config_state.side.read()).map(Some),
			&self.parent_configs.0,
			|x| x.instance.side.map(Some),
		)
		.maybe_child(show_none_option, || {
			SelectOption::new(None, "Inherit", Some("diagram"))
		})
		.child(
			SelectOption::new(Some(Side::Client), "Client", Some("controller"))
				.tip("Standard Minecraft game"),
		)
		.child(
			SelectOption::new(Some(Side::Server), "Server", Some("server"))
				.tip("Dedicated multiplayer server"),
		);

		let side_field = field("Side", "controller", &theme, side_field);

		let minecraft_versions = minecraft_versions
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		let minecraft_versions = minecraft_versions
			.into_iter()
			.rev()
			.map(|x| SelectOption::new(Some(x.clone()), &x, Some("tag")));
		let version = self.config_state.version;
		let version_selector = Dropdown::new(
			Selected::Single(self.config_state.version.read().cloned()),
			Rc::new(move |selected| {
				version.clone().set(selected.single_optional().flatten());
			}),
		)
		.panel_colorway()
		.derived_value_owned(
			self.config_state.version.peek().cloned().map(Some),
			&self.parent_configs.0,
			|x| {
				x.instance
					.version
					.as_ref()
					.map(|x| Some(to_string_json(&x)))
			},
		)
		.allow_inherit()
		.child(
			SelectOption::new(Some("latest".into()), "Latest", Some("star"))
				.tip("Always pick the latest stable version when the instance is updated"),
		)
		.maybe_child(*include_snapshots.read(), || {
			SelectOption::new(
				Some("latest_snapshot".into()),
				"Latest Snapshot",
				Some("star"),
			)
			.tip("Always pick the latest snapshot version when the instance is updated")
		})
		.children(minecraft_versions);

		let version_field = rect()
			.spacing(theme.gap3)
			.child(
				rect()
					.child(version_selector)
					.tip(&front_state, "The version of Minecraft to use"),
			)
			.child(
				rect()
					.cont()
					.main_align(Alignment::Start)
					.cross_align(Alignment::Center)
					.color(theme.fg3)
					.child("Include snapshots")
					.child(Switch {
						enabled: *include_snapshots.read(),
						on_toggle: EventHandler::from(move |_| {
							include_snapshots.toggle();
						}),
					})
					.tip(
						&front_state,
						"Whether to include pre-release versions in the dropdown",
					),
			);
		let version_field = field("Minecraft version", "tag", &theme, version_field);

		let loaders_config = LoadersConfig {
			config_state: self.config_state.clone(),
			parent_configs: self.parent_configs.clone(),
		};

		let datapack_folder =
			use_transform_optional_string(self.config_state.datapack_folder.clone());
		let datapack_folder = TextInput::new(datapack_folder).derived_value(
			self.config_state.datapack_folder.read().as_ref(),
			&self.parent_configs.0,
			|x| x.instance.datapack_folder.as_ref(),
		);
		let datapack_folder = field("Datapack folder", "folder", &theme, datapack_folder).tip(
			&front_state,
			"Folder to install global datapacks in. Relative to the instance root.",
		);

		let main = rect()
			.width(Size::fill())
			.padding(15.0)
			.maybe(self.config_state.ty != ConfigKind::BaseTemplate, |this| {
				this.child(config_settings)
			})
			.maybe(show_side_field, |this| this.child(side_field))
			.child(version_field)
			.child(loaders_config)
			.child(datapack_folder);

		let main = ScrollView::new()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.child(main);

		rect().expanded().flex().child(top).child(main)
	}
}

#[derive(PartialEq)]
struct LoadersConfig {
	config_state: ConfigState,
	parent_configs: PtrEq<[TemplateConfig]>,
}

impl Component for LoadersConfig {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let supported_loaders = use_query(FetchSupportedLoaders::new(back_state.clone()));
		let supported_loaders = supported_loaders
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		let minecraft_version = final_value_owned(
			self.config_state.version.read().cloned(),
			&self.parent_configs.0,
			|x| x.instance.version.as_ref().map(to_string_json),
		);
		let client_loader_versions = use_query(FetchLoaderVersions::new(
			back_state.clone(),
			self.config_state
				.client_loader
				.read()
				.cloned()
				.unwrap_or_default(),
			minecraft_version.clone(),
		));
		let server_loader_versions = use_query(FetchLoaderVersions::new(
			back_state,
			self.config_state
				.server_loader
				.read()
				.cloned()
				.unwrap_or_default(),
			minecraft_version.clone(),
		));

		let client_options = supported_loaders.iter().filter(|x| x.is_client()).map(|x| {
			SelectOption::new_custom_icon(
				Some(x.clone()),
				&x.to_string(),
				get_loader_icon(x).into_element(),
			)
		});
		let server_options = supported_loaders.iter().filter(|x| x.is_server()).map(|x| {
			SelectOption::new_custom_icon(
				Some(x.clone()),
				&x.to_string(),
				get_loader_icon(x).into_element(),
			)
		});

		let client_loader = self.config_state.client_loader;
		let client_field = InlineSelect::new(
			Selected::Single(self.config_state.client_loader.read().clone()),
			Rc::new(move |selected| {
				client_loader
					.clone()
					.set(selected.single_optional().flatten());
			}),
		)
		.grid(4)
		.allow_inherit()
		.derived_value_owned(
			self.config_state.client_loader.read().clone().map(Some),
			&self.parent_configs.0,
			|x| x.client_loader().map(|x| Some(x.0)),
		)
		.children(client_options);
		let field_name = if self.config_state.ty == ConfigKind::Instance {
			"Loader"
		} else {
			"Client loader"
		};
		let client_field = field(field_name, "controller", &theme, client_field).tip(
			&front_state,
			if self.config_state.ty == ConfigKind::Instance {
				"What to install for loading mods"
			} else {
				"Loader for client instances under this template"
			},
		);
		let show_client_fields = self.config_state.ty.is_template()
			|| *self.config_state.side.read() == Some(Side::Client);

		let server_loader = self.config_state.server_loader;
		let server_field = InlineSelect::new(
			Selected::Single(server_loader.read().clone()),
			Rc::new(move |selected| {
				server_loader
					.clone()
					.set(selected.single_optional().flatten());
			}),
		)
		.grid(4)
		.allow_inherit()
		.derived_value_owned(
			self.config_state.server_loader.read().clone().map(Some),
			&self.parent_configs.0,
			|x| x.server_loader().map(|x| Some(x.0)),
		)
		.children(server_options);
		let field_name = if self.config_state.ty == ConfigKind::Instance {
			"Loader"
		} else {
			"Server loader"
		};
		let server_field = field(field_name, "server", &theme, server_field).tip(
			&front_state,
			if self.config_state.ty == ConfigKind::Instance {
				"What to install for loading mods or plugins"
			} else {
				"Loader for server instances under this template"
			},
		);
		let show_server_fields = self.config_state.ty.is_template()
			|| *self.config_state.side.read() == Some(Side::Server);

		let client_loader_versions = client_loader_versions
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		let options = client_loader_versions
			.into_iter()
			.map(|x| SelectOption::simple_or_none(Some(x)));
		let client_version = self.config_state.client_loader_version;
		let client_version_field = Dropdown::new(
			Selected::Single(
				self.config_state
					.client_loader_version
					.read()
					.optional()
					.map(|x| x.to_string()),
			),
			Rc::new(move |selected| {
				client_version.clone().set(
					selected
						.single_optional()
						.flatten()
						.map(|x| VersionPattern::from(&x))
						.unwrap_or_default(),
				);
			}),
		)
		.panel_colorway()
		.allow_inherit()
		.children(options);
		let field_name = if self.config_state.ty == ConfigKind::Instance {
			"Loader version"
		} else {
			"Client loader version"
		};
		let client_version_field = field(field_name, "tag", &theme, client_version_field);

		let server_loader_versions = server_loader_versions
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		let options = server_loader_versions
			.into_iter()
			.map(|x| SelectOption::simple_or_none(Some(x)));
		let server_version = self.config_state.server_loader_version;
		let server_version_field = Dropdown::new(
			Selected::Single(
				self.config_state
					.server_loader_version
					.read()
					.optional()
					.map(|x| x.to_string()),
			),
			Rc::new(move |selected| {
				server_version.clone().set(
					selected
						.single_optional()
						.flatten()
						.map(|x| VersionPattern::from(&x))
						.unwrap_or_default(),
				);
			}),
		)
		.panel_colorway()
		.allow_inherit()
		.children(options);
		let field_name = if self.config_state.ty == ConfigKind::Instance {
			"Loader version"
		} else {
			"Server loader version"
		};
		let server_version_field = field(field_name, "tag", &theme, server_version_field);

		rect()
			.width(Size::fill())
			.maybe(show_client_fields, |this| {
				this.child(client_field)
					.maybe(self.config_state.client_loader.read().is_some(), |this| {
						this.child(client_version_field)
					})
			})
			.maybe(show_server_fields, |this| {
				this.child(server_field)
					.maybe(self.config_state.server_loader.read().is_some(), |this| {
						this.child(server_version_field)
					})
			})
	}
}
