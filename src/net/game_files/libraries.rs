use std::{
	collections::HashMap,
	fs::File,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{anyhow, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};
use zip::ZipArchive;

use crate::{
	data::profile::update::manager::{UpdateManager, UpdateMethodResult},
	io::{
		files::{self, paths::Paths},
		java::classpath::Classpath,
	},
	net::download::FD_SENSIBLE_LIMIT,
	skip_none,
	util::{self, mojang},
};

use super::client_meta::{libraries::Library, ClientMeta};

/// Checks the rules of a game library to see if it should be installed
fn is_allowed(lib: &Library) -> anyhow::Result<bool> {
	for rule in &lib.rules {
		let allowed = mojang::is_allowed(&rule.action.to_string());
		if let Some(os_name) = &rule.os.name {
			if allowed != (os_name.to_string() == util::OS_STRING) {
				return Ok(false);
			}
		}
		if let Some(os_arch) = &rule.os.arch {
			if allowed != (os_arch.to_string() == util::ARCH_STRING) {
				return Ok(false);
			}
		}
	}

	Ok(true)
}

/// Extract the files of a native library into the natives directory.
fn extract_native(
	path: &Path,
	natives_dir: &Path,
	manager: &UpdateManager,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let file = File::open(path)?;
	let mut zip = ZipArchive::new(file)?;
	for i in 0..zip.len() {
		let mut file = zip.by_index(i)?;
		let rel_path = PathBuf::from(
			file.enclosed_name()
				.context("Invalid compressed file path")?,
		);
		if let Some(extension) = rel_path.extension() {
			match extension.to_str() {
				Some("so" | "dylib" | "dll") => {
					let out_path = natives_dir.join(rel_path);
					if !manager.should_update_file(&out_path) {
						continue;
					}
					let mut out_file = File::create(&out_path)?;
					out.files_updated.insert(out_path);
					std::io::copy(&mut file, &mut out_file)
						.context("Failed to copy compressed file")?;
				}
				_ => continue,
			}
		}
	}

	Ok(out)
}

/// Gets the list of allowed libraries from the client JSON
/// and also the number of libraries found.
pub fn get_list(client_json: &ClientMeta) -> anyhow::Result<impl Iterator<Item = &Library>> {
	let libraries = client_json.libraries.iter().filter_map(|lib| {
		if !is_allowed(lib).ok()? {
			None
		} else {
			Some(lib)
		}
	});

	Ok(libraries)
}

/// Downloads base client libraries.
/// Returns a set of files to be added to the update manager.
pub async fn get(
	client_json: &ClientMeta,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let libraries_path = paths.internal.join("libraries");
	files::create_dir_async(&libraries_path).await?;
	let natives_path = paths
		.internal
		.join("versions")
		.join(version)
		.join("natives");
	files::create_dir_async(&natives_path).await?;
	let natives_jars_path = paths.internal.join("natives");

	let mut native_paths = Vec::new();

	let libraries = get_list(client_json)?;

	let mut libs_to_download = Vec::new();

	for lib in libraries {
		if !lib.natives.is_empty() {
			let key = skip_none!(get_natives_classifier_key(&lib.natives));

			let classifier = lib
				.downloads
				.native_classifiers
				.get(&key)
				.ok_or(anyhow!("Native lib artifact does not exist"))?;

			let path = natives_jars_path.join(classifier.path.clone());

			native_paths.push((path.clone(), lib.name.clone()));
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((lib.name.clone(), classifier.clone(), path));
			continue;
		}
		if let Some(artifact) = &lib.downloads.artifact {
			let path = libraries_path.join(artifact.path.clone());
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((lib.name.clone(), artifact.clone(), path));
			continue;
		}
	}

	let count = libs_to_download.len();
	if manager.print.verbose && count > 0 {
		o.display(
			MessageContents::StartProcess(format!("Downloading {count} libraries")),
			MessageLevel::Important,
		);

		o.start_process();
	}

	let client = Client::new();
	let mut join = JoinSet::new();
	let mut num_done = 0;
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	// Clippy complains about num_done, but if we iter().enumerate() the compiler complains
	#[allow(clippy::explicit_counter_loop)]
	for (name, library, path) in libs_to_download {
		o.display(
			MessageContents::Associated(
				format!("{num_done}/{count}"),
				Box::new(MessageContents::StartProcess(format!(
					"Downloading library {name}"
				))),
			),
			MessageLevel::Important,
		);

		files::create_leading_dirs_async(&path).await?;
		out.files_updated.insert(path.clone());

		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(library.url).send();
			let _permit = permit;
			tokio::fs::write(&path, response.await?.error_for_status()?.bytes().await?).await?;
			Ok::<(), anyhow::Error>(())
		};
		join.spawn(fut);
		num_done += 1;
	}

	while let Some(lib) = join.join_next().await {
		lib??;
	}
	o.end_process();

	o.start_process();
	for (path, name) in native_paths {
		o.display(
			MessageContents::StartProcess(format!("Extracting native library {name}")),
			MessageLevel::Important,
		);
		let natives_result = extract_native(&path, &natives_path, manager)
			.with_context(|| format!("Failed to extract native library {name}"))?;
		out.merge(natives_result);
	}

	o.display(
		MessageContents::Success("Libraries downloaded".to_string()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(out)
}

/// Gets the classpath from Minecraft libraries
pub fn get_classpath(client_json: &ClientMeta, paths: &Paths) -> anyhow::Result<Classpath> {
	let natives_jars_path = paths.internal.join("natives");
	let libraries_path = paths.internal.join("libraries");

	let mut classpath = Classpath::new();
	let libraries = get_list(client_json).context("Failed to get list of libraries")?;
	for lib in libraries {
		if !lib.natives.is_empty() {
			let key = skip_none!(get_natives_classifier_key(&lib.natives));

			let classifier = lib
				.downloads
				.native_classifiers
				.get(&key)
				.ok_or(anyhow!("Native lib artifact does not exist"))?;

			let path = natives_jars_path.join(classifier.path.clone());
			classpath.add_path(&path);

			continue;
		}
		if let Some(artifact) = &lib.downloads.artifact {
			let path = libraries_path.join(artifact.path.clone());
			classpath.add_path(&path);
			continue;
		}
	}
	Ok(classpath)
}

/// Get the key for the natives classifier
fn get_natives_classifier_key(classifiers: &HashMap<String, String>) -> Option<String> {
	let key = classifiers
		.get(&format!("natives-{}", util::OS_STRING))
		.unwrap_or(classifiers.get(util::OS_STRING)?);
	let key = key.replace("${arch}", util::TARGET_BITS_STR);

	Some(key)
}
