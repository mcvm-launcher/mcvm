use anyhow::anyhow;
use mcvm_pkg::declarative::{DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage};
use mcvm_shared::pkg::PkgIdentifier;

use super::{
	conditions::check_os_condition, create_valid_addon_request, EvalData, EvalInput,
	RequiredPackage, Routine,
};

/// Evaluate a declarative package
pub fn eval_declarative_package<'a>(
	id: PkgIdentifier,
	contents: &DeclarativePackage,
	input: EvalInput<'a>,
	routine: Routine,
) -> anyhow::Result<EvalData<'a>> {
	let pkg_id = id;

	let mut eval_data = EvalData::new(input, pkg_id.clone(), &routine);

	// Vars for the EvalData that are modified by conditions / versions
	let mut relations = contents.relations.clone();
	let mut notices = Vec::new();

	// Select addon versions
	for (addon_id, addon) in &contents.addons {
		// Check conditions
		if !check_multiple_condition_sets(&addon.conditions, &eval_data.input) {
			continue;
		}

		// Pick the best version
		let version = pick_best_addon_version(&addon.versions, &eval_data.input);
		let version = version.ok_or(anyhow!("No valid addon version found"))?;

		let addon_req = create_valid_addon_request(
			addon_id.clone(),
			version.url.clone(),
			version.path.clone(),
			addon.kind,
			version.filename.clone(),
			pkg_id.clone(),
			version.version.clone(),
			&eval_data.input.params.perms,
		)?;

		eval_data.addon_reqs.push(addon_req);

		relations.merge(version.relations.clone());
		notices.extend(version.notices.get_vec());
	}

	// Apply conditional rules
	for rule in &contents.conditional_rules {
		for condition in &rule.conditions {
			if !check_condition_set(condition, &eval_data.input) {
				continue;
			}
		}

		relations.merge(rule.properties.relations.clone());
		notices.extend(rule.properties.notices.get_vec());
	}

	eval_data
		.deps
		.extend(relations.dependencies.get_vec().iter().map(|x| {
			vec![RequiredPackage {
				value: x.clone(),
				explicit: false,
			}]
		}));
	eval_data
		.deps
		.extend(relations.explicit_dependencies.get_vec().iter().map(|x| {
			vec![RequiredPackage {
				value: x.clone(),
				explicit: true,
			}]
		}));
	eval_data.conflicts.extend(relations.conflicts.get_vec());
	eval_data.extensions.extend(relations.extensions.get_vec());
	eval_data.bundled.extend(relations.bundled.get_vec());
	eval_data.compats.extend(relations.compats.get_vec());
	eval_data
		.recommendations
		.extend(relations.recommendations.get_vec());

	eval_data.notices.extend(notices);

	Ok(eval_data)
}

/// Pick the best addon version from a list of declarative addon versions
pub fn pick_best_addon_version<'a>(
	versions: &'a [DeclarativeAddonVersion],
	input: &'a EvalInput<'a>,
) -> Option<&'a DeclarativeAddonVersion> {
	// Filter versions that are not allowed
	let mut versions = versions
		.iter()
		.filter(|x| check_condition_set(&x.conditional_properties, input));

	versions.next()
}

/// Check multiple setso f addon version conditions
fn check_multiple_condition_sets<'a>(
	conditions: &[DeclarativeConditionSet],
	input: &'a EvalInput<'a>,
) -> bool {
	conditions.iter().all(|x| check_condition_set(x, input))
}

/// Filtering function for addon version picking and rule checking
fn check_condition_set<'a>(conditions: &DeclarativeConditionSet, input: &'a EvalInput<'a>) -> bool {
	if let Some(minecraft_versions) = &conditions.minecraft_versions {
		if !minecraft_versions
			.get_vec()
			.iter()
			.any(|x| x.matches_single(&input.constants.version, &input.constants.version_list))
		{
			return false;
		}
	}

	if let Some(side) = conditions.side {
		if side != input.params.side {
			return false;
		}
	}

	if let Some(modloaders) = &conditions.modloaders {
		if !modloaders.get_vec().iter().any(|x| {
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
			.get_vec()
			.iter()
			.any(|x| x.matches(&input.constants.modifications.server_type))
		{
			return false;
		}
	}

	if let Some(stability) = &conditions.stability {
		if stability != &input.params.stability {
			return false;
		}
	}

	if let Some(features) = &conditions.features {
		for feature in features.get_vec() {
			if !input.params.features.contains(&feature) {
				return false;
			}
		}
	}

	if let Some(os) = &conditions.os {
		if !check_os_condition(os) {
			return false;
		}
	}

	if let Some(language) = &conditions.language {
		if language != &input.constants.language {
			return false;
		}
	}

	true
}

#[cfg(test)]
mod tests {
	use mcvm_pkg::declarative::deserialize_declarative_package;
	use mcvm_shared::{
		instance::Side,
		lang::Language,
		modifications::{ClientType, Modloader, ServerType},
		pkg::PackageStability,
	};

	use crate::{
		data::config::profile::GameModifications,
		package::eval::{EvalConstants, EvalParameters, EvalPermissions, RequiredPackage},
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

		let constants = EvalConstants {
			version: String::from("1.19.2"),
			version_list: vec!["1.19.2".to_string(), "1.19.3".to_string()],
			modifications: GameModifications::new(
				Modloader::Fabric,
				ClientType::Fabric,
				ServerType::Fabric,
			),
			language: Language::AmericanEnglish,
		};
		let input = EvalInput {
			constants: &constants,
			params: EvalParameters {
				side: Side::Client,
				features: vec![],
				perms: EvalPermissions::Standard,
				stability: PackageStability::Stable,
			},
		};

		let eval =
			eval_declarative_package(PkgIdentifier::new("foo", 1), &pkg, input, Routine::Install)
				.unwrap();

		let addon = eval.addon_reqs.first().unwrap();
		assert_eq!(addon.addon.version, Some(String::from("2")));

		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: String::from("foo"),
			explicit: false
		}]));
		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: String::from("bar"),
			explicit: false
		}]));
		assert!(eval.deps.contains(&vec![RequiredPackage {
			value: String::from("baz"),
			explicit: false
		}]));
	}
}
