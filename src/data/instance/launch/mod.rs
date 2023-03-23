pub mod client;
pub mod server;

use color_print::cprintln;

use crate::data::profile::update::UpdateManager;
use crate::data::{instance::InstKind, user::Auth};
use crate::io::files::paths::Paths;
use crate::io::java::args::ArgsPreset;
use crate::io::java::{
	args::{MemoryArg, MemoryNum},
	JavaKind,
};
use crate::util::json;
use crate::util::print::PrintOptions;

use super::{create::CreateError, Instance};

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
	#[error("Failed to create instance:\n{}", .0)]
	Create(#[from] CreateError),
	#[error("Java is not installed")]
	Java,
	#[error("Game process failed:\n{}", .0)]
	Command(std::io::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	Json(#[from] json::JsonError),
}

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		paths: &Paths,
		auth: &Auth,
	) -> Result<(), LaunchError> {
		cprintln!("Checking for updates...");
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false);
		manager.add_requirements(self.get_requirements());
		manager.fulfill_requirements(paths, &self.version).await?;
		
		self.create(&manager, paths).await?;
		cprintln!("<g>Launching!");
		match &self.kind {
			InstKind::Client => {
				self.launch_client(paths, auth)?;
			}
			InstKind::Server => {
				self.launch_server(paths)?;
			}
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct LaunchOptions {
	pub java: JavaKind,
	pub jvm_args: Vec<String>,
	pub game_args: Vec<String>,
	pub min_mem: Option<MemoryNum>,
	pub max_mem: Option<MemoryNum>,
	pub preset: ArgsPreset
}

impl LaunchOptions {
	/// Create the args for the JVM when launching the game
	pub fn generate_jvm_args(&self) -> Vec<String> {
		let mut out = self.jvm_args.clone();
		if let Some(n) = &self.min_mem {
			out.push(MemoryArg::Min.to_string(n.clone()));
		}
		if let Some(n) = &self.max_mem {
			out.push(MemoryArg::Max.to_string(n.clone()));
		}

		let avg = match &self.min_mem {
			Some(min) => match &self.max_mem {
				Some(max) => Some(MemoryNum::avg(min.clone(), max.clone())),
				None => None
			}
			None => None	
		};
		out.extend(self.preset.generate_args(avg));

		out
	}
}
