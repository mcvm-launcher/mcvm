use std::{
	fs::File,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};
use zip::ZipArchive;

use crate::{
	data::profile::update::{UpdateManager, UpdateMethodResult},
	io::{
		files::{self, paths::Paths},
		java::classpath::Classpath,
	},
	net::download::FD_SENSIBLE_LIMIT,
	util::{
		self,
		json::{self, JsonObject, JsonType},
		mojang,
	},
};

/// Checks the rules of a game library to see if it should be installed
fn is_allowed(lib: &JsonObject) -> anyhow::Result<bool> {
	if let Some(rules) = lib.get("rules") {
		let rules = json::ensure_type(rules.as_array(), JsonType::Arr)?;
		for rule in rules.iter() {
			let rule = json::ensure_type(rule.as_object(), JsonType::Obj)?;
			let action = json::access_str(rule, "action")?;
			if let Some(os) = rule.get("os") {
				let os = json::ensure_type(os.as_object(), JsonType::Obj)?;
				let os_name = json::access_str(os, "name")?;
				let allowed = mojang::is_allowed(action);
				if allowed != (os_name == util::OS_STRING) {
					return Ok(false);
				}
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
pub fn get_list(
	client_json: &json::JsonObject,
) -> anyhow::Result<impl Iterator<Item = &JsonObject>> {
	let libraries = json::access_array(client_json, "libraries")?;
	let libraries = libraries.iter().filter_map(|lib| {
		let lib = json::ensure_type(lib.as_object(), JsonType::Obj).ok()?;
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
	client_json: &json::JsonObject,
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
		let name = json::access_str(lib, "name")?;
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives) = lib.get("natives") {
			let natives = json::ensure_type(natives.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, util::OS_STRING)?
				.replace("${arch}", util::TARGET_BITS_STR);
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, &key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);

			native_paths.push((path.clone(), name.to_owned()));
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((name, classifier.clone(), path));
			continue;
		}
		if let Some(artifact) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((name, artifact.clone(), path));
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
		let url = json::access_str(&library, "url")?.to_owned();

		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(url).send();
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
pub fn get_classpath(client_json: &json::JsonObject, paths: &Paths) -> anyhow::Result<Classpath> {
	let natives_jars_path = paths.internal.join("natives");
	let libraries_path = paths.internal.join("libraries");

	let mut classpath = Classpath::new();
	let libraries = get_list(client_json).context("Failed to get list of libraries")?;
	for lib in libraries {
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives) = lib.get("natives") {
			let natives = json::ensure_type(natives.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, util::OS_STRING)?
				.replace("${arch}", util::TARGET_BITS_STR);
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, &key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);
			classpath.add_path(&path);

			continue;
		}
		if let Some(artifact) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			classpath.add_path(&path);
			continue;
		}
	}
	Ok(classpath)
}
