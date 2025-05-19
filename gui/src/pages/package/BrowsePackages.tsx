import { useLocation, useParams } from "@solidjs/router";
import "./BrowsePackages.css";
import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, For } from "solid-js";
import "@thisbeyond/solid-select/style.css";
import PageButtons from "../../components/input/PageButtons";
import { PackageMeta } from "../../types";
import SearchBar from "../../components/input/SearchBar";
import { parseQueryString } from "../../utils";

const PACKAGES_PER_PAGE = 12;

export default function BrowsePackages() {
	let params = useParams();
	let searchParams = parseQueryString(useLocation().search);

	let page = +params.page;
	let search = searchParams["search"];

	let [packages, _] = createResource(updatePackages);

	let [packageCount, setPackageCount] = createSignal(0);

	async function updatePackages() {
		let start = PACKAGES_PER_PAGE * page;
		let end = start + PACKAGES_PER_PAGE;
		console.log(start, end);
		let [packagesToRequest, packageCount] = (await invoke("get_packages", {
			start: start,
			end: end,
			search: search,
		})) as [string[], number];
		setPackageCount(packageCount);
		console.log(packageCount);
		console.log("Packages fetched");

		let promises = [];
		for (let pkg of packagesToRequest) {
			promises.push(invoke("get_package_meta", { package: pkg }));
		}

		let finalPackages = (await Promise.all(promises)) as PackageMeta[];
		console.log(finalPackages);
		let packagesAndIds = finalPackages.map((val, i) => {
			return {
				id: packagesToRequest[i],
				meta: val,
			} as PackageProps;
		});
		return packagesAndIds;
	}

	return (
		<div class="cont col" style="width:100%">
			<div id="header">
				<div></div>
				<h1 class="noselect">Packages</h1>
				<div class="cont">
					<SearchBar
						placeholder="Search for packages..."
						value={search}
						method={(term) => {
							window.location.href = `/packages/0?search=${term}`;
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
					let query = search == undefined ? "" : `?search=${search}`;
					window.location.href = `/packages/${page}${query}`;
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
			class="cont col package"
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
