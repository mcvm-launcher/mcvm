use std::{path::PathBuf, process::Child};

use anyhow::Context;
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::io::java::install::{JavaInstallation, JavaInstallationKind};
use mcvm_core::io::java::JavaMajorVersion;
use mcvm_core::launch::{
	launch_process, LaunchConfiguration, LaunchProcessParameters, LaunchProcessProperties,
};
use mcvm_core::user::UserManager;
use mcvm_mods::paper;
use mcvm_shared::lang::translate::TranslationKey;
use mcvm_shared::modifications::Proxy;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use reqwest::Client;

use crate::io::files::paths::Paths;

use super::{update::manager::UpdateManager, Profile};

impl Profile {
	/// Create the profile's proxy, if it has one
	pub async fn create_proxy(
		&mut self,
		manager: &mut UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		o.start_process();
		o.display(
			MessageContents::StartProcess(translate!(o, StartUpdatingProxy)),
			MessageLevel::Important,
		);

		// Create the proxy dir
		self.get_and_create_proxy_dir(paths).await?;

		match self.modifications.proxy {
			Proxy::Velocity => {
				let (jar_path, main_class) = paper::install_velocity(&paths.core, client)
					.await
					.context("Failed to install Velocity")?;

				let java = manager
					.core
					.get_mut()
					.get_java_installation(JavaMajorVersion::new(17), JavaInstallationKind::Auto, o)
					.await
					.context("Failed to install Java for proxy")?;

				self.proxy_props.fill(ProxyProperties {
					jar_path,
					main_class,
					java,
				});
			}
			_ => {}
		}

		o.display(
			MessageContents::Success(translate!(o, FinishUpdatingProxy)),
			MessageLevel::Important,
		);
		o.end_process();

		Ok(())
	}

	/// Launch the profile's proxy, if it has one, returning the child process
	pub async fn launch_proxy(
		&mut self,
		client: &Client,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<Child>> {
		// Check for updates first
		let mut manager = UpdateManager::new(false, true);
		manager
			.fulfill_requirements(
				&UserManager::new(ClientId::new(String::new())),
				paths,
				client,
				o,
			)
			.await
			.context("Failed to fulfill update manager")?;
		self.create_proxy(&mut manager, paths, client, o)
			.await
			.context("Failed to check for proxy updates")?;

		o.display(
			MessageContents::Simple(translate!(o, Launch)),
			MessageLevel::Important,
		);

		let child = match self.modifications.proxy {
			Proxy::None => None,
			_ => {
				let dir = self.get_and_create_proxy_dir(paths).await?;
				let props = self.proxy_props.get();
				let jvm_path = props.java.get_jvm_path();

				let proc_props = LaunchProcessProperties {
					jvm_args: vec!["-jar".into(), props.jar_path.to_string_lossy().into()],
					..Default::default()
				};
				let params = LaunchProcessParameters {
					cwd: &dir,
					command: jvm_path.as_os_str(),
					main_class: Some(&props.main_class),
					launch_config: &LaunchConfiguration::default(),
					props: proc_props,
				};

				launch_process(params).context("Failed to launch Velocity child process")?;

				None
			}
		};

		Ok(child)
	}

	/// Gets the directory for this profile's proxy and creates it
	async fn get_and_create_proxy_dir(&self, paths: &Paths) -> anyhow::Result<PathBuf> {
		let path = paths.proxy.join(self.id.to_string());
		tokio::fs::create_dir_all(&path)
			.await
			.context("Failed to create profile proxy dir")?;

		Ok(path)
	}
}

/// Properties for a proxy
#[derive(Debug)]
pub struct ProxyProperties {
	jar_path: PathBuf,
	main_class: String,
	java: JavaInstallation,
}
