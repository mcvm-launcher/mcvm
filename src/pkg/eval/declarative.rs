use std::sync::Arc;

use anyhow::bail;
use itertools::Itertools;
use nitro_pkg::RequiredPackage;
use nitro_pkg::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
};
use nitro_pkg::properties::PackageProperties;
use nitro_pkg::script_eval::AddonInstructionData;
use nitro_shared::loaders::LoaderMatch;
use nitro_shared::pkg::{ArcPkgReq, PackageID};
use nitro_shared::util::DeserListOrSingle;

use crate::plugin::PluginManager;

use super::conditions::{check_arch_condition, check_os_condition};
use super::{
	EvalData, EvalInput, MAX_NOTICE_CHARACTERS, MAX_NOTICE_INSTRUCTIONS, Routine,
	create_valid_addon_request,
};

/// Evaluate a declarative package
pub fn eval_declarative_package(
	req: &ArcPkgReq,
	contents: &DeclarativePackage,
	input: EvalInput,
	properties: Arc<PackageProperties>,
	routine: Routine,
	plugins: PluginManager,
) -> anyhow::Result<EvalData> {
	let eval_data =
		eval_declarative_package_impl(req, contents, input, properties, routine, plugins)?;

	Ok(eval_data)
}

/// Implementation for evaluating a declarative package
fn eval_declarative_package_impl(
	req: &ArcPkgReq,
	contents: &DeclarativePackage,
	input: EvalInput,
	properties: Arc<PackageProperties>,
	routine: Routine,
	plugins: PluginManager,
) -> anyhow::Result<EvalData> {
	let mut eval_data = EvalData::new(input, req.clone(), properties, &routine, plugins);

	// Vars for the EvalData that are modified by conditions / versions
	let mut relations = contents.relations.clone();
	let mut notices = Vec::new();

	if eval_data.input.params.force && eval_data.input.params.required_content_versions.is_empty() {
		bail!("Force override set on package without specifying a version");
	}

	// Apply conditional rules
	for rule in &contents.conditional_rules {
		for condition in &rule.conditions {
			if !check_condition_set(condition, &eval_data.input, false) {
				continue;
			}
		}

		relations.merge(rule.properties.relations.clone());
		notices.extend(rule.properties.notices.iter().cloned());
	}

	// Select addon versions
	for (addon_id, addon) in &contents.addons {
		// Check conditions
		if !check_multiple_condition_sets(&addon.conditions, &eval_data.input) {
			continue;
		}

		// Pick the best version
		let version =
			pick_best_addon_version(&addon.versions, &eval_data.input, &eval_data.properties);
		if let Some(version) = version {
			// Bundle addons won't have an actual addon
			let addon_kind = addon.kind.to_addon_kind();
			if let Some(addon_kind) = addon_kind {
				let data = AddonInstructionData {
					id: addon_id.clone(),
					url: version.url.clone(),
					path: version.path.clone(),
					kind: addon_kind,
					file_name: version.filename.clone(),
					version: version.version.clone(),
					modpack_format: addon.modpack_format.clone(),
					hashes: version.hashes.clone(),
					is_manual: version.is_manual,
				};

				let addon_req = create_valid_addon_request(data, req.clone(), &eval_data.input)?;

				eval_data.addon_reqs.push(addon_req);
			}

			if let Some(versions) = &version.conditional_properties.minecraft_versions {
				eval_data.available_minecraft_versions.extend(
					versions
						.iter()
						.flat_map(|x| x.get_matches(&eval_data.input.constants.version_list)),
				);
			}

			relations.merge(version.relations.clone());
			notices.extend(version.notices.iter().cloned());
			if let Some(content_version) = version
				.conditional_properties
				.content_versions
				.as_ref()
				.and_then(|x| x.first())
			{
				eval_data.selected_content_version = Some(content_version.clone());
			}
		} else {
			handle_no_matched_versions(addon)?;
		}
	}

	eval_data
		.deps
		.extend(relations.dependencies.iter().map(|x| {
			vec![RequiredPackage {
				value: x.clone().into(),
				explicit: false,
			}]
		}));
	eval_data
		.deps
		.extend(relations.explicit_dependencies.iter().map(|x| {
			vec![RequiredPackage {
				value: x.clone().into(),
				explicit: true,
			}]
		}));
	eval_data
		.conflicts
		.extend(relations.conflicts.iter().cloned().map(PackageID::from));
	eval_data
		.extensions
		.extend(relations.extensions.iter().cloned().map(PackageID::from));
	eval_data
		.bundled
		.extend(relations.bundled.iter().cloned().map(PackageID::from));
	eval_data.compats.extend(
		relations
			.compats
			.iter()
			.cloned()
			.map(|(a, b)| (a.into(), b.into())),
	);
	eval_data
		.recommendations
		.extend(relations.recommendations.iter().cloned());
	eval_data
		.inclusions
		.extend(relations.inclusions.iter().cloned().map(PackageID::from));

	eval_data.notices.extend(notices);

	// Check notices
	if eval_data.notices.len() > MAX_NOTICE_INSTRUCTIONS {
		bail!("Max number of notices was exceded (>{MAX_NOTICE_INSTRUCTIONS})");
	}
	for notice in &eval_data.notices {
		if notice.len() > MAX_NOTICE_CHARACTERS {
			bail!("Notice message is too long (>{MAX_NOTICE_CHARACTERS})");
		}
	}

	Ok(eval_data)
}

