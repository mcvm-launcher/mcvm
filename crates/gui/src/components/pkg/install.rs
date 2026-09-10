use nitrolaunch::{
	config_crate::{ConfigKind, instance::make_valid_instance_id},
	pkg_crate::{metadata::PackageMetadata, properties::PackageProperties},
	shared::{
		pkg::{ArcPkgReq, PackageKind},
		versions::VersionPattern,
	},
};

use crate::{
	components::{
		dialog::modal::{MODAL_MEDIUM_HEIGHT, MODAL_MEDIUM_WIDTH, Modal, ModalButton},
		input::{tabs::TopTabs, text::TextInput},
	},
	ops::{
		instance::{FetchItems, InstanceItemInfo, InstancesAndTemplates},
		packages::{
			CheckPackageCompatability, CheckPackageCompatabilityKeys, InstallPackage,
			PackageCompatabilityError, PackageInstallLocation,
		},
	},
	pages::config::ConfiguredItem,
	prelude::*,
	util::{PtrEq, assets::get_instance_icon},
};

#[derive(PartialEq)]
pub struct PackageInstallModal {
	pub req: ArcPkgReq,
	pub meta: PtrEq<PackageMetadata>,
	pub props: PtrEq<PackageProperties>,
	pub on_close: EventHandler<()>,
}

impl Component for PackageInstallModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let install_package = use_mutation(Mutation::new(
			InstallPackage::new(back_state.clone()).toast(
				&back_state,
				Some("Package installed"),
				"Failed to install package",
			),
		));
		let items_query = use_query(FetchItems::new(back_state.clone()));

		let is_modpack = self.props.0.kinds.contains(&PackageKind::Modpack);

		let tab = use_state(|| {
			if is_modpack {
				Tab::ModpackInstance
			} else {
				Tab::Instance
			}
		});
		let selected_item = use_state::<Option<ConfiguredItem>>(|| None);
		let new_instance_id = use_state(String::new);
		let mut new_instance_id2 = new_instance_id.clone();
		use_side_effect(move || {
			let new_value = make_valid_instance_id(&*new_instance_id2.read());
			new_instance_id2.set_if_modified(new_value);
		});

		let enable = selected_item.read().is_some();
		let compatability_check = use_query(
			Query::new(
				CheckPackageCompatabilityKeys {
					item: selected_item.read().cloned().unwrap_or(ConfiguredItem {
						ty: ConfigKind::BaseTemplate,
						id: None,
						is_new: false,
					}),
					package: self.req.clone(),
				},
				CheckPackageCompatability::new(back_state.clone()),
			)
			.enable(enable),
		);
		let compatability_err = compatability_check.read().state().ok().cloned().flatten();

		let tab2 = tab;
		let mut selected_item2 = selected_item;
		use_side_effect(move || {
			tab2.read();
			selected_item2.set(None);
		});

		let name = self
			.meta
			.0
			.name
			.clone()
			.unwrap_or(self.req.to_string_no_version());

		let tabs = TopTabs::from_state(tab).children(
			Tab::get_tabs(is_modpack)
				.iter()
				.map(|x| SelectOption::new(x.clone(), x.name(is_modpack), Some(x.icon()))),
		);

		let version_indicator = if let VersionPattern::Single(version) = &self.req.content_version {
			format!("Installing version {version}")
		} else {
			"Installing best version".into()
		};
		let version_indicator = rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.padding(theme.gap2 * 2.0)
			.horizontal()
			.spacing(theme.gap2)
			.cross_align(Alignment::Center)
			.color(theme.fg2)
			.border(border_bottom(theme.border, theme.panel_border))
			.child(icon("tag", 12.0))
			.child(version_indicator);

		let default_items = InstancesAndTemplates::default();
		let items = items_query.read();
		let items = items.state();
		let items = items.ok().unwrap_or(&default_items);

		let tab_contents = match &*tab.read() {
			Tab::Instance => {
				let out = grid(
					3,
					items.instances.iter().map(|x| Item {
						item: x.clone(),
						selected: selected_item,
					}),
				)
				.gap(theme.gap2);

				ScrollView::new().expanded().child(out).into_element()
			}
			Tab::Template => {
				let out = grid(
					3,
					items.templates.iter().map(|x| Item {
						item: x.clone(),
						selected: selected_item,
					}),
				)
				.gap(theme.gap2);

				ScrollView::new().expanded().child(out).into_element()
			}
			Tab::ModpackInstance => rect()
				.width(Size::fill())
				.padding(theme.gap2)
				.child(field(
					"ID for new instance",
					"hashtag",
					&theme,
					TextInput::new(new_instance_id),
				))
				.into_element(),
		};

		let base_error = rect()
			.width(Size::fill())
			.position(Position::new_absolute().bottom(theme.gap))
			.margin((0.0, theme.gap2))
			.layer(Layer::Relative(12))
			.padding(theme.gap2)
			.cont()
			.main_align(Alignment::Start)
			.cross_align(Alignment::Center)
			.corner_radius(theme.round);
		let error = if let Some(error) = &compatability_err {
			let message = match error {
				PackageCompatabilityError::WrongMinecraftVersion => {
					"Package does not support this Minecraft version"
				}
				PackageCompatabilityError::WrongLoader => "Package does not support this loader",
			};
			Some(
				base_error
					.color(theme.warning)
					.background(theme.error_bg)
					.border(theme.border(theme.warning))
					.child(icon("warning", 16.0))
					.child(message),
			)
		} else if compatability_check.read().state().is_loading() {
			Some(
				base_error
					.background(theme.panel)
					.border(theme.border(theme.panel_border))
					.child(CircularLoader::new().size(16.0))
					.child("Checking package compatibility..."),
			)
		} else {
			None
		};

		let contents = rect()
			.flex()
			.child(tabs)
			.child(version_indicator)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(tab_contents),
			)
			.maybe_child(error);

		let is_save_ready = match &*tab.read() {
			Tab::ModpackInstance => !new_instance_id.read().is_empty(),
			_ => selected_item.read().is_some() && compatability_err.is_none(),
		};

		let req = self.req.clone();
		let tab = tab;
		let selected_item = selected_item;
		let new_instance_id = new_instance_id;
		let on_close = self.on_close.clone();
		Modal::new(format!("Install {name}"), "download".into())
			.size(MODAL_MEDIUM_WIDTH, MODAL_MEDIUM_HEIGHT)
			.maybe_child(true, || contents)
			.on_close(self.on_close.clone())
			.cancel_button()
			.button(ModalButton {
				title: "Install".into(),
				icon: "download".into(),
				on_click: (move |_| {
					let location = match &*tab.read() {
						Tab::Instance => {
							let Some(selected) = selected_item.read().clone() else {
								return;
							};
							if is_modpack {
								PackageInstallLocation::InstanceModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Instance(selected.id.unwrap().into())
							}
						}
						Tab::Template => {
							let Some(selected) = selected_item.read().clone() else {
								return;
							};
							if selected.ty == ConfigKind::BaseTemplate {
								PackageInstallLocation::BaseTemplate(None)
							} else if is_modpack {
								PackageInstallLocation::TemplateModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Template(selected.id.unwrap().into(), None)
							}
						}
						Tab::ModpackInstance => PackageInstallLocation::NewInstanceModpack(
							new_instance_id.read().clone().into(),
						),
					};

					let on_close = on_close.clone();
					let req = req.clone();
					spawn_forever(async move {
						install_package.mutate_async((req.clone(), location)).await;
						on_close.call(());
					});
				})
				.into(),
				active: is_save_ready,
			})
	}
}

