import { useParams } from "@solidjs/router";
import "./ViewPackage.css";
import { invoke } from "@tauri-apps/api";
import {
	createEffect,
	createResource,
	createSignal,
	For,
	JSX,
	Show,
} from "solid-js";
import "@thisbeyond/solid-select/style.css";
import { PackageMeta, PackageProperties } from "../../types";
import { marked } from "marked";
import { errorToast } from "../../components/dialog/Toasts";
import { FooterData } from "../../App";
import { FooterMode } from "../../components/launch/Footer";
import Icon, { HasWidthHeight } from "../../components/Icon";
import {
	Book,
	CurlyBraces,
	Folder,
	Globe,
	Heart,
	Key,
	Picture,
	Text,
	User,
	Warning,
} from "../../icons";
import Modal from "../../components/dialog/Modal";
import PackageLabels from "../../components/package/PackageLabels";

export default function ViewPackage(props: ViewPackageProps) {
	let params = useParams();

	let packageId = params.id;

	let [meta] = createResource(updateMeta);
	let [properties] = createResource(updateProps);

	let [shortDescription, setShortDescription] = createSignal("");
	let [longDescription, setLongDescription] = createSignal("");

	let [selectedTab, setSelectedTab] = createSignal("description");
	let [galleryPreview, setGalleryPreview] = createSignal<string | undefined>();

	createEffect(() => {
		props.setFooterData({
			selectedItem: "",
			mode: FooterMode.InstallPackage,
			action: () => {},
		});
	});

	async function updateMeta() {
		let meta: PackageMeta = await invoke("get_package_meta", {
			package: packageId,
		});

		let description = meta.description == undefined ? "" : meta.description;
		setShortDescription(description.slice(0, 200));
		let longDescription =
			meta.long_description == undefined ? "" : meta.long_description;
		let longDescriptionHtml = `<div>${await marked.parse(
			longDescription
		)}</div>`;
		setLongDescription(longDescriptionHtml);

		return meta;
	}

	async function updateProps() {
		try {
			let props: PackageProperties = await invoke("get_package_props", {
				package: packageId,
			});

			return props;
		} catch (e) {
			errorToast("Failed to load package: " + e);
		}
	}

	return (
		<Show when={meta() != undefined && properties() != undefined}>
			<div class="cont col" style="width:100%">
				<div class="cont col" id="package-container">
					<div class="cont" id="package-header-container">
						<div class="package-shadow" id="package-header">
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
								<Show when={meta()!.categories != undefined}>
									<PackageLabels categories={meta()!.categories!} />
								</Show>
							</div>
						</div>
					</div>
					<Show when={meta()!.banner != undefined}>
						<div id="package-banner-container">
							<img src={meta()!.banner} id="package-banner" />
							<div id="package-banner-gradient"></div>
						</div>
					</Show>
					<div id="package-contents">
						<div id="package-body">
							<div class="package-shadow" id="package-tabs">
								<div
									class={`cont package-tab ${
										selectedTab() == "description" ? "selected" : ""
									}`}
									onclick={() => setSelectedTab("description")}
								>
									<Icon icon={Text} size="1rem" />
									Description
								</div>
								<div
									class={`cont package-tab ${
										selectedTab() == "versions" ? "selected" : ""
									}`}
									onclick={() => setSelectedTab("versions")}
								>
									<Icon icon={Folder} size="1rem" />
									Versions
								</div>
								<div
									class={`cont package-tab ${
										selectedTab() == "gallery" ? "selected" : ""
									}`}
									onclick={() => setSelectedTab("gallery")}
								>
									<Icon icon={Picture} size="1rem" />
									Gallery
								</div>
							</div>
							<div class="cont col package-shadow" id="package-tab-contents">
								<Show when={selectedTab() == "description"}>
									<div
										class="cont col"
										id="package-description"
										innerHTML={longDescription()}
									></div>
								</Show>
								<Show
									when={
										selectedTab() == "gallery" && meta()!.gallery != undefined
									}
								>
									<div class="cont">
										<div id="package-gallery">
											<For each={meta()!.gallery!}>
												{(entry) => (
													<img
														class="package-gallery-entry"
														src={entry}
														onclick={() => setGalleryPreview(entry)}
													/>
												)}
											</For>
										</div>
									</div>
									<Modal
										width="55rem"
										visible={galleryPreview() != undefined}
										onClose={() => setGalleryPreview(undefined)}
									>
										<img id="package-gallery-preview" src={galleryPreview()} />
									</Modal>
								</Show>
							</div>
						</div>
						<div class="package-shadow cont col" id="package-properties">
							<Show when={meta()!.website != undefined}>
								<Property icon={Globe} label="Website">
									<a href={meta()!.website} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Show when={meta()!.support_link != undefined}>
								<Property icon={Heart} label="Donate" color="var(--error)">
									<a href={meta()!.support_link} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Show when={meta()!.documentation != undefined}>
								<Property icon={Book} label="Documentation">
									<a href={meta()!.documentation} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Show when={meta()!.source != undefined}>
								<Property icon={CurlyBraces} label="Source">
									<a href={meta()!.source} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Show when={meta()!.issues != undefined}>
								<Property icon={Warning} label="Issue Tracker">
									<a href={meta()!.issues} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Show when={meta()!.community != undefined}>
								<Property icon={User} label="Community">
									<a href={meta()!.community} target="_blank">
										Open
									</a>
								</Property>
							</Show>
							<Property icon={Key} label="License">
								{meta()!.license == undefined ? (
									"Unknown"
								) : meta()!.license!.startsWith("http") ? (
									<a href={meta()!.license} target="_blank">
										Open
									</a>
								) : (
									meta()!.license
								)}
							</Property>
						</div>
					</div>
				</div>
				<br />
				<br />
				<br />
			</div>
		</Show>
	);
}

export interface ViewPackageProps {
	setFooterData: (data: FooterData) => void;
}

function Property(props: PropertyProps) {
	let color = props.color == undefined ? "var(--fg)" : props.color;

	return (
		<div class="package-property">
			<div class="cont package-property-icon" style={`color:${color}`}>
				<Icon icon={props.icon} size="1rem" />
			</div>
			<div class="cont package-property-label">{props.label}</div>
			<div class="cont package-property-value">{props.children}</div>
		</div>
	);
}

interface PropertyProps {
	icon: (props: HasWidthHeight) => JSX.Element;
	label: string;
	children: JSX.Element;
	color?: string;
}
