use itertools::Itertools;
use nitrolaunch::{
	pkg_crate::{
		declarative::DeclarativeAddonVersion, metadata::PackageMetadata,
		properties::PackageProperties,
	},
	shared::{
		pkg::{ArcPkgReq, PackageStability},
		versions::VersionPattern,
	},
};

use crate::{
	components::{
		pkg::install::PackageInstallModal,
		tag::{icon_text_tag, loader_tag, text_tag},
	},
	ops::packages::FetchPackageContentVersions,
	prelude::*,
	util::PtrEq,
};

#[derive(PartialEq)]
pub struct PackageVersions {
	pub req: ArcPkgReq,
	pub meta: PtrEq<PackageMetadata>,
	pub props: PtrEq<PackageProperties>,
}

impl Component for PackageVersions {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let versions_query = use_query(Query::new(
			self.req.clone(),
			FetchPackageContentVersions::new(back_state.clone()),
		));
		let installing_version = use_state::<Option<String>>(|| None);

		let default_versions = Vec::new();
		let versions = versions_query.read();
		let versions = versions.state();
		let versions = versions.ok().cloned().unwrap_or(default_versions);
		let len = versions.len();

		let versions = VirtualScrollView::new(move |item, _| {
			let version = &versions[item.index];

			Version {
				version: NotEq(version.clone()),
				installing_version,
			}
			.into_element()
		})
		.item_size(48.0 + theme.gap)
		.length(len)
		.expanded();

		let mut installing_version2 = installing_version;
		rect().expanded().padding(theme.gap2).child(versions).maybe(
			installing_version.read().is_some(),
			|this| {
				this.child(PackageInstallModal {
					req: self
						.req
						.with_content_version(VersionPattern::Single(
							installing_version.read().clone().unwrap(),
						))
						.arc(),
					meta: self.meta.clone(),
					props: self.props.clone(),
					on_close: (move |_| {
						installing_version2.set(None);
					})
					.into(),
				})
			},
		)
	}
}

#[derive(PartialEq)]
struct Version {
	version: NotEq<DeclarativeAddonVersion>,
	installing_version: State<Option<String>>,
}

impl Component for Version {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		let (ico, ico_fg, ico_bg, ico_tip) = match self
			.version
			.0
			.conditional_properties
			.stability
			.unwrap_or(PackageStability::Stable)
		{
			PackageStability::Stable => ("tag", theme.success, theme.success_bg, "Stable"),
			PackageStability::Latest => ("warning", theme.warning, theme.error_bg, "Unstable"),
		};
		let ico = rect()
			.width(Size::px(48.0))
			.height(Size::px(48.0))
			.center()
			.child(
				rect()
					.width(Size::px(24.0))
					.height(Size::px(24.0))
					.center()
					.background(ico_bg)
					.corner_radius(theme.round)
					.tip(&front_state, ico_tip)
					.child(icon(ico, 12.0).color(ico_fg)),
			);

		let name = self
			.version
			.0
			.conditional_properties
			.content_versions
			.as_ref()
			.and_then(|x| x.first().map(|x| x.as_str()))
			.or(self.version.0.version.as_deref())
			.unwrap_or("idk");

		let mut installing_version = self.installing_version;
		let content_version = self
			.version
			.0
			.conditional_properties
			.content_versions
			.as_ref()
			.and_then(|x| x.first().cloned());
		let install = rect()
			.width(Size::px(48.0))
			.height(Size::px(48.0))
			.center()
			.child(
				rect()
					.width(Size::px(24.0))
					.height(Size::px(24.0))
					.center()
					.background(theme.primary_bg)
					.border(theme.border(theme.primary))
					.corner_radius(theme.round)
					.clickable()
					.on_press(move |_| {
						if let Some(version) = &content_version {
							installing_version.set(Some(version.clone()));
						}
					})
					.child(icon("download", 12.0).color(theme.primary)),
			);

		let versions =
			if let Some(versions) = &self.version.0.conditional_properties.minecraft_versions {
				let out = versions
					.iter()
					.map(|x| x.to_string())
					.map(|x| text_tag(&x, &theme))
					.take(3);
				let ellipsis = if versions.len() > 3 {
					Some(text_tag("...", &theme))
				} else {
					None
				};

				out.chain(ellipsis).collect_vec()
			} else {
				Vec::new()
			};
		let loaders = self
			.version
			.0
			.conditional_properties
			.loaders
			.iter()
			.flat_map(|x| x.iter())
			.flat_map(|x| x.get_matches())
			.unique()
			.map(|x| loader_tag(&x, true, &theme).into_element());

		let manual_indicator = if self.version.0.is_manual {
			Some(
				icon("file_cross", 12.0)
					.color(theme.warning)
					.tip(&front_state, "Must be downloaded manually"),
			)
		} else {
			None
		};

		rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.margin(Gaps::new(0.0, 0.0, theme.gap, 0.0))
			.cont()
			.child(ico)
			.child(
				segment(name, 1.0)
					.height(Size::fill())
					.main_align(Alignment::Center),
			)
			.child(
				rect()
					.height(Size::fill())
					.cont()
					.main_align(Alignment::End)
					.cross_align(Alignment::Center)
					.children(versions)
					.children(loaders)
					.maybe_child(manual_indicator),
			)
			.child(install)
	}
}

/// Used for displaying or configuring a package's version
#[derive(PartialEq)]
pub struct InstalledPackageVersion {
	pub configured: Option<String>,
	pub installed: Option<String>,
}

impl Component for InstalledPackageVersion {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		let (ico, contents) = if let Some(configured) = &self.configured {
			("lock", configured.clone())
		} else if let Some(installed) = &self.installed {
			("tag", installed.clone())
		} else {
			("tag", "None".to_string())
		};

		let tip = if let Some(configured) = &self.configured {
			if let Some(installed) = &self.installed {
				if configured != installed {
					format!("Requested version {configured}, installed version {installed}")
				} else {
					format!("Requested and installed version {configured}")
				}
			} else {
				format!("Requested version {configured}")
			}
		} else if let Some(installed) = &self.installed {
			format!("Installed version {installed}")
		} else {
			"No version".into()
		};

		icon_text_tag(ico, &contents, &theme).tip(&front_state, &tip)
	}
}
