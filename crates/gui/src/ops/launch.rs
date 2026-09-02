use std::io::Write;

use anyhow::Context;
use nitrolaunch::{
	core::{QuickPlayType, io::open_named_pipe},
	instance::{
		launch::LaunchSettings,
		tracking::RunningInstanceEntry,
		update::{InstanceUpdateContext, manager::UpdateSettings},
	},
	io::lock::Lockfile,
	shared::{UpdateDepth, id::InstanceID},
};

use crate::{
	data::LauncherData,
	ops::{MakeSend, task::Task},
	prelude::*,
	secrets::get_ms_client_id,
	simple_mutation, simple_query,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LaunchInstance {
	pub back_state: Captured<BackState>,
}

#[derive(Clone, PartialEq, Hash)]
pub struct LaunchInstanceParams {
	pub id: String,
	pub offline: bool,
	pub quick_play: QuickPlayType,
}

impl LaunchInstance {
	pub fn new(back_state: BackState) -> Mutation<Self> {
		Mutation::new(Self {
			back_state: Captured(back_state),
		})
	}
}

impl MutationCapability for LaunchInstance {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = LaunchInstanceParams;

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = keys.id.clone();
		let offline = keys.offline;
		let quick_play = keys.quick_play.clone();
		let back_state = self.back_state.clone();

		let task = async move {
			let mut config = back_state.config().await?;
			let mut output = back_state.output();
			output.set_task(Task::LaunchInstance(id.clone()));

			let data = LauncherData::open(&back_state.paths)?;
			if let Some(account) = data.current_account {
				let _ = config.accounts.choose_account(&account);
			}

			let core = config
				.get_core(
					Some(&get_ms_client_id()),
					&UpdateSettings {
						depth: UpdateDepth::Shallow,
						offline_auth: offline,
					},
					&back_state.client,
					&config.plugins,
					&back_state.paths,
					&mut output,
				)
				.await?;

			let instance = config
				.instances
				.get_mut(&InstanceID::from(id))
				.context("Instance does not exist")?;

			let settings = LaunchSettings {
				offline_auth: offline,
				pipe_stdin: false,
				quick_play: Some(quick_play),
			};

			let mut lock = Lockfile::open(&back_state.paths)?;
			let mut ctx = InstanceUpdateContext {
				packages: &config.packages,
				accounts: &mut config.accounts,
				plugins: &config.plugins,
				prefs: &config.prefs,
				paths: &back_state.paths,
				lock: &mut lock,
				client: &back_state.client,
				output: &mut output,
				core: &core,
			};

			let mut handle = instance
				.launch(settings, &mut ctx)
				.await
				.context("Failed to launch instance")?;

			handle.silence_output(true);
			output.finish_task();

			handle
				.wait(&config.plugins, &back_state.paths, &mut output)
				.await?;

			Ok(())
		};

		let task = unsafe { MakeSend::new(task) };
		self.back_state
			.register_task(Task::LaunchInstance(keys.id.clone()), tokio::spawn(task));

		async { Ok(()) }
	}
}

/// Only for the initial fetch. Events will be used afterwards.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchRunningInstances {
	back_state: Captured<BackState>,
}

impl FetchRunningInstances {
	pub fn new(back_state: BackState) -> Query<Self> {
		Query::new(
			(),
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchRunningInstances {
	type Ok = Vec<RunningInstanceEntry>;
	type Err = anyhow::Error;
	type Keys = ();

	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move {
			Ok(back_state.running_instances.get_running_instances().await)
		})
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KillInstance {
	back_state: Captured<BackState>,
}

impl KillInstance {
	pub fn new(back_state: BackState) -> Mutation<Self> {
		Mutation::new(Self {
			back_state: Captured(back_state),
		})
	}
}

impl MutationCapability for KillInstance {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = (String, Option<String>);

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = keys.0.clone();
		let account = keys.1.clone();
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move {
			let _: () = back_state
				.running_instances
				.kill(&id, account.as_deref())
				.await;
			Ok(())
		})
	}
}

simple_query!(
	name = FetchInstanceRunState,
	ok = InstanceRunState,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let keys = keys.clone();

		async move {
			if back_state
				.running_instances
				.get_entry(&keys, None)
				.await
				.is_some()
			{
				Ok(InstanceRunState::Running)
			} else {
				Ok(InstanceRunState::Stopped)
			}
		}
	}
);

simple_mutation!(
	name = WriteInstanceInput,
	ok = (),
	err = anyhow::Error,
	keys = (String, String),
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (id, input) = keys.clone();

		async move {
			let Some(entry) = back_state.running_instances.get_entry(&id, None).await else {
				return Ok(());
			};

			let Some(path) = &entry.stdin_file else {
				return Ok(());
			};
			let path = back_state.paths.internal.join("stdio").join(path);

			let mut file = open_named_pipe(path).context("Failed to open input pipe")?;
			file.write_all(input.as_bytes())
				.context("Failed to write to instance input")?;

			Ok(())
		}
	}
);

#[derive(Clone, Copy, PartialEq, Default)]
pub enum InstanceRunState {
	#[default]
	Stopped,
	Running,
}
