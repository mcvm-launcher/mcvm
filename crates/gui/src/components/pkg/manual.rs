use std::time::Duration;

use nitrolaunch::shared::{
	manual_files::{self, ManualFile},
	util::{OS_STRING, open_link},
};

use crate::{
	components::{dialog::modal::Modal, pkg::PackageChip},
	prelude::*,
	state::{BackEvent, FrontState},
	util::{PtrEq, Shared},
};

#[derive(PartialEq)]
pub struct ManualFilesModal {
	pub files: PtrEq<[ManualFile]>,
}

impl Component for ManualFilesModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let found_files = use_state(|| Vec::new());

		let files = self.files.clone();
		let found_files2 = found_files.clone();
		let front_state2 = front_state.clone();
		let back_state2 = back_state.clone();
		use_future(move || {
			let files = files.clone();
			let mut found_files2 = found_files2.clone();
			let front_state2 = front_state2.clone();
			let back_state2 = back_state2.clone();
			async move {
				let manual_dir = manual_files::get_scan_dir_from_os(OS_STRING);
				loop {
					let scanned = manual_files::scan(&manual_dir, &files.0);
					if scanned.len() == files.0.len() {
						front_state2.write().set_modal(None);
						let _ = back_state2
							.event_tx
							.send(BackEvent::ConfirmYesNoPrompt { yes: true });
						break;
					}

					found_files2.set(scanned);
					tokio::time::sleep(Duration::from_millis(250)).await;
				}
			}
		});

		let files =
			ScrollView::new()
				.expanded()
				.spacing(theme.gap)
				.children(self.files.0.iter().map(|x| {
					file(
						x,
						found_files.read().contains(&x.filename),
						&front_state,
						&theme,
					)
					.into_element()
				}));

		let files2 = self.files.clone();
		let top = rect()
			.width(Size::fill())
			.height(Size::px(64.0))
			.horizontal()
			.spacing(theme.gap)
			.center()
			.border(border_bottom(theme.border, theme.panel_border))
			.child("These files cannot be downloaded automatically. Please download them manually and place them in your Downloads folder.")
			.child(
				icon_text_button("popout", "Open All", &theme)
					.active(&theme)
					.on_press(move |_| {
						manual_files::open_all(&files2.0);
					})
			);

		let contents = rect()
			.expanded()
			.spacing(theme.gap2)
			.child(top)
			.child(rect().expanded().padding(theme.gap2).child(files));

		let back_state2 = back_state.clone();
		Modal::new("Files Need Downloading".into(), "download".into())
			.size_large()
			.on_close(move |_| {
				let _ = back_state2
					.event_tx
					.send(BackEvent::ConfirmYesNoPrompt { yes: false });
				front_state.write().set_modal(None);
			})
			.maybe_child(true, || contents)
			.cancel_button()
	}
}

fn file(
	file: &ManualFile,
	is_found: bool,
	front_state: &Shared<FrontState>,
	theme: &Theme,
) -> impl IntoElement {
	let size = Size::px(40.0);
	let (ico, fg, bg, tip) = if is_found {
		("check", theme.success, theme.success_bg, "File found")
	} else {
		("delete", theme.error, theme.error_bg, "File not found")
	};

	let indicator = rect()
		.width(size.clone())
		.height(size.clone())
		.center()
		.color(fg)
		.background(bg)
		.border(theme.border(fg))
		.corner_radius(CornerRadius::new(theme.round, 0.0, 0.0, theme.round))
		.tip(front_state, tip)
		.child(icon(ico, 16.0));

	let url = file.url.clone();
	let copy_button = icon_button("copy", theme).on_press(move |_| {
		let _ = Clipboard::set(url.clone());
	});
	let copy_button = rect()
		.width(size.clone())
		.height(size.clone())
		.center()
		.tip(front_state, "Copy URL")
		.child(copy_button);

	let url = file.url.clone();
	let open_button = icon_button("popout", theme).on_press(move |_| {
		let _ = open_link(&url);
	});
	let open_button = rect()
		.width(size.clone())
		.height(size.clone())
		.center()
		.tip(front_state, "Open in browser")
		.child(open_button);

	let contents = rect()
		.cont()
		.cross_align(Alignment::Center)
		.child(segment(file.filename.clone(), 1.0))
		.maybe(file.req.is_some(), |this| {
			this.child(
				segment(
					PackageChip {
						req: file.req.clone().unwrap(),
						error: false,
					},
					1.0,
				)
				.cross_align(Alignment::End),
			)
		})
		.child(copy_button)
		.child(open_button);

	let contents = segment(contents, 1.0)
		.height(size.clone())
		.main_align(Alignment::Center)
		.border(Border {
			fill: theme.panel_border,
			width: BorderWidth {
				top: theme.border,
				right: theme.border,
				bottom: theme.border,
				left: 0.0,
			},
			alignment: BorderAlignment::Inner,
		})
		.corner_radius(CornerRadius::new(0.0, theme.round, theme.round, 0.0))
		.padding(Gaps::new(0.0, 0.0, 0.0, theme.gap2));

	rect()
		.width(Size::fill())
		.height(size.clone())
		.corner_radius(theme.round)
		.horizontal()
		.flex()
		.cross_align(Alignment::Center)
		.child(indicator)
		.child(contents)
}