/// Pick the best addon version from a list of declarative addon versions
pub fn pick_best_addon_version<'a>(
	versions: &'a [DeclarativeAddonVersion],
	input: &'a EvalInput,
	properties: &PackageProperties,
) -> Option<&'a DeclarativeAddonVersion> {
	// If a specific version is forced, return that version no matter what
	if input.params.force {
		return versions
			.iter()
			.find(|x| x.content_versions_match(&input.params.required_content_versions));
	}

	// Filter versions that are not allowed
	let versions = versions.iter().filter(|x| {
		let version_id_matches = x
			.version
			.as_ref()
			.is_some_and(|version| input.params.required_content_versions.contains(version));
		check_condition_set(&x.conditional_properties, input, version_id_matches)
	});

	// Sort so that versions with less loader matches come first
	fn get_matches(version: &DeclarativeAddonVersion) -> u16 {
		let mut out = 0;
		if let Some(loaders) = &version.conditional_properties.loaders {
			out += loaders.iter().fold(0, |acc, x| acc + get_loader_matches(x));
		}

		out
	}
	let versions = versions.sorted_by_cached_key(|x| get_matches(x));

	let versions: Vec<_> = versions.collect();

	// Check preferred content versions first
	if !input.params.preferred_content_versions.is_empty()
		&& let Some(content_versions) = &properties.content_versions
	{
		// Sort so newest comes first
		let preferred_content_versions = match_ordering(
			input.params.preferred_content_versions.iter(),
			content_versions,
		)
		.rev();

		let default = DeserListOrSingle::default();
		for version in preferred_content_versions {
			if let Some(version) = versions.iter().find(|x| {
				x.conditional_properties
					.content_versions
					.as_ref()
					.unwrap_or(&default)
					.contains(version)
					|| x.version.as_ref().is_some_and(|x| x == version)
			}) {
				return Some(*version);
			}
		}
	}

	// Sort so that versions with newer content versions come first
	if let Some(content_versions) = &properties.content_versions {
		// We reverse the iterator because max_by_key returns the last of equal elements, and we want versions at the beginning to take priority
		let version = versions.iter().rev().max_by_key(|x| {
			if let Some(versions) = &x.conditional_properties.content_versions {
				versions
					.iter()
					.map(|x| content_versions.iter().position(|candidate| candidate == x))
					.max()
					.unwrap_or(Some(0))
			} else {
				Some(0)
			}
		});

		if version.is_some() {
			return version.copied();
		}
	}

	versions.into_iter().next()
}

/// Check multiple sets of addon version conditions
fn check_multiple_condition_sets(
	conditions: &[DeclarativeConditionSet],
	input: &EvalInput,
) -> bool {
	conditions
		.iter()
		.all(|x| check_condition_set(x, input, false))
}

/// Filtering function for addon version picking and rule checking
fn check_condition_set(
	conditions: &DeclarativeConditionSet,
	input: &EvalInput,
	skip_content_versions: bool,
) -> bool {
	if let Some(stability) = &conditions.stability
		&& stability > &input.params.stability
	{
		return false;
	}

	if let Some(side) = conditions.side
		&& side != input.params.side
	{
		return false;
	}

	if let Some(features) = &conditions.features {
		for feature in features.iter() {
			if !input.params.features.contains(feature) {
				return false;
			}
		}
	}

	if let Some(minecraft_version) = &input.constants.version
		&& let Some(minecraft_versions) = &conditions.minecraft_versions
		&& !minecraft_versions
			.iter()
			.any(|x| x.matches_single(minecraft_version, &input.constants.version_list))
	{
		return false;
	}

	if let Some(loaders) = &conditions.loaders
		&& !loaders.iter().any(|x| x.matches(&input.constants.loader))
	{
		return false;
	}

	if let Some(operating_systems) = &conditions.operating_systems
		&& !operating_systems.iter().any(check_os_condition)
	{
		return false;
	}

	if let Some(architectures) = &conditions.architectures
		&& !architectures.iter().any(check_arch_condition)
	{
		return false;
	}

	if let Some(languages) = &conditions.languages
		&& !languages.iter().any(|x| x == &input.constants.language)
	{
		return false;
	}

	if !skip_content_versions
		&& let Some(content_versions) = &conditions.content_versions
		&& !input.params.required_content_versions.is_empty()
		&& !content_versions
			.iter()
			.any(|x| input.params.required_content_versions.contains(x))
	{
		return false;
	}

	true
}

/// Handle the case where no versions were matched for an addon
fn handle_no_matched_versions(addon: &DeclarativeAddon) -> anyhow::Result<()> {
	// If the addon is optional then this is ok
	if addon.optional {
		return Ok(());
	}

	bail!("No valid addon version found")
}