#[derive(PartialEq)]
struct Item {
	item: InstanceItemInfo,
	selected: State<Option<ConfiguredItem>>,
}

impl Component for Item {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);

		let is_selected = self.selected.read().as_ref() == Some(&self.item.get_config_item());

		let mut selected = self.selected;

		let inst_icon = if self.item.icon.is_none() {
			icon("box_dot", 28.0).into_element()
		} else {
			let inst_icon = get_instance_icon(self.item.icon.as_deref());
			ImageViewer::new(inst_icon)
				.width(Size::px(28.0))
				.height(Size::px(28.0))
				.into_element()
		};
		let inst_icon = rect()
			.width(Size::px(48.0))
			.height(Size::px(48.0))
			.center()
			.child(inst_icon);

		let item = self.item.get_config_item();
		rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.panel_colorway(&theme, *is_hovered.read(), is_selected)
			.hover(is_hovered)
			.corner_radius(theme.round)
			.on_press(move |_| {
				selected.set(Some(item.clone()));
			})
			.clickable()
			.cont()
			.child(inst_icon)
			.child(
				segment(self.item.name(), 1.0)
					.height(Size::fill())
					.main_align(Alignment::Center),
			)
	}
}

#[derive(PartialEq, Clone)]
enum Tab {
	Instance,
	Template,
	ModpackInstance,
}

impl Tab {
	fn get_tabs(is_modpack: bool) -> &'static [Tab] {
		if is_modpack {
			&[Self::ModpackInstance, Self::Instance, Self::Template]
		} else {
			&[Self::Instance, Self::Template]
		}
	}

	fn name(&self, is_modpack: bool) -> &str {
		match self {
			Self::Instance if is_modpack => "Existing Instance",
			Self::Instance => "Instance",
			Self::Template if is_modpack => "Existing Template",
			Self::Template => "Template",
			Self::ModpackInstance => "New Instance",
		}
	}

	fn icon(&self) -> &'static str {
		match self {
			Self::Instance => "box_dot",
			Self::Template => "diagram",
			Self::ModpackInstance => "minecraft",
		}
	}
}
