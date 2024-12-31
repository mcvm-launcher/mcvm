use anyhow::bail;
use itertools::Itertools;
use mcvm_pkg::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
};
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::script_eval::AddonInstructionData;
use mcvm_pkg::RequiredPackage;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm_shared::pkg::PackageID;

use crate::plugin::PluginManager;

use super::conditions::{check_arch_condition, check_os_condition};
use super::{
	create_valid_addon_request, EvalData, EvalInput, Routine, MAX_NOTICE_CHARACTERS,
	MAX_NOTICE_INSTRUCTIONS,
};

/// Evaluate a declarative package
pub fn eval_declarative_package<'a>(
	id: PackageID,
	contents: &DeclarativePackage,
	input: EvalInput<'a>,
	properties: PackageProperties,
	routine: Routine,
	plugins: &'a PluginManager,
) -> anyhow::Result<EvalData<'a>> {
	let eval_data =
		eval_declarative_package_impl(id, contents, input, properties, routine, plugins)?;

	Ok(eval_data)
}

/// Implementation for evaluating a declarative package
fn eval_declarative_package_impl<'a>(
	id: PackageID,
	contents: &DeclarativePackage,
	input: EvalInput<'a>,
	properties: PackageProperties,
	routine: Routine,
	plugins: &'a PluginManager,
) -> anyhow::Result<EvalData<'a>> {
	let pkg_id = id;

	let mut eval_data = EvalData::new(input, pkg_id.clone(), properties, &routine, plugins);

	// Vars for the EvalData that are modified by conditions / versions
	let mut relations = contents.relations.clone();
	let mut notices = Vec::new();

	// Apply conditional rules
	for rule in &contents.conditional_rules {
		for condition in &rule.conditions {
			if !check_condition_set(condition, &eval_data.input, &eval_data.properties) {
				continue;
			}
		}

		relations.merge(rule.properties.relations.clone());
		notices.extend(rule.properties.notices.iter().cloned());
	}

	// Select addon versions
	for (addon_id, addon) in &contents.addons {
		// Check conditions
		if !check_multiple_condition_sets(
			&addon.conditions,
			&eval_data.input,
			&eval_data.properties,
		) {
			continue;
		}

		// Pick the best version
		let version =
			pick_best_addon_version(&addon.versions, &eval_data.input, &eval_data.properties);
		if let Some(version) = version {
			let data = AddonInstructionData {
				id: addon_id.clone(),
				url: version.url.clone(),
				path: version.path.clone(),
				kind: addon.kind,
				file_name: version.filename.clone(),
				version: version.version.clone(),
				hashes: version.hashes.clone(),
			};

			let addon_req = create_valid_addon_request(data, pkg_id.clone(), &eval_data.input)?;

			eval_data.addon_reqs.push(addon_req);

			relations.merge(version.relations.clone());
			notices.extend(version.notices.iter().cloned());
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
	input: &'a EvalInput<'a>,
	properties: &PackageProperties,
) -> Option<&'a DeclarativeAddonVersion> {
	// Filter versions that are not allowed
	let versions = versions
		.iter()
		.filter(|x| check_condition_set(&x.conditional_properties, input, properties));

	// Sort so that versions with less loader matches come first
	fn get_matches(version: &DeclarativeAddonVersion) -> u16 {
		let mut out = 0;
		if let Some(modloaders) = &version.conditional_properties.modloaders {
			out += modloaders
				.iter()
				.fold(0, |acc, x| acc + get_modloader_matches(x));
		}

		if let Some(plugin_loaders) = &version.conditional_properties.plugin_loaders {
			out += plugin_loaders
				.iter()
				.fold(0, |acc, x| acc + get_plugin_loader_matches(x));
		}

		out
	}

	let mut versions = versions.sorted_by_cached_key(|x| get_matches(x));

	// Sort so that versions with newer content versions come first
	if let Some(content_versions) = &properties.content_versions {
		versions = versions.sorted_by_cached_key(|x| {
			if let Some(versions) = &x.conditional_properties.content_versions {
				versions
					.iter()
					.map(|x| content_versions.iter().position(|candidate| candidate == x))
					.min()
					.unwrap_or(Some(content_versions.len()))
			} else {
				Some(content_versions.len())
			}
		});
	}

	versions.next()
}

/// Check multiple sets of addon version conditions
fn check_multiple_condition_sets<'a>(
	conditions: &[DeclarativeConditionSet],
	input: &'a EvalInput<'a>,
	properties: &PackageProperties,
) -> bool {
	conditions
		.iter()
		.all(|x| check_condition_set(x, input, properties))
}