/// Get the number of matches that a loader match can have
fn get_loader_matches(loader: &LoaderMatch) -> u16 {
	match loader {
		LoaderMatch::FabricLike | LoaderMatch::ForgeLike => 2,
		LoaderMatch::Bukkit => 8,
		_ => 1,
	}
}

/// Matches the ordering of items in an iterator to a reference array
fn match_ordering<'a, T: Ord + Eq + 'a>(
	input: impl Iterator<Item = &'a T>,
	reference: &[T],
) -> impl DoubleEndedIterator<Item = &'a T> {
	input.sorted_by_cached_key(|x| {
		reference
			.iter()
			.position(|candidate| &candidate == x)
			.unwrap_or(reference.len())
	})
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use nitro_pkg::{PkgRequest, declarative::deserialize_declarative_package};
	use nitro_shared::Side;
	use nitro_shared::lang::Language;
	use nitro_shared::loaders::Loader;
	use nitro_shared::pkg::PackageStability;
	use nitro_shared::util::DeserListOrSingle;

	use crate::{
		io::paths::Paths,
		pkg::eval::{EvalConstants, EvalParameters, RequiredPackage},
	};

	use super::*;

	#[test]
	fn test_declarative_package_eval() {
		let contents = r#"
			{
				"addons": {
					"test": {
						"kind": "mod",
						"versions": [
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.2" ],
								"loaders": [ "forge" ],
								"version": "1"
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.2" ],
								"loaders": [ "fabriclike" ],
								"version": "2",
								"relations": {
									"dependencies": [ "foo" ]
								}
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.3" ],
								"loaders": [ "fabriclike" ],
								"version": "3"
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.2" ],
								"loaders": [ "fabriclike" ],
								"version": "4"
							}
						]
					}
				},
				"relations": {
					"dependencies": [ "bar" ]
				},
				"conditional_rules": [
					{
						"conditions": [
							{
								"minecraft_versions": [ "1.19.2" ]
							}
						],
						"properties": {
							"relations": {
								"dependencies": [ "baz" ]
							}
						}
					}
				]
			}
		"#;

		let pkg = deserialize_declarative_package(contents).unwrap();

		let constants = get_eval_constants();
		let input = EvalInput {
			constants: Arc::new(constants),
			params: EvalParameters::new(Side::Client),
		};

		let plugins = PluginManager::new(&Paths::new_no_create().unwrap());
		let eval = eval_declarative_package(
			&PkgRequest::parse("foo", nitro_pkg::PkgRequestSource::UserRequire).arc(),
			&pkg,
			input,
			Arc::new(PackageProperties::default()),
			Routine::Install,
			plugins,
		)
		.unwrap();

		let addon = eval.addon_reqs.first().unwrap();
		assert_eq!(addon.addon.version, Some("2".into()));

		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: "foo".into(),
			explicit: false
		}]));
		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: "bar".into(),
			explicit: false
		}]));
		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: "baz".into(),
			explicit: false
		}]));
	}

	#[test]
	fn test_addon_version_picking() {
		let version1 = DeclarativeAddonVersion {
			conditional_properties: DeclarativeConditionSet {
				loaders: Some(DeserListOrSingle::List(vec![LoaderMatch::Loader(
					Loader::Fabric,
				)])),
				content_versions: Some(DeserListOrSingle::Single("1".into())),
				..Default::default()
			},
			version: Some("1".into()),
			..Default::default()
		};

		let version2 = DeclarativeAddonVersion {
			conditional_properties: DeclarativeConditionSet {
				loaders: Some(DeserListOrSingle::List(vec![LoaderMatch::Loader(
					Loader::Fabric,
				)])),
				content_versions: Some(DeserListOrSingle::Single("2".into())),
				..Default::default()
			},
			version: Some("2".into()),
			..Default::default()
		};

		let version3 = DeclarativeAddonVersion {
			conditional_properties: DeclarativeConditionSet {
				loaders: Some(DeserListOrSingle::List(vec![LoaderMatch::FabricLike])),
				content_versions: Some(DeserListOrSingle::Single("2".into())),
				..Default::default()
			},
			version: Some("3".into()),
			..Default::default()
		};

		let versions = vec![version1, version2, version3];

		let constants = get_eval_constants();
		let input = EvalInput {
			constants: Arc::new(constants),
			params: EvalParameters::new(Side::Client),
		};

		let properties = PackageProperties {
			content_versions: Some(vec!["1".into(), "2".into()]),
			..Default::default()
		};

		let version = pick_best_addon_version(&versions, &input, &properties)
			.expect("Version should have been found");

		assert_eq!(version.version, Some("2".into()));
	}

	fn get_eval_constants() -> EvalConstants {
		EvalConstants {
			version: Some("1.19.2".into()),
			version_list: vec!["1.19.2".to_string(), "1.19.3".to_string()],
			loader: Loader::Fabric,
			language: Language::AmericanEnglish,
			default_stability: PackageStability::Latest,
			suppress: Vec::new(),
		}
	}
}
