use std::{path::PathBuf, rc::Rc};

use freya::query::UseMutation;
use nitrolaunch::shared::Side;

use crate::{
	components::{
		dialog::modal::{Modal, ModalButton},
		input::{file::FileSelector, select::Selected, text::TextInput},
	},
	ops::transfer::{
		CheckMigration, ExportInstance, ExportInstanceKeys, FetchTransferFormats, ImportInstance,
		ImportInstanceKeys, MigrateInstances, MigrateInstancesKeys,
	},
	prelude::*,
	state::FrontState,
	util::Shared,
};

#[derive(PartialEq)]
pub struct InstanceTransferModal {
	mode: InstanceTransferMode,
	exporting_instance: Option<String>,
}

impl InstanceTransferModal {
	pub fn import() -> Self {
		Self {
			mode: InstanceTransferMode::Import,
			exporting_instance: None,
		}
	}

	pub fn export(instance_id: String) -> Self {
		Self {
			mode: InstanceTransferMode::Export,
			exporting_instance: Some(instance_id),
		}
	}
}

impl Component for InstanceTransferModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let formats = use_query(Query::new(
			(),
			FetchTransferFormats::new(back_state.clone()),
		));
		let import_mutation = use_mutation(Mutation::new(ImportInstance::new(back_state.clone())));
		let export_mutation = use_mutation(Mutation::new(ExportInstance::new(back_state.clone())));

		let format = use_state::<Option<String>>(|| None);
		let source_path = use_state::<Option<PathBuf>>(|| None);
		let new_id = use_state(String::new);
		let import_side = use_state(|| Side::Client);

		let default = Vec::new();
		let formats = formats.read();
		let formats = formats.state();
		let formats = formats.ok().unwrap_or(&default);

		let selected_format = formats
			.iter()
			.find(|x| Some(&x.id) == format.read().as_ref());

		let formats: Vec<_> = formats
			.iter()
			.filter(|x| {
				self.mode == InstanceTransferMode::Import && x.import.is_some()
					|| self.mode == InstanceTransferMode::Export && x.export.is_some()
			})
			.collect();

		let contents = if formats.is_empty() {
			rect().expanded().child(placeholder(
				"No formats available. Please install some from plugins.",
				&theme,
			))
		} else {
			let format2 = format;
			let format_selector = Dropdown::new(
				Selected::Single(format.read().cloned()),
				Rc::new(move |selected| {
					format2.clone().set(selected.single());
				}),
			)
			.panel_colorway()
			.allow_none()
			.children(
				formats
					.into_iter()
					.map(|x| SelectOption::new(Some(x.id.clone()), &x.name, Some("box"))),
			);
			let format_selector = field("Format", "curly_braces", &theme, format_selector);

			let path_selector = if self.mode == InstanceTransferMode::Import {
				FileSelector::select(source_path)
			} else {
				FileSelector::save(source_path)
			};
			let title = if self.mode == InstanceTransferMode::Import {
				"File"
			} else {
				"Save As"
			};
			let path_selector = field(title, "folder", &theme, path_selector);

			let new_id_field = TextInput::new(new_id);
			let new_id_field = field("New ID", "hashtag", &theme, new_id_field)
				.tip(&front_state, "The ID for the new instance");

			let import_side2 = import_side;
			let side_selector = InlineSelect::new(
				Selected::Single(*import_side.read()),
				Rc::new(move |selected| {
					import_side2.clone().set(selected.single());
				}),
			)
			.child(SelectOption::new(
				Side::Client,
				"Client",
				Some("controller"),
			))
			.child(SelectOption::new(Side::Server, "Server", Some("server")));
			let side_selector = field("Side", "controller", &theme, side_selector);

			let show_side_selector = self.mode == InstanceTransferMode::Import
				&& source_path.read().is_some()
				&& selected_format.is_some_and(|x| x.needs_import_side);

			rect()
				.width(Size::fill())
				.padding(theme.gap3)
				.child(format_selector)
				.child(path_selector)
				.maybe(
					self.mode == InstanceTransferMode::Import && source_path.read().is_some(),
					|this| this.child(new_id_field),
				)
				.maybe(show_side_selector, |this| this.child(side_selector))
		};

		let front_state2 = front_state.clone();
		let exporting_instance = self.exporting_instance.clone();
		let mode = self.mode.clone();
		let on_submit = move |_: ()| {
			let Some(format) = format.read().clone() else {
				front_state2
					.write()
					.toast(Toast::error("A format must be selected", None));
				return;
			};
			let Some(source_path) = source_path.read().clone() else {
				front_state2
					.write()
					.toast(Toast::error("A file must be selected", None));
				return;
			};
			let new_id = new_id.read().clone();
			if new_id.is_empty() && mode == InstanceTransferMode::Import {
				front_state2
					.write()
					.toast(Toast::error("An instance ID must be provided", None));
				return;
			}
			let import_side = *import_side.read();

			let mode = mode.clone();
			let front_state2 = front_state2.clone();
			let exporting_instance = exporting_instance.clone();
			spawn_forever(async move {
				match mode {
					InstanceTransferMode::Import => {
						import_mutation
							.mutate_async(ImportInstanceKeys {
								format,
								path: source_path,
								id: new_id,
								side: Some(import_side),
							})
							.await;
					}
					InstanceTransferMode::Export => {
						export_mutation
							.mutate_async(ExportInstanceKeys {
								format,
								path: source_path,
								id: exporting_instance
									.clone()
									.expect("Exporting instance ID must be provided"),
							})
							.await;
					}
				}
				front_state2.write().set_modal(None);
			});
		};
		let submit_title = match self.mode {
			InstanceTransferMode::Import => "Import",
			InstanceTransferMode::Export => "Export",
		};

		let title = match self.mode {
			InstanceTransferMode::Import => "Import Instance",
			InstanceTransferMode::Export => "Export Instance",
		};
		let title_icon = match self.mode {
			InstanceTransferMode::Import => "download",
			InstanceTransferMode::Export => "popout",
		};

		let front_state2 = front_state.clone();
		Modal::new(title.into(), title_icon.into())
			.maybe_child(true, || contents)
			.on_close(move |_| {
				front_state2.write().set_modal(None);
			})
			.cancel_button()
			.button(ModalButton {
				title: submit_title.into(),
				icon: title_icon.into(),
				on_click: on_submit.into(),
				active: true,
			})
	}
}

