use itertools::Itertools;
use nitrolaunch::{
	pkg_crate::metadata::{LongDescriptionFormat, PackageMetadata},
	shared::{pkg::ArcPkgReq, util::open_link},
};

use crate::{
	components::{
		gallery::Gallery,
		input::tabs::TopTabs,
		markdown::{HTMLViewer, MarkdownHTMLViewer},
		pkg::versions::PackageVersions,
		tag::{loader_tag, repo_tag},
	},
	ops::packages::FetchPackageDetails,
	prelude::*,
	routing::Page,
	state::FrontState,
	util::{PtrEq, Shared},
};

#[derive(PartialEq)]
pub struct PackageView {
	pub req: ArcPkgReq,
	pub fullscreen: bool,
}

impl Component for PackageView {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let details_query = use_query(Query::new(
			self.req.clone(),
			FetchPackageDetails::new(back_state.clone()).toast(
				&back_state,
				None,
				"Failed to fetch package",
			),
		));

		let tab = use_state(|| Tab::Description);
		let icon_is_hovered = use_state(|| false);

		let details = details_query.read();
		let details = details.state();
		let is_loading = !details.is_ok();
		let details = details.ok();
		let meta = details.map(|x| x.meta.clone()).unwrap_or_default();
		let props = details.map(|x| x.props.clone()).unwrap_or_default();

		let default_icon = icon("box", 48.0).into_element();
		let ico = if is_loading {
			CircularLoader::new().size(48.0).into_element()
		} else if let Some(ico) = &meta.icon {
			img(ico)
				.error_renderer(move |_| default_icon.clone())
				.width(Size::px(56.0))
				.height(Size::px(56.0))
				.corner_radius(theme.round2)
				.maybe(*icon_is_hovered.read() && !self.fullscreen, |this| {
					this.child(
						rect()
							.expanded()
							.position(Position::new_absolute())
							.center()
							.child(icon("fullscreen", 24.0).color(theme.bg)),
					)
				})
				.into_element()
		} else {
			default_icon
		};

		let fullscreen = self.fullscreen;
		let front_state2 = front_state.clone();
		let req = self.req.clone();
		let ico = rect()
			.maybe(!self.fullscreen, |this| this.hover(icon_is_hovered))
			.on_press(move |_| {
				if !fullscreen {
					front_state2.write().navigate(Page::Package(req.clone()));
				}
			})
			.child(ico);

		let name = if is_loading {
			"Loading".into()
		} else if let Some(name) = &meta.name {
			name.clone()
		} else {
			self.req.to_string()
		};

		let upper_details = rect()
			.width(Size::fill())
			.horizontal()
			.cross_align(Alignment::Center)
			.child(
				label()
					.text(name)
					.font_size(theme.font2)
					.font_weight(FontWeight::BOLD),
			);

		let repo = self
			.req
			.repository
			.as_deref()
			.map(|x| repo_tag(x, false, &back_state, &theme));
		let loaders = props
			.supported_loaders
			.iter()
			.flatten()
			.flat_map(|x| x.get_matches())
			.unique()
			.map(|x| loader_tag(&x, false, &theme).into_element());
		let lower_details = rect()
			.width(Size::fill())
			.horizontal()
			.spacing(theme.gap)
			.cross_align(Alignment::Center)
			.maybe_child(repo)
			.children(loaders);

		let details = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.spacing(theme.gap2)
			.center()
			.child(upper_details)
			.child(lower_details);

		let description = rect()
			.width(Size::flex(2.0))
			.height(Size::fill())
			.main_align(Alignment::Center)
			.cross_align(Alignment::End)
			.padding(Gaps::new(0.0, theme.gap2 * 2.0, 0.0, 0.0))
			.child(
				clip_text(meta.description.as_deref().unwrap_or("..."))
					.color(theme.fg2)
					.text_align(TextAlign::End)
					.max_lines(2),
			);

		let top = rect()
			.expanded()
			.cont()
			.child(
				rect()
					.width(Size::px(80.0))
					.height(Size::fill())
					.center()
					.child(ico),
			)
			.child(details)
			.child(description);

		let banner = meta.banner.as_ref().map(|x| {
			let image = img(x)
				.expanded()
				.aspect_ratio(AspectRatio::Max)
				.opacity(0.25)
				.error_renderer(|_| rect().into_element())
				.loading_placeholder(rect());

			rect()
				.width(Size::fill())
				.height(Size::percent(100.0))
				.position(Position::new_absolute())
				.child(image)
		});

		let top_container = rect()
			.width(Size::fill())
			.height(Size::px(80.0))
			.border(border_bottom(theme.border, theme.panel_border))
			.maybe_child(banner)
			.child(top);

		let loading_spinner = rect()
			.expanded()
			.center()
			.child(CircularLoader::new())
			.into_element();
		let contents = match &*tab.read() {
			Tab::Description => {
				if let Some(long_description) = &meta.long_description {
					let viewer = match meta.long_description_format {
						LongDescriptionFormat::Markdown => MarkdownHTMLViewer {
							body: long_description.clone(),
						}
						.into_element(),
						LongDescriptionFormat::HTML => HTMLViewer {
							body: long_description.clone(),
						}
						.into_element(),
					};
					let viewer = ScrollView::new()
						.expanded()
						.direction(Direction::Vertical)
						.child(viewer);

					rect().expanded().child(viewer).into_element()
				} else if is_loading {
					loading_spinner
				} else {
					placeholder("No description provided", &theme).into_element()
				}
			}
			Tab::Versions => PackageVersions {
				req: self.req.clone(),
				meta: PtrEq(meta.clone()),
				props: PtrEq(props.clone()),
			}
			.into_element(),
			Tab::Gallery => {
				if let Some(gallery) = meta.gallery.as_ref().filter(|x| !x.is_empty()) {
					Gallery {
						items: gallery
							.iter()
							.map(|x| {
								ImageSource::Uri(
									Url::parse(x)
										.unwrap_or(Url::parse("https://example.com").unwrap()),
								)
							})
							.collect(),
						columns: 3,
					}
					.into_element()
				} else if is_loading {
					loading_spinner
				} else {
					placeholder("Gallery empty", &theme).into_element()
				}
			}
		};
		let main = rect()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.child(contents);