/// Filtering function for addon version picking and rule checking
fn check_condition_set<'a>(
	conditions: &DeclarativeConditionSet,
	input: &'a EvalInput<'a>,
	properties: &PackageProperties,
) -> bool {
	if let Some(stability) = &conditions.stability {
		if stability > &input.params.stability {
			return false;
		}
	}

	if let Some(side) = conditions.side {
		if side != input.params.side {
			return false;
		}
	}

	if let Some(features) = &conditions.features {
		for feature in features.iter() {
			if !input.params.features.contains(feature) {
				return false;
			}
		}
	}

	if let Some(minecraft_versions) = &conditions.minecraft_versions {
		if !minecraft_versions
			.iter()
			.any(|x| x.matches_single(&input.constants.version, &input.constants.version_list))
		{
			return false;
		}
	}

	if let Some(modloaders) = &conditions.modloaders {
		if !modloaders.iter().any(|x| {
			x.matches(
				&input
					.constants
					.modifications
					.get_modloader(input.params.side),
			)
		}) {
			return false;
		}
	}

	if let Some(plugin_loaders) = &conditions.plugin_loaders {
		if !plugin_loaders
			.iter()
			.any(|x| x.matches(&input.constants.modifications.server_type))
		{
			return false;
		}
	}

	if let Some(operating_systems) = &conditions.operating_systems {
		if !operating_systems.iter().any(check_os_condition) {
			return false;
		}
	}

	if let Some(architectures) = &conditions.architectures {
		if !architectures.iter().any(check_arch_condition) {
			return false;
		}
	}

	if let Some(languages) = &conditions.languages {
		if !languages.iter().any(|x| x == &input.constants.language) {
			return false;
		}
	}

	if let Some(content_versions) = &conditions.content_versions {
		if let Some(desired_version) = &input.params.content_version {
			let default_versions = Vec::new();
			if !content_versions.iter().any(|x| {
				desired_version.matches_single(
					x,
					properties
						.content_versions
						.as_ref()
						.unwrap_or(&default_versions),
				)
			}) {
				return false;
			}
		}
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

/// Get the number of matches that a modloader match can have
fn get_modloader_matches(modloader: &ModloaderMatch) -> u16 {
	match modloader {
		ModloaderMatch::FabricLike | ModloaderMatch::ForgeLike => 2,
		_ => 1,
	}
}

/// Get the number of matches that a plugin loader match can have
fn get_plugin_loader_matches(plugin_loader: &PluginLoaderMatch) -> u16 {
	match plugin_loader {
		PluginLoaderMatch::Bukkit => 8,
		_ => 1,
	}
}

#[cfg(test)]
mod tests {
	use mcvm_pkg::declarative::deserialize_declarative_package;
	use mcvm_shared::lang::Language;
	use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
	use mcvm_shared::pkg::PackageStability;
	use mcvm_shared::util::DeserListOrSingle;
	use mcvm_shared::Side;

	use crate::config::profile::GameModifications;
	use crate::pkg::eval::{EvalConstants, EvalParameters, RequiredPackage};

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
								"modloaders": [ "forge" ],
								"version": "1"
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.2" ],
								"modloaders": [ "fabriclike" ],
								"version": "2",
								"relations": {
									"dependencies": [ "foo" ]
								}
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.3" ],
								"modloaders": [ "fabriclike" ],
								"version": "3"
							},
							{
								"url": "example.com",
								"minecraft_versions": [ "1.19.2" ],
								"modloaders": [ "fabriclike" ],
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
			constants: &constants,
			params: EvalParameters::new(Side::Client),
		};

		let plugins = PluginManager::new();
		let eval = eval_declarative_package(
			PackageID::from("foo"),
			&pkg,
			input,
			PackageProperties::default(),
			Routine::Install,
			&plugins,
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
				modloaders: Some(DeserListOrSingle::List(vec![ModloaderMatch::Fabric])),
				content_versions: Some(DeserListOrSingle::Single("1".into())),
				..Default::default()
			},
			version: Some("1".into()),
			..Default::default()
		};

		let version2 = DeclarativeAddonVersion {
			conditional_properties: DeclarativeConditionSet {
				modloaders: Some(DeserListOrSingle::List(vec![ModloaderMatch::Fabric])),
				content_versions: Some(DeserListOrSingle::Single("2".into())),
				..Default::default()
			},
			version: Some("2".into()),
			..Default::default()
		};

		let version3 = DeclarativeAddonVersion {
			conditional_properties: DeclarativeConditionSet {
				modloaders: Some(DeserListOrSingle::List(vec![ModloaderMatch::FabricLike])),
				content_versions: Some(DeserListOrSingle::Single("2".into())),
				..Default::default()
			},
			version: Some("3".into()),
			..Default::default()
		};

		let versions = vec![version1, version2, version3];

		let constants = get_eval_constants();
		let input = EvalInput {
			constants: &constants,
			params: EvalParameters::new(Side::Client),
		};

		let properties = PackageProperties {
			content_versions: Some(vec!["2".into(), "1".into()]),
			..Default::default()
		};

		let version = pick_best_addon_version(&versions, &input, &properties)
			.expect("Version should have been found");

		assert_eq!(version.version, Some("2".into()));
	}

	fn get_eval_constants() -> EvalConstants {
		EvalConstants {
			version: "1.19.2".into(),
			version_list: vec!["1.19.2".to_string(), "1.19.3".to_string()],
			modifications: GameModifications::new(
				Modloader::Fabric,
				ClientType::Fabric,
				ServerType::Fabric,
			),
			language: Language::AmericanEnglish,
			profile_stability: PackageStability::Latest,
		}
	}
}