#[derive(PartialEq, Clone)]
pub enum InstanceTransferMode {
	Import,
	Export,
}

#[derive(PartialEq)]
pub struct MigrateModal;

impl Component for MigrateModal {
	fn render(&self) -> impl IntoElement {
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let migrate_mutation =
			use_mutation(Mutation::new(MigrateInstances::new(back_state.clone())));

		let format = use_state::<Option<String>>(|| None);
		let link = use_state(|| false);
		let instances = use_state(Vec::new);

		let contents = MigrateContents {
			format,
			link,
			instances,
		};

		let on_submit = on_migrate(
			front_state.clone(),
			migrate_mutation,
			format,
			link,
			instances,
		);

		let front_state2 = front_state.clone();
		Modal::new("Migrate Instances".into(), "shuffle".into())
			.size_large()
			.maybe_child(true, || contents)
			.on_close(move |_| {
				front_state2.write().set_modal(None);
			})
			.cancel_button()
			.button(ModalButton {
				title: "Migrate".into(),
				icon: "cycle".into(),
				on_click: on_submit.into(),
				active: true,
			})
	}
}

#[derive(PartialEq)]
pub struct MigrateContents {
	pub format: State<Option<String>>,
	pub link: State<bool>,
	pub instances: State<Vec<String>>,
}

impl Component for MigrateContents {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let formats = use_query(Query::new(
			(),
			FetchTransferFormats::new(back_state.clone()),
		));
		let check_migration = use_query(
			Query::new(
				self.format.read().clone().unwrap_or_default(),
				CheckMigration::new(back_state.clone()),
			)
			.enable(self.format.read().is_some()),
		);

