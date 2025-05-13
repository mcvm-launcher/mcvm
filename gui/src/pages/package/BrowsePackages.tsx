import { useParams } from "@solidjs/router";
import "./BrowsePackages.css";
import IconTextButton from "../../components/input/IconTextButton";
import { AngleLeft, Check } from "../../icons";
import { invoke } from "@tauri-apps/api";
import {
	createEffect,
	createResource,
	createSignal,
	For,
	Show,
} from "solid-js";
import "@thisbeyond/solid-select/style.css";

const PACKAGES_PER_PAGE = 12;

export default function BrowsePackages() {
	let params = useParams();

	let page = +params.page;

	let [packages, _] = createResource(updatePackages);

	async function updatePackages() {
		// let allPackages: string[] = [];

		let start = PACKAGES_PER_PAGE * page;
		let end = start + PACKAGES_PER_PAGE;
		console.log("here");
		let allPackages: string[] = await invoke("get_packages", {
			start: start,
			end: end,
		});
		console.log("Packages fetched");

		let packagesToRequest = allPackages.slice(start, end);

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
			<h1 class="noselect">Packages</h1>
			<div id="packages-container">
				<For each={packages()}>{(props) => <Package {...props} />}</For>
			</div>
			<br />
			<br />
			<br />
		</div>
	);
}

interface PackageMeta {
	name?: string;
	banner?: string;
	icon?: string;
}

function Package(props: PackageProps) {
	return (
		<div class="cont col package border border-big">
			<div class="package-inner">
				<div class="package-image-container">
					<img
						src={
							props.meta.banner == undefined
								? props.meta.icon
								: props.meta.banner
						}
						class="package-image"
					/>
				</div>
				<div class="package-header">
					<div class="package-name">{props.meta.name}</div>
				</div>
			</div>
		</div>
	);
}

interface PackageProps {
	id: string;
	meta: PackageMeta;
}
