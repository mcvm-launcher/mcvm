import { useLocation, useParams } from "@solidjs/router";
import "./BrowsePackages.css";
import { invoke } from "@tauri-apps/api";
import {
	createEffect,
	createResource,
	createSignal,
	For,
	Show,
} from "solid-js";
import "@thisbeyond/solid-select/style.css";
import PageButtons from "../../components/input/PageButtons";
import { PackageMeta, PackageProperties } from "../../types";
import SearchBar from "../../components/input/SearchBar";
import { parseQueryString } from "../../utils";
import InlineSelect from "../../components/input/InlineSelect";
import { FooterData } from "../../App";
import { FooterMode } from "../../components/launch/Footer";
import { errorToast, warningToast } from "../../components/dialog/Toasts";
import PackageLabels from "../../components/package/PackageLabels";

const PACKAGES_PER_PAGE = 12;

export default function BrowsePackages(props: BrowsePackagesProps) {
	let params = useParams();
	let searchParams = parseQueryString(useLocation().search);

	createEffect(() => {
		props.setFooterData({
			mode: FooterMode.PreviewPackage,
			selectedItem: undefined,
			action: () => {},
		});
	});

	let page = +params.page;
	let search = searchParams["search"];
	let repo = searchParams["repo"];

	let [packages, _] = createResource(updatePackages);

	let [repos, setRepos] = createSignal<RepoInfo[] | undefined>(undefined);
	let [packageCount, setPackageCount] = createSignal(0);

	let selectedRepo = () => {
		if (repos() == undefined) {
			return undefined;
		}
		if (repo == undefined) {
			if (repos()!.some((x) => x.id == "std")) {
				return "std";
			}
			return undefined;
		}
		return repo;
	};

	async function updatePackages() {
		let repos: RepoInfo[] = [];
		try {
			repos = await invoke("get_package_repos");
		} catch (e) {
			errorToast("Failed to get available repos: " + e);
			return undefined;
		}

		if (repos.length == 0) {
			warningToast("No repositories available");
		}

		let index = repos.findIndex((x) => x.id == "core");
		if (index != -1) {
			repos.splice(index, 1);
		}
		setRepos(repos);

		try {
			let [packagesToRequest, packageCount] = (await invoke("get_packages", {
				repo: selectedRepo(),
				page: page,
				search: search,
			})) as [string[], number];

			setPackageCount(packageCount);

			let promises = [];

			try {
				await invoke("preload_packages", {
					packages: packagesToRequest,
					repo: selectedRepo(),
				});
			} catch (e) {
				errorToast("Failed to load packages: " + e);
			}

			for (let pkg of packagesToRequest) {
				promises.push(
					(async () => {
						try {
							let meta = await invoke("get_package_meta", { package: pkg });
							let props = await invoke("get_package_props", {
								package: pkg,
							});
							return [meta, props];
						} catch (e) {
							console.error(e);
							return "error";
						}
					})()
				);
			}

			try {
				let finalPackages = (await Promise.all(promises)) as (
					| [PackageMeta, PackageProps]
					| string
				)[];
				let packagesAndIds = finalPackages.map((val, i) => {
					if (val == "error") {
						return "error";
					} else {
						let [meta, props] = val;
						return {
							id: packagesToRequest[i],
							meta: meta,
							props: props,
						} as PackageData;
					}
				});
				return packagesAndIds;
			} catch (e) {
				errorToast("Failed to load some packages: " + e);
			}
		} catch (e) {
			errorToast("Failed to search packages: " + e);
		}
	}

	let [selectedPackage, setSelectedPackage] = createSignal<string | undefined>(
		undefined
	);

	return (
		<div class="cont col" style="width:100%">
			<div id="header">
				<div class="cont">
					<Show when={repos() != undefined}>
						<div class="cont" style="width:18rem">
							<InlineSelect
								options={repos()!.map((x) => {
									return {
										value: x.id,
										contents: (
											<div style="padding:0rem 0.3rem">
												{x.id.replace(/\_/g, " ").toLocaleUpperCase()}
											</div>
										),
										color: x.meta.color,
									};
								})}
								connected={false}
								grid={false}
								selected={selectedRepo()}
								columns={repos()!.length}
								onChange={(x) => {
									if (x != undefined) {
										window.location.href = formatUrl(0, x, search);
									}
								}}
								optionClass="repo"
								solidSelect={true}
							/>
						</div>
					</Show>
				</div>
				<h1 class="noselect">Packages</h1>
				<div class="cont">
					<SearchBar
						placeholder="Search for packages..."
						value={search}
						method={(term) => {
							window.location.href = formatUrl(0, selectedRepo(), term);
						}}
					/>
				</div>
			</div>
			<div id="packages-container">
				<For each={packages()}>
					{(data) => {
						if (data == "error") {
							return (
								<div class="cont package package-error">Error with package</div>
							);
						} else {
							return (
								<Package
									id={data.id}
									meta={data.meta}
									selected={selectedPackage()}
									onSelect={(pkg) => {
										setSelectedPackage(pkg);
										props.setFooterData({
											mode: FooterMode.PreviewPackage,
											selectedItem: "",
											action: () => {
												window.location.href = `/packages/package/${data.id}`;
											},
										});
									}}
								/>
							);
						}
					}}
				</For>
			</div>
			<PageButtons
				page={page}
				pageCount={Math.floor(packageCount() / PACKAGES_PER_PAGE)}
				pageFunction={(page) => {
					window.location.href = formatUrl(page, selectedRepo(), search);
				}}
			/>
			<br />
			<br />
			<br />
		</div>
	);
}

function Package(props: PackageProps) {
	let image =
		props.meta.banner == undefined
			? props.meta.gallery == undefined || props.meta.gallery!.length == 0
				? props.meta.icon == undefined
					? "/icons/default_instance.png"
					: props.meta.icon
				: props.meta.gallery![0]
			: props.meta.banner;

	let isSelected = () => props.selected == props.id;

	return (
		<div
			class={`cont col input-shadow package ${isSelected() ? "selected" : ""}`}
			style="cursor:pointer"
			onclick={() => {
				// Double click to open
				if (isSelected()) {
					window.location.href = `/packages/package/${props.id}`;
				} else {
					props.onSelect(props.id);
				}
			}}
		>
			<div class="package-inner">
				<div class="package-image-container">
					<img
						src={image}
						class="package-image"
						onerror={(e) => e.target.remove()}
					/>
				</div>
				<div class="cont col package-header">
					<div class="package-name">{props.meta.name}</div>
					<Show when={props.meta.categories != undefined}>
						<div style="margin-top:-0.2rem">
							<PackageLabels
								categories={props.meta.categories!}
								small
								limit={3}
							/>
						</div>
					</Show>
					<div class="package-description">{props.meta.description}</div>
				</div>
			</div>
		</div>
	);
}

interface PackageData {
	id: string;
	meta: PackageMeta;
	props: PackageProperties;
}

interface PackageProps {
	id: string;
	meta: PackageMeta;
	selected?: string;
	onSelect: (pkg: string) => void;
}

interface RepoInfo {
	id: string;
	meta: RepoMetadata;
}

interface RepoMetadata {
	name?: string;
	description?: string;
	mcvm_verseion?: string;
	color?: string;
}

export interface BrowsePackagesProps {
	setFooterData: (data: FooterData) => void;
}

// Formats the URL to the browse page based on search parameters
function formatUrl(page: number, repo?: string, search?: string) {
	let query = search == undefined ? "" : `&search=${search}`;
	return `/packages/${page}?repo=${repo}${query}`;
}