		let tabs = TopTabs::from_state(tab)
			.child(SelectOption::new(
				Tab::Description,
				"Description",
				Some("text"),
			))
			.child(SelectOption::new(Tab::Versions, "Versions", Some("tag")))
			.child(SelectOption::new(Tab::Gallery, "Gallery", Some("picture")));

		let main = rect()
			.width(Size::flex(4.0))
			.height(Size::fill())
			.flex()
			.child(tabs)
			.child(main);

		let right = rect()
			.width(Size::flex(1.5))
			.height(Size::fill())
			.border(border_left(theme.border, theme.panel_border))
			.child(properties(&self.req, &meta, &front_state, &theme));

		let bottom = rect()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.flex()
			.horizontal()
			.child(main)
			.child(right);

		rect().expanded().flex().child(top_container).child(bottom)
	}
}

#[derive(PartialEq, Clone)]
enum Tab {
	Description,
	Versions,
	Gallery,
}

fn properties(
	req: &ArcPkgReq,
	meta: &PackageMetadata,
	front_state: &Shared<FrontState>,
	theme: &Theme,
) -> Rect {
	rect()
		.expanded()
		.padding(theme.gap)
		.spacing(theme.gap)
		.child(property("hashtag", "ID", req, front_state, theme))
		.maybe(meta.authors.is_some(), |this| {
			let authors = meta.authors.as_ref().unwrap();
			let title = if authors.len() == 1 {
				"Author"
			} else {
				"Authors"
			};
			let ico = if authors.len() == 1 {
				"user"
			} else {
				"multiple_users"
			};
			this.child(property(
				ico,
				title,
				meta.authors.as_ref().unwrap().iter().join("  "),
				front_state,
				theme,
			))
		})
		.maybe(meta.support_link.is_some(), |this| {
			this.child(property_link(
				"heart",
				"Support",
				meta.support_link.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.downloads.is_some(), |this| {
			this.child(property(
				"download",
				"Downloads",
				format_downloads(*meta.downloads.as_ref().unwrap()),
				front_state,
				theme,
			))
		})
		.maybe(meta.website.is_some(), |this| {
			this.child(property_link(
				"globe",
				"Website",
				meta.website.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.community.is_some(), |this| {
			this.child(property_link(
				"multiple_users",
				"Community",
				meta.community.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.documentation.is_some(), |this| {
			this.child(property_link(
				"book",
				"Documentation",
				meta.documentation.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.source.is_some(), |this| {
			this.child(property_link(
				"curly_braces",
				"Source Code",
				meta.source.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.issues.is_some(), |this| {
			this.child(property_link(
				"warning",
				"Issues",
				meta.issues.as_ref().unwrap(),
				theme,
			))
		})
		.maybe(meta.license.is_some(), |this| {
			this.child(property(
				"key",
				"License",
				meta.license.as_ref().unwrap(),
				front_state,
				theme,
			))
		})
}

fn property(
	ico: &'static str,
	title: &str,
	value: impl ToString,
	front_state: &Shared<FrontState>,
	theme: &Theme,
) -> Rect {
	property_impl(
		ico,
		title,
		clip_text(&value.to_string())
			.color(theme.fg2)
			.tip(front_state, &value.to_string()),
		theme,
	)
}

fn property_link(ico: &'static str, title: &str, value: impl ToString, theme: &Theme) -> Rect {
	let value = value.to_string();
	property_impl(
		ico,
		title,
		icon_button("popout", theme).on_press(move |_| {
			let _ = open_link(&value);
		}),
		theme,
	)
}

fn property_impl(ico: &'static str, title: &str, value: impl IntoElement, theme: &Theme) -> Rect {
	rect()
		.width(Size::fill())
		.height(Size::px(32.0))
		.corner_radius(theme.round)
		.cont()
		.cross_align(Alignment::Center)
		.padding(Gaps::new(0.0, (32.0 - 16.0) / 2.0, 0.0, 0.0))
		.child(
			rect()
				.width(Size::px(32.0))
				.height(Size::fill())
				.center()
				.maybe(ico == "heart", |this| this.color(theme.error))
				.child(icon(ico, 16.0)),
		)
		.child(
			rect()
				.margin(Gaps::new(0.0, theme.gap, 0.0, 0.0))
				.child(title),
		)
		.child(
			segment(value, 1.0)
				.cross_align(Alignment::End)
				.overflow(Overflow::Clip),
		)
}

fn format_downloads(download: u32) -> String {
	if download >= 1_000_000 {
		format!("{:.1}M", download as f64 / 1_000_000.0)
	} else if download >= 1_000 {
		format!("{:.1}K", download as f64 / 1_000.0)
	} else {
		format!("{}", download)
	}
}
