use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, KeyEvent};
use inquire::{
	MultiSelect, Select, Text,
	validator::{ErrorMessage, StringValidator, Validation},
};
use itertools::Itertools;
use nitrolaunch::{
	config::Config,
	config_crate::instance::is_valid_instance_id,
	core::{account::AccountID, util::versions::MinecraftVersion},
	io::paths::Paths,
	plugin::PluginManager,
	plugin_crate::hook::hooks::AddSupportedLoaders,
	shared::{
		Side,
		id::{InstanceID, TemplateID},
		loaders::Loader,
		output::NoOp,
	},
};

/// Pick which instance to use if the user has not selected one
pub fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceID> {
	if let Some(instance) = instance {
		Ok(instance.into())
	} else {
		let options = config.instances.keys().sorted().collect();
		let selection = Select::new("Choose an instance", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection.to_owned())
	}
}

/// Pick which instances to use
pub fn pick_instances(config: &Config) -> anyhow::Result<Vec<InstanceID>> {
	let options = config.instances.keys().sorted().cloned().collect();

	MultiSelect::new("Choose instances", options)
		.prompt()
		.context("Prompt failed")
}

/// Pick which template to use
pub fn pick_template(template: Option<String>, config: &Config) -> anyhow::Result<TemplateID> {
	if let Some(template) = template {
		Ok(template.into())
	} else {
		let options = config.templates.keys().sorted().collect();
		let selection = Select::new("Choose a template", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection.to_owned())
	}
}

/// Pick which account to use if the user has not selected one
pub fn pick_account(account: Option<String>, config: &Config) -> anyhow::Result<AccountID> {
	if let Some(account) = account {
		Ok(account.into())
	} else {
		let options = config
			.accounts
			.iter_accounts()
			.map(|x| x.0)
			.sorted()
			.collect();
		let selection = Select::new("Choose an account", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection.clone())
	}
}

/// Pick which side to use if the user has not selected one
pub fn pick_side(side: Option<Side>) -> anyhow::Result<Side> {
	if let Some(side) = side {
		Ok(side)
	} else {
		Select::new("Choose a side", vec![Side::Client, Side::Server])
			.prompt()
			.context("Prompt failed")
	}
}

/// Pick which Minecraft version to use
pub async fn pick_minecraft_version(versions: &[String]) -> anyhow::Result<MinecraftVersion> {
	let versions = versions
		.iter()
		.map(|x| MinecraftVersion::Version(x.clone().into()));
	let mut all_versions = vec![MinecraftVersion::Latest, MinecraftVersion::LatestSnapshot];
	all_versions.extend(versions.rev());

	Select::new("Choose a Minecraft version", all_versions)
		.prompt()
		.context("Prompt failed")
}

/// Pick which loader to use if the user has not selected one
pub async fn pick_loader(
	loader: Option<Loader>,
	side: Option<Side>,
	plugins: &PluginManager,
	paths: &Paths,
) -> anyhow::Result<Loader> {
	if let Some(loader) = loader {
		Ok(loader)
	} else {
		let new_loaders = plugins
			.call_hook(AddSupportedLoaders, &(), paths, &mut NoOp)
			.await
			.context("Failed to add loaders")?
			.flatten_all_results(&mut NoOp)
			.await?;

		let mut loaders = vec![Loader::Vanilla];
		loaders.extend(new_loaders);

		if let Some(side) = side {
			loaders.retain(|x| match side {
				Side::Client => x.is_client(),
				Side::Server => x.is_server(),
			});
		}

		Select::new(
			"Choose a loader (More loaders can be added with plugins)",
			loaders,
		)
		.prompt()
		.context("Prompt failed")
	}
}

/// Pick an ID for a new instance
pub fn pick_instance_id() -> anyhow::Result<InstanceID> {
	Ok(Text::new("Type an ID for the instance")
		.with_validator(IDValidator)
		.prompt()?
		.into())
}

#[derive(Clone)]
struct IDValidator;

impl StringValidator for IDValidator {
	fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
		if is_valid_instance_id(input) {
			Ok(Validation::Valid)
		} else {
			Ok(Validation::Invalid(ErrorMessage::Custom(
				"IDs must be lowercase and cannot contain special characters other than - or _"
					.into(),
			)))
		}
	}
}

/// Gets a key
pub fn get_key() -> anyhow::Result<Option<KeyEvent>> {
	if !event::poll(Duration::from_millis(10)).context("Event poll failed")? {
		return Ok(None);
	}

	let event = event::read()
		.context("event read failed")?
		.as_key_press_event();
	let Some(event) = event else {
		return Ok(None);
	};

	Ok(Some(event))
}
