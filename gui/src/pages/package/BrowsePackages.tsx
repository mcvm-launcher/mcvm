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
import { PackageMeta } from "../../types";
import SearchBar from "../../components/input/SearchBar";
import { parseQueryString } from "../../utils";
import InlineSelect from "../../components/input/InlineSelect";
import { emit } from "@tauri-apps/api/event";
import { FooterData } from "../../App";
import { FooterMode } from "../../components/launch/Footer";

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
		let repos: RepoInfo[] = await invoke("get_package_repos");
		let index = repos.findIndex((x) => x.id == "core");
		if (index != -1) {
			repos.splice(index, 1);
		}
		setRepos(repos);

		let [packagesToRequest, packageCount] = (await invoke("get_packages", {
			repo: selectedRepo(),
			page: page,
			search: search,
		})) as [string[], number];
		setPackageCount(packageCount);
		console.log(packageCount);
		console.log("Packages fetched");

		let promises = [];
		console.log("Waiting for packages");
		emit("mcvm_output_create_task", "get_packages");
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
			console.log(finalPackages);
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
			console.error(e);
		} finally {
			emit("mcvm_output_finish_task", "get_packages");
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
										let query = search == undefined ? "" : `&search=${search}`;
										window.location.href = `/packages/0?repo=${x}${query}`;
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
							window.location.href = `/packages/0?search=${term}&repo=${selectedRepo()}`;
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
											selectedItem: pkg,
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
					let query = search == undefined ? "" : `&search=${search}`;
					window.location.href = `/packages/${page}?repo=${selectedRepo()}${query}`;
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
					<div class="package-description">{props.meta.description}</div>
				</div>
			</div>
		</div>
	);
}

interface PackageData {
	id: string;
	meta: PackageMeta;
	props: PackageProps;
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
