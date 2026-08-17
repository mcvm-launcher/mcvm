use std::path::PathBuf;

use anyhow::Context;
use nitrolaunch::{
	config::modifications::{ConfigModification, apply_modifications_and_write},
	instance::{
		Instance,
		transfer::{load_formats, migrate_instances},
	},
	io::lock::Lockfile,
	plugin_crate::hook::hooks::{
		AddInstanceTransferFormats, CheckMigrationResult, InstanceTransferFormat,
	},
	shared::{Side, id::InstanceID},
};

use crate::{
	dependency::BackDependency, ops::task::Task, prelude::*, simple_mutation, simple_query,
};

simple_query!(
	name = FetchTransferFormats,
	ok = Vec<InstanceTransferFormat>,
	err = anyhow::Error,
	keys = (),
	fn run(&self, _keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move {
			let mut o = back_state.output();
			back_state
				.plugins
				.call_hook(AddInstanceTransferFormats, &(), &back_state.paths, &mut o)
				.await?
				.flatten_all_results(&mut o)
				.await
		})
	}
);

simple_mutation!(
	name = ImportInstance,
	ok = (),
	err = anyhow::Error,
	keys = ImportInstanceKeys,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (id, format, path, side) = (
			keys.id.clone(),
			keys.format.clone(),
			keys.path.clone(),
			keys.side,
		);

		let task = async move {
			let mut o = back_state.output();
			o.set_task(Task::ImportInstance);

			let formats = load_formats(&back_state.plugins, &back_state.paths, &mut o).await?;
			let version_manifest = back_state.versions().await?;

			let instance = Instance::import(
				&id,
				&format,
				&path,
				side,
				&formats,
				&version_manifest.list,
				&back_state.plugins,
				&back_state.paths,
				&mut o,
			)
			.await?;

			let mut raw_config = back_state.raw_config().await?;
			apply_modifications_and_write(
				&mut raw_config,
				vec![ConfigModification::AddInstance(id.into(), instance)],
				&back_state.paths,
				&back_state.plugins,
				&mut o,
			)
			.await?;

			back_state.invalidate(BackDependency::Items);

			Ok(())
		};

		self.back_state
			.register_task(Task::ImportInstance, tokio::spawn(task));
		async { Ok(()) }
	}
);

simple_mutation!(
	name = MigrateInstances,
	ok = (),
	err = anyhow::Error,
	keys = MigrateInstancesKeys,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (format, link, instances) = (keys.format.clone(), keys.link, keys.instances.clone());

		let task = async move {
			let mut o = back_state.output();
			o.set_task(Task::MigrateInstances);

			let formats = load_formats(&back_state.plugins, &back_state.paths, &mut o).await?;

			let instances = Some(instances).filter(|x| !x.is_empty());
			let new_instances = migrate_instances(
				&format,
				instances,
				link,
				&formats,
				&back_state.plugins,
				&back_state.paths,
				&mut o,
			)
			.await?;

			let modifications = new_instances
				.into_iter()
				.map(|(id, instance)| ConfigModification::AddInstance(id.into(), instance))
				.collect::<Vec<_>>();

			let mut raw_config = back_state.raw_config().await?;
			apply_modifications_and_write(
				&mut raw_config,
				modifications,
				&back_state.paths,
				&back_state.plugins,
				&mut o,
			)
			.await?;

			back_state.invalidate(BackDependency::Items);

			Ok(())
		};

		self.back_state
			.register_task(Task::MigrateInstances, tokio::spawn(task));
		async { Ok(()) }
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MigrateInstancesKeys {
	pub format: String,
	pub link: bool,
	pub instances: Vec<String>,
}

simple_query!(
	name = CheckMigration,
	ok = CheckMigrationResult,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let format = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let mut o = back_state.output();
			let result = back_state
				.plugins
				.call_hook(
					nitrolaunch::plugin_crate::hook::hooks::CheckMigration,
					&format,
					&back_state.paths,
					&mut o,
				)
				.await?;
			Ok(result.first_some(&mut o).await?.unwrap_or_default())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImportInstanceKeys {
	pub format: String,
	pub path: PathBuf,
	pub id: String,
	pub side: Option<Side>,
}

simple_mutation!(
	name = ExportInstance,
	ok = (),
	err = anyhow::Error,
	keys = ExportInstanceKeys,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (id, format, path) = (keys.id.clone(), keys.format.clone(), keys.path.clone());

		let task = async move {
			let config = back_state.config().await?;
			let mut o = back_state.output();
			o.set_task(Task::ExportInstance);

			let formats = load_formats(&back_state.plugins, &back_state.paths, &mut o).await?;
			let lock = Lockfile::open(&back_state.paths)?;

			let instance = config
				.instances
				.get(&InstanceID::from(id))
				.context("Instance does not exist")?;

			instance
				.export(
					&format,
					&path,
					&formats,
					&back_state.plugins,
					&lock,
					&back_state.paths,
					&mut o,
				)
				.await?;

			Ok(())
		};

		self.back_state
			.register_task(Task::ExportInstance, tokio::spawn(task));
		async { Ok(()) }
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ExportInstanceKeys {
	pub format: String,
	pub id: String,
	pub path: PathBuf,
}
