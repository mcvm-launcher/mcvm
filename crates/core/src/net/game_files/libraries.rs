use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};
use zip::ZipArchive;

use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::io::update::{UpdateManager, UpdateMethodResult};
use crate::net::download::{self, FD_SENSIBLE_LIMIT};
use mcvm_shared::skip_none;
use mcvm_shared::util;

use super::client_meta::libraries::ExtractionRules;
use super::client_meta::{libraries::Library, ClientMeta};

/// Downloads base client libraries.
/// Returns a set of files to be added to the update manager.
pub async fn get(
	client_meta: &ClientMeta,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let libraries_path = paths.internal.join("libraries");
	files::create_dir(&libraries_path)?;
	let natives_path = paths
		.internal
		.join("versions")
		.join(version)
		.join("natives");
	files::create_dir(&natives_path)?;
	let natives_jars_path = paths.internal.join("natives");

	let mut natives = Vec::new();

	let libraries = get_list(client_meta);

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

			natives.push((path.clone(), &lib.name, &lib.extract));
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

	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (name, library, path) in libs_to_download {
		files::create_leading_dirs(&path)?;

		let client = client.clone();
		let permit = sem.clone().acquire_owned().await;
		let path_clone = path.clone();
		let fut = async move {
			let response = download::bytes(library.url, &client)
				.await
				.context("Failed to download library")?;
			let _permit = permit;
			tokio::fs::write(&path_clone, response)
				.await
				.context("Failed to write library file")?;

			Ok::<String, anyhow::Error>(name)
		};
		join.spawn(fut);
		out.files_updated.insert(path.clone());
	}

	o.display(
		MessageContents::Associated(
			Box::new(MessageContents::Progress {
				current: 0,
				total: count as u32,
			}),
			Box::new(MessageContents::Simple(String::new())),
		),
		MessageLevel::Important,
	);
	let mut num_done = 0;
	while let Some(lib) = join.join_next().await {
		let name = lib??;
		num_done += 1;
		o.display(
			MessageContents::Associated(
				Box::new(MessageContents::Progress {
					current: num_done,
					total: count as u32,
				}),
				Box::new(MessageContents::StartProcess(format!(
					"Downloaded library {name}"
				))),
			),
			MessageLevel::Important,
		);
	}

	for (path, name, extract) in natives {
		o.display(
			MessageContents::StartProcess(format!("Extracting native library {name}")),
			MessageLevel::Debug,
		);
		let natives_result = extract_native(&path, &natives_path, extract, manager, o)
			.with_context(|| format!("Failed to extract native library {name}"))?;
		out.merge(natives_result);
	}

	o.display(
		MessageContents::Success("Libraries downloaded".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(out)
}

/// Gets the classpath from Minecraft libraries
pub fn get_classpath(client_meta: &ClientMeta, paths: &Paths) -> anyhow::Result<Classpath> {
	let natives_jars_path = paths.internal.join("natives");
	let libraries_path = paths.internal.join("libraries");

	let mut classpath = Classpath::new();
	let libraries = get_list(client_meta);
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
		.or_else(|| classifiers.get(util::OS_STRING))?;
	let key = key.replace("${arch}", util::TARGET_BITS_STR);

	Some(key)
}

/// Checks the rules of a game library to see if it should be installed
fn is_allowed(lib: &Library) -> bool {
	for rule in &lib.rules {
		let allowed = rule.action.is_allowed();
		if let Some(os_name) = &rule.os.name {
			if allowed != (os_name.to_string() == util::OS_STRING) {
				return false;
			}
		}
		if let Some(os_arch) = &rule.os.arch {
			if allowed != (os_arch.to_string() == util::ARCH_STRING) {
				return false;
			}
		}
	}

	true
}

/// Extract the files of a native library into the natives directory.
fn extract_native(
	path: &Path,
	natives_dir: &Path,
	extraction_rules: &ExtractionRules,
	manager: &UpdateManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let file = File::open(path).context("Failed to open native file")?;
	let mut zip = ZipArchive::new(file).context("Failed to unarchive native")?;
	for i in 0..zip.len() {
		let mut file = zip.by_index(i)?;
		let rel_path = PathBuf::from(
			file.enclosed_name()
				.context("Invalid compressed file path")?,
		);
		if let Some(rel_path_str) = rel_path.to_str() {
			if extraction_rules.exclude.iter().any(|x| x == rel_path_str) {
				continue;
			}
		}
		if let Some(extension) = rel_path.extension() {
			match extension.to_str() {
				Some("so" | "dylib" | "dll") => {
					let out_path = natives_dir.join(rel_path);
					if !manager.should_update_file(&out_path) {
						continue;
					}
					let mut out_file =
						File::create(&out_path).context("Failed to open output file for native")?;
					std::io::copy(&mut file, &mut out_file)
						.context("Failed to copy compressed file")?;
					o.display(
						MessageContents::Simple(format!(
							"Extracted native file {}",
							out_path.to_string_lossy()
						)),
						MessageLevel::Debug,
					);
					out.files_updated.insert(out_path);
				}
				_ => continue,
			}
		}
	}

	Ok(out)
}

/// Gets the list of allowed libraries from the client meta
pub fn get_list(client_meta: &ClientMeta) -> impl Iterator<Item = &Library> {
	client_meta.libraries.iter().filter(|lib| is_allowed(lib))
}