		let formats2 = formats;
		let mut format2 = self.format;
		use_side_effect(move || {
			let formats = formats2.read();
			let formats = formats.state();
			let formats = formats.ok();
			if let Some(formats) = formats {
				format2.set(
					formats
						.iter()
						.find(|x| x.migrate.is_some())
						.map(|x| x.id.clone()),
				);
			}
		});

		let default = Vec::new();
		let formats = formats.read();
		let formats = formats.state();
		let formats = formats.ok().unwrap_or(&default);

		let formats: Vec<_> = formats.iter().filter(|x| x.migrate.is_some()).collect();

		if formats.is_empty() {
			rect()
				.expanded()
				.child(placeholder(
					"No launcher formats available. Please install some from plugins.",
					&theme,
				))
				.into_element()
		} else {
			let format2 = self.format;
			let format_selector = Dropdown::new(
				Selected::Single(self.format.read().cloned()),
				Rc::new(move |selected| {
					format2.clone().set(selected.single());
				}),
			)
			.panel_colorway()
			.maybe_child(formats.is_empty() || self.format.read().is_none(), || {
				SelectOption::none()
			})
			.children(
				formats
					.into_iter()
					.map(|x| SelectOption::new(Some(x.id.clone()), &x.name, Some("box"))),
			);
			let format_selector = field("Launcher", "star", &theme, format_selector);

			let more_options = match &*check_migration.read().state() {
				QueryStateData::Pending => rect()
					.width(Size::fill())
					.child(placeholder("Please select a launcher", &theme)),
				QueryStateData::Loading { .. } => rect()
					.width(Size::fill())
					.child(placeholder("Checking for launcher...", &theme)),
				QueryStateData::Settled { res: Err(e), .. } => rect().width(Size::fill()).child(
					placeholder(&format!("Failed to check for launcher: {e}"), &theme),
				),
				QueryStateData::Settled { res: Ok(res), .. } if res.instances.is_empty() => rect()
					.width(Size::fill())
					.child(placeholder("No instances found to migrate", &theme)),
				QueryStateData::Settled { res: Ok(res), .. } => {
					let instances2 = self.instances;
					let instance_selector = InlineSelect::new(
						Selected::Multi(self.instances.read().clone()),
						Rc::new(move |selected| {
							instances2.clone().set(selected.multi());
						}),
					)
					.children(
						res.instances
							.iter()
							.map(|x| SelectOption::simple(x.clone())),
					)
					.grid(2);
					let instance_selector = field("Instances", "honeycomb", &theme, instance_selector)
                        .tip(&front_state, "Instances to migrate from the launcher. If none are selected, all instances will be migrated.");

					let link2 = self.link;
					let mode_selector = InlineSelect::new(
						Selected::Single(*self.link.read()),
						Rc::new(move |selected| {
							link2.clone().set(selected.single());
						}),
					)
					.child(
						SelectOption::new(false, "Copy", Some("copy"))
							.tip("Unique copies of the launcher's instances will be created"),
					).child(
                        SelectOption::new(true, "Link", Some("link"))
                            .tip("The launcher's instances will be linked to this launcher. Changes to the launcher's instances will affect this launcher, and vice versa."),
                    );
					let mode_selector = field("Mode", "link", &theme, mode_selector);

					rect().child(instance_selector).child(mode_selector)
				}
			};

			let contents = rect()
				.width(Size::fill())
				.padding(theme.gap3)
				.child(format_selector)
				.child(more_options);

			ScrollView::new().expanded().child(contents).into_element()
		}
	}
}

pub fn on_migrate(
	front_state: Shared<FrontState>,
	mutation: UseMutation<MigrateInstances>,
	format: State<Option<String>>,
	link: State<bool>,
	instances: State<Vec<String>>,
) -> impl FnMut(()) {
	move |_: ()| {
		let Some(format) = format.read().clone() else {
			front_state
				.write()
				.toast(Toast::error("A format must be selected", None));
			return;
		};
		let link = *link.read();
		let instances = instances.read().clone();

		mutation.mutate(MigrateInstancesKeys {
			format,
			link,
			instances,
		});
	}
}
