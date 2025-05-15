import { useParams } from "@solidjs/router";
import "./ViewPackage.css";
import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, Show } from "solid-js";
import "@thisbeyond/solid-select/style.css";
import { PackageMeta, PackageProps } from "../../types";
import { marked } from "marked";

export default function ViewPackage() {
	let params = useParams();

	let packageId = params.id;

	let [meta] = createResource(updateMeta);
	let [props] = createResource(updateProps);

	let [shortDescription, setShortDescription] = createSignal("");
	let [longDescription, setLongDescription] = createSignal("");

	async function updateMeta() {
		let meta: PackageMeta = await invoke("get_package_meta", {
			package: packageId,
		});

		let description = meta.description == undefined ? "" : meta.description;
		setShortDescription(description.slice(0, 150));
		let longDescription =
			meta.long_description == undefined ? "" : meta.long_description;
		let longDescriptionHtml = `<div>${await marked.parse(
			longDescription
		)}</div>`;
		setLongDescription(longDescriptionHtml);

		return meta;
	}

	async function updateProps() {
		let props: PackageProps = await invoke("get_package_props", {
			package: packageId,
		});

		return props;
	}

	return (
		<Show when={meta() != undefined && props() != undefined}>
			<div class="cont col" style="width:100%">
				<div class="cont" id="package-header-container">
					<div id="package-header">
						<div class="cont" id="package-icon">
							<img
								id="package-icon-image"
								src={
									meta()?.icon == undefined
										? "/icons/default_instance.png"
										: meta()!.icon
								}
							/>
						</div>
						<div class="col" id="package-details">
							<div class="cont" id="package-upper-details">
								<div id="package-name">{meta()!.name}</div>
								<div id="package-id">{packageId}</div>
							</div>
							<div class="cont" id="package-short-description">
								{shortDescription()}
							</div>
						</div>
					</div>
				</div>
				<div
					class="cont col"
					id="package-description"
					innerHTML={longDescription()}
				></div>
				<br />
				<br />
				<br />
			</div>
		</Show>
	);
}
