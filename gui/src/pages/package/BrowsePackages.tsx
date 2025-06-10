import { useLocation, useParams } from "@solidjs/router";
import "./BrowsePackages.css";
import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, For, Show } from "solid-js";
import "@thisbeyond/solid-select/style.css";
import PageButtons from "../../components/input/PageButtons";
import { PackageMeta } from "../../types";
import SearchBar from "../../components/input/SearchBar";
import { parseQueryString } from "../../utils";
import InlineSelect from "../../components/input/InlineSelect";
import { emit } from "@tauri-apps/api/event";

const PACKAGES_PER_PAGE = 12;

export default function BrowsePackages() {
	let params = useParams();
	let searchParams = parseQueryString(useLocation().search);

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
		let start = PACKAGES_PER_PAGE * page;
		let end = start + PACKAGES_PER_PAGE;

		let repos: RepoInfo[] = await invoke("get_package_repos");
		let index = repos.findIndex((x) => x.id == "core");
		if (index != -1) {
			repos.splice(index, 1);
		}
		setRepos(repos);

		let [packagesToRequest, packageCount] = (await invoke("get_packages", {
			repo: selectedRepo(),
			start: start,
			end: end,
			search: search,
		})) as [string[], number];
		setPackageCount(packageCount);
		console.log(packageCount);
		console.log("Packages fetched");

		let promises = [];
		console.log("Waiting for packages");
		emit("mcvm_output_create_task", "get_packages");
		for (let pkg of packagesToRequest) {
			promises.push(invoke("get_package_meta", { package: pkg }));
		}

		try {
			let finalPackages = (await Promise.all(promises)) as PackageMeta[];
			console.log(finalPackages);
			let packagesAndIds = finalPackages.map((val, i) => {
				return {
					id: packagesToRequest[i],
					meta: val,
				} as PackageProps;
			});
			return packagesAndIds;
		} catch (e) {
			console.error(e);
		} finally {
			emit("mcvm_output_finish_task", "get_packages");
		}
	}

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
				<For each={packages()}>{(props) => <Package {...props} />}</For>
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
	return (
		<div
			class="cont col input-shadow package"
			style="cursor:pointer"
			onclick={() => (window.location.href = `/packages/package/${props.id}`)}
		>
			<div class="package-inner">
				<div class="package-image-container">
					<img src={image} class="package-image" />
				</div>
				<div class="cont col package-header">
					<div class="package-name">{props.meta.name}</div>
					<div class="package-description">{props.meta.description}</div>
				</div>
			</div>
		</div>
	);
}

interface PackageProps {
	id: string;
	meta: PackageMeta;
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
