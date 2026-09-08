use freya::radio::use_radio;
use nitrolaunch::shared::output::NitroOutput;

use crate::{
	components::{
		account::auth::MicrosoftAuthPrompt,
		dialog::{custom_popup::CustomPopupModal, modal::Modal},
		instance::transfer::{InstanceTransferModal, InstanceTransferMode, MigrateModal},
		pkg::{diffs::PackageDiffsModal, manual::ManualFilesModal},
	},
	ops::instance::{DeleteInstance, DeleteTemplate},
	pages::{config::ConfigPage, onboarding::OnboardingModal, settings::SettingsPage},
	prelude::*,
	routing::Page,
	state::{BackEvent, ModalType, use_launcher_data},
	theme::ThemeDeser,
	util::PtrEq,
};

/// Global event listeners and modals
#[derive(PartialEq)]
pub struct Global;

impl Component for Global {
	fn render(&self) -> impl IntoElement {
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let data = use_launcher_data();

		front_state.read().subscribe(FrontChannel::Modal);

		let delete_instance_mutation = use_mutation(Mutation::new(
			DeleteInstance::new(back_state.clone()).toast(
				&back_state,
				Some("Instance deleted"),
				"Failed to delete instance",
			),
		));
		let delete_template_mutation = use_mutation(Mutation::new(
			DeleteTemplate::new(back_state.clone()).toast(
				&back_state,
				Some("Template deleted"),
				"Failed to delete template",
			),
		));

		let front_state2 = front_state.clone();
		let back_state2 = back_state.clone();
		let radio = use_radio(FrontChannel::ThemeConfig);
		use_side_effect(move || {
			radio.read();
			let data = back_state2.data();
			let available_themes = back_state2.themes();
			back_state2.output().debug("Applying theme".into());

			let mut theme = ThemeDeser::dark();
			for new_theme in data.base_theme.into_iter().chain(data.overlay_themes) {
				if new_theme == "light" {
					theme = theme.merge(ThemeDeser::light());
				} else if let Some(data) = available_themes.iter().find(|x| x.id == new_theme)
					&& let Ok(new_theme) = serde_json::from_str::<ThemeDeser>(&data.settings)
				{
					theme = theme.merge(new_theme);
				}
			}
			front_state2.write().set_theme(theme.into());
			back_state2.output().debug("Theme applied".into());
		});
		let radio = use_radio(FrontChannel::Zoom);
		use_side_effect(move || {
			radio.read();
			let data = back_state.data();
			let platform = Platform::get();
			platform.set_custom_scale_factor(data.zoom);
		});

		let is_onboarding = matches!(front_state.read().modal(), Some(ModalType::Onboarding))
			|| !data.data.read().launcher_opened_before;

		let front_state2 = front_state.clone();
		use_future(move || {
			let front_state2 = front_state2.clone();
			async move {
				let mut event_rx = front_state2.read().subscribe_events();
				while let Ok(event) = event_rx.recv().await {
					match event {
						BackEvent::ShowAuthPrompt { url, device_code } => {
							if !is_onboarding {
								front_state2
									.write()
									.set_modal(Some(ModalType::MicrosoftAuth { url, device_code }))
							}
						}
						BackEvent::CloseAuthPrompt => {
							// Prevent double borrow
							let should_close = matches!(
								front_state2.read().modal(),
								Some(ModalType::MicrosoftAuth { .. })
							);
							if should_close {
								front_state2.write().set_modal(None);
								front_state2
									.write()
									.toast(Toast::success("Account logged in"));
							}
						}
						BackEvent::ShowPackageDiffsPrompt { diffs } => {
							front_state2
								.write()
								.set_modal(Some(ModalType::PackageDiffs(diffs)));
						}
						BackEvent::ShowManualFilesPrompt { files } => {
							front_state2
								.write()
								.set_modal(Some(ModalType::ManualFiles(files)));
						}
						BackEvent::InvalidateData => {
							front_state2.write().invalidate(FrontChannel::Data);
						}
						_ => {}
					}
				}
			}
		});

		let front_state2 = front_state.clone();
		use_future(move || {
			let front_state2 = front_state2.clone();
			async move {
				let mut event_rx = front_state2.read().subscribe_events();
				while let Ok(event) = event_rx.recv().await {
					if let BackEvent::Invalidate(dependency) = event {
						dependency.invalidate();
					}
				}
			}
		});

		let front_state2 = front_state.clone();
		let front_state3 = front_state.clone();
		let front_state4 = front_state.clone();
		let simple_modal = match front_state.read().modal() {
			Some(modal) => match modal {
				ModalType::DeleteInstance(id) => {
					let id = id.clone();
					Some(
						Modal::simple_confirm(
							"Delete instance forever",
							"trash",
							rect().expanded().center().child(
								"Are you sure you want to delete this instance and all its files? This action cannot be undone.",
							),
							true,
							move |_| front_state3.write().set_modal(None),
							move |_| {
								delete_instance_mutation.mutate(id.clone());
								front_state2.write().set_modal(None);
								front_state2.write().navigate(Page::Home);
							},
						)
						.into_element(),
					)
				}
				ModalType::DeleteTemplate(id) => {
					let id = id.clone();
					Some(
						Modal::simple_confirm(
							"Delete template forever",
							"trash",
							rect().expanded().center().child(
								"Are you sure you want to delete this template? This action cannot be undone. Make sure that no instances are using this template before deleting it.",
							),
							true,
							move |_| front_state3.write().set_modal(None),
							move |_| {
								delete_template_mutation.mutate(id.clone());
								front_state2.write().set_modal(None);
							},
						)
						.into_element(),
					)
				}
				ModalType::PackageDiffs(diffs) => Some(
					PackageDiffsModal {
						diffs: PtrEq(diffs.clone()),
					}
					.into_element(),
				),
				ModalType::ManualFiles(files) => Some(
					ManualFilesModal {
						files: files.clone(),
					}
					.into_element(),
				),
				ModalType::MicrosoftAuth { url, device_code } => Some(
					MicrosoftAuthPrompt::new(url.clone(), device_code.clone(), move |_| {
						front_state4.write().set_modal(None);
					})
					.into_element(),
				),
				ModalType::Transfer(mode, exporting_id) => Some(match mode {
					InstanceTransferMode::Import => InstanceTransferModal::import().into_element(),
					InstanceTransferMode::Export => {
						InstanceTransferModal::export(exporting_id.clone().unwrap_or_default())
							.into_element()
					}
				}),
				ModalType::Migrate => Some(MigrateModal.into_element()),
				ModalType::CustomPopup(popup) => Some(
					CustomPopupModal {
						popup: popup.clone(),
					}
					.into_element(),
				),
				_ => None,
			},
			None => None,
		};

		let simple_modal2 = if is_onboarding {
			Some(OnboardingModal.into_element())
		} else {
			None
		};

		rect()
			.child(ConfigPage)
			.child(SettingsPage)
			.maybe_child(simple_modal)
			.maybe_child(simple_modal2)
	}
}
