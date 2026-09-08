use std::rc::Rc;

use itertools::Itertools;
use nitrolaunch::shared::pkg::ArcPkgReq;

use crate::{
	components::input::select::Selected, ops::packages::FetchPackageDetails, prelude::*,
	theme::Colorway,
};

pub mod diffs;
pub mod error;
pub mod filters;
pub mod install;
pub mod manual;
pub mod versions;

#[derive(PartialEq)]
pub struct RepoSelector {
	pub repo: State<Option<String>>,
}

impl Component for RepoSelector {
	fn render(&self) -> impl IntoElement {
		let back_state = use_consume::<BackState>();
		let theme = use_theme();
		let repos = back_state.repos();

		let repo = self.repo;

		InlineSelect::new(
			Selected::Single(self.repo.read().cloned()),
			Rc::new(move |selected| {
				repo.clone().set(selected.single().clone());
			}),
		)
		.fit()
		.child(SelectOption::new(None, "", Some("asterisk")).tip("All repositories"))
		.children(
			repos
				.iter()
				.sorted_by_cached_key(|x| x.0.clone())
				.map(|(id, meta)| {
					let name = meta.name.as_deref().unwrap_or(id);
					let selected_bg = meta
						.color
						.as_deref()
						.and_then(Color::from_hex)
						.unwrap_or(theme.bg);

					let ico = meta
						.icon
						.as_deref()
						.map(|x| {
							SvgViewer::new(Bytes::from(x.as_bytes().to_vec()))
								.width(Size::px(16.0))
								.height(Size::px(16.0))
								.into_element()
						})
						.unwrap_or(icon("box", 16.0).into_element());
					SelectOption::new_custom_icon(Some(id.clone()), "", ico)
						.selected_colorway(Colorway::new(theme.bg, selected_bg, selected_bg))
						.tip(name)
				}),
		)
	}
}

#[derive(PartialEq)]
pub struct PackageChip {
	pub req: ArcPkgReq,
	pub error: bool,
}

impl Component for PackageChip {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let details_query = use_query(Query::new(
			self.req.clone(),
			FetchPackageDetails::new(back_state.clone()),
		));

		let ico = details_query
			.read()
			.state()
			.ok()
			.and_then(|x| x.meta.icon.clone());
		let default_icon = icon("box", 32.0).into_element();
		let ico = ico
			.map(|x| {
				let default_icon = default_icon.clone();
				img(&x)
					.error_renderer(move |_| default_icon.clone())
					.width(Size::px(20.0))
					.height(Size::px(20.0))
					.corner_radius(theme.round)
					.into_element()
			})
			.unwrap_or(default_icon);

		let name = details_query
			.read()
			.state()
			.ok()
			.and_then(|x| x.meta.name.clone())
			.unwrap_or_else(|| self.req.to_string_no_version());

		rect()
			.height(Size::px(theme.input_height))
			.padding(theme.gap)
			.cont()
			.cross_align(Alignment::Center)
			.child(ico)
			.child(
				rect()
					.height(Size::fill())
					.main_align(Alignment::Center)
					.child(name),
			)
	}
}
