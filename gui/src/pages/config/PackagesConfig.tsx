import { createResource, createSignal, For, Show } from "solid-js";
import InlineSelect from "../../components/input/InlineSelect";
import "./PackagesConfig.css";
import { PackageMeta, PackageProperties, PkgRequest } from "../../types";
import { invoke } from "@tauri-apps/api";
import {
	parsePkgRequest,
	pkgRequestToString,
	stringCompare,
} from "../../utils";
import IconButton from "../../components/input/IconButton";
import { AngleRight, Delete } from "../../icons";
import { errorToast } from "../../components/dialog/Toasts";

export default function PackagesConfig(props: PackagesConfigProps) {
	let [filter, setFilter] = createSignal("user");
	let [sideFilter, setSideFilter] = createSignal("all");

	let [installedPackages, setInstalledPackages] = createSignal<string[]>([]);

	let [packageMetas, setPackageMetas] = createSignal<
		{ [key: string]: PackageMeta } | undefined
	>();
	let [packageProps, setPackageProps] = createSignal<
		{ [key: string]: PackageProperties } | undefined
	>();

	let [allPackages, _] = createResource(async () => {
		let installedPackages: string[] = [];
		if (!props.isProfile) {
			let map: { [key: string]: LockfilePackage } = await invoke(
				"get_instance_packages",
				{ instance: props.id }
			);
			installedPackages = installedPackages.concat(Object.keys(map));
		}

		// Get a list of all packages
		let allPackages = installedPackages.concat([]);

		for (let pkg of props.globalPackages.map(getPackageConfigRequest)) {
			allPackages = allPackages.filter((x) => parsePkgRequest(x).id != pkg.id);
			allPackages.push(pkgRequestToString(pkg));
		}
		for (let pkg of props.clientPackages.map(getPackageConfigRequest)) {
			allPackages = allPackages.filter((x) => parsePkgRequest(x).id != pkg.id);
			allPackages.push(pkgRequestToString(pkg));
		}
		for (let pkg of props.serverPackages.map(getPackageConfigRequest)) {
			allPackages = allPackages.filter((x) => parsePkgRequest(x).id != pkg.id);
			allPackages.push(pkgRequestToString(pkg));
		}

		setInstalledPackages(installedPackages);

		// Get metadata and properties
		let metas: any = {};
		let properties: any = {};

		let promises = [];
		for (let pkg of allPackages) {
			promises.push(
				(async () => {
					try {
						return [
							pkg,
							await invoke("get_package_meta_and_props", { package: pkg }),
						];
					} catch (e) {
						console.error("Failed to load package: " + e);
						return "error";
					}
				})()
			);
		}

		let results = await Promise.all(promises);
		let errorExists = false;
		for (let result of results) {
			if (result == "error") {
				errorExists = true;
				continue;
			}
			let [id, [meta, props]] = result as [
				string,
				[PackageMeta, PackageProperties]
			];
			metas[id] = meta;
			properties[id] = props;
		}

		if (errorExists) {
			errorToast("One or more packages failed to load");
		}

		setPackageMetas(metas);
		setPackageProps(properties);

		allPackages.sort((a, b) =>
			stringCompare(parsePkgRequest(a).id, parsePkgRequest(b).id)
		);

		return allPackages;
	});

	return (
		<div id="packages-config">
			<div id="packages-config-header">
				<div class="cont" style="justify-content:flex-start">
					<InlineSelect
						options={[
							{
								value: "all",
								contents: <div>ALL</div>,
								color: "var(--fg)",
								tip: "All packages",
							},
							{
								value: "user",
								contents: <div>USER</div>,
								color: "var(--instance)",
								tip: "Only packages you have set. No dependencies",
							},
							{
								value: "bundled",
								contents: <div>BUNDLED</div>,
								color: "var(--package)",
								tip: "Packages from modpacks and bundles",
							},
							{
								value: "dependencies",
								contents: <div>DEPS</div>,
								color: "var(--plugin)",
								tip: "Dependencies of other packages",
							},
						]}
						optionClass="package-config-filter"
						selected={filter()}
						onChange={setFilter}
						grid={false}
						connected={false}
						solidSelect={true}
					/>
				</div>
				<div class="cont" style="justify-content:flex-end">
					<Show when={props.isProfile}>
						<InlineSelect
							options={[
								{ value: "all", contents: <div>ALL</div>, color: "var(--fg)" },
								{
									value: "client",
									contents: <div>CLIENT</div>,
									color: "var(--instance)",
								},
								{
									value: "server",
									contents: <div>SERVER</div>,
									color: "var(--profile)",
								},
							]}
							optionClass="package-config-filter"
							selected={sideFilter()}
							onChange={setSideFilter}
							grid={false}
							connected={false}
							solidSelect={true}
						/>
					</Show>
				</div>
			</div>
			<div class="cont col" id="configured-packages">
				<Show
					when={
						allPackages() != undefined &&
						installedPackages() != undefined &&
						packageMetas() != undefined &&
						packageProps() != undefined
					}
				>
					<For each={allPackages()}>
						{(pkg) => {
							let isInstalled = installedPackages()!.includes(pkg);
							let isClient = props.clientPackages.includes(pkg);
							let isServer = props.serverPackages.includes(pkg);
							let isConfigured =
								isClient || isServer || props.globalPackages.includes(pkg);

							let isVisible = () => {
								if (filter() == "user" && !isConfigured) {
									return false;
								} else if (filter() == "bundled") {
									return false;
								} else if (filter() == "dependencies" && isConfigured) {
									return false;
								} else if (sideFilter() == "client" && !isClient) {
									return false;
								} else if (sideFilter() == "server" && !isServer) {
									return false;
								}
								return true;
							};

							let meta = packageMetas()![pkg];
							let properties = packageProps()![pkg];
							if (meta == undefined || properties == undefined) {
								return (
									<Show when={isVisible()}>
										<div
											class="input-shadow configured-package"
											style="border-color:var(--error)"
										>
											<div class="cont">
												<img
													src="/icons/default_instance.png"
													class="configured-package-icon"
												/>
											</div>
											<div class="cont configured-package-name">
												{`${pkg} - Error`}
											</div>
											<div></div>
										</div>
									</Show>
								);
							}

							return (
								<Show when={isVisible()}>
									<ConfiguredPackage
										request={parsePkgRequest(pkg)}
										meta={meta}
										props={properties}
										isInherited={false}
										isInstalled={isInstalled}
										isConfigured={isConfigured}
										category={
											isClient ? "client" : isServer ? "server" : "global"
										}
										onRemove={props.onRemove}
									/>
								</Show>
							);
						}}
					</For>
				</Show>
			</div>
		</div>
	);
}

export interface PackagesConfigProps {
	id?: string;
	globalPackages: PackageConfig[];
	clientPackages: PackageConfig[];
	serverPackages: PackageConfig[];
	isProfile: boolean;
	onRemove: (pkg: string, category: ConfiguredPackageCategory) => void;
	setGlobalPackages: (packages: PackageConfig[]) => void;
	setClientPackages: (packages: PackageConfig[]) => void;
	setServerPackages: (packages: PackageConfig[]) => void;
}

function ConfiguredPackage(props: ConfiguredPackageProps) {
	let [isHovered, setIsHovered] = createSignal(false);
	let name = props.meta.name == undefined ? props.request.id : props.meta.name;
	let icon =
		props.meta.icon == undefined
			? "/icons/default_instance.png"
			: props.meta.icon;

	return (
		<div
			class="input-shadow configured-package"
			onmouseenter={() => setIsHovered(true)}
			onmouseleave={() => setIsHovered(false)}
		>
			<div class="cont">
				<img src={icon} class="configured-package-icon" />
			</div>
			<div class="cont col configured-package-details">
				<div class="cont configured-package-details-top">
					<div class="configured-package-name">{name}</div>
					<Show when={props.request.version != undefined}>
						<div class="configured-package-version">
							{props.request.version}
						</div>
					</Show>
				</div>
				<Show when={props.request.repo != undefined}>
					<div class="configured-package-repo">{props.request.repo}</div>
				</Show>
			</div>
			<div></div>
			<div class="cont configured-package-controls">
				<Show when={isHovered()}>
					<IconButton
						icon={AngleRight}
						size="24px"
						color="var(--bg2)"
						border="var(--bg3)"
						selectedColor="var(--accent)"
						onClick={(e) => {
							e.preventDefault();
							e.stopPropagation();
							window.location.href = `/packages/package/${pkgRequestToString(
								props.request
							)}`;
						}}
						selected={false}
					/>
					<Show when={props.isConfigured}>
						<IconButton
							icon={Delete}
							size="24px"
							color="var(--error)"
							border="var(--error)"
							selectedColor="var(--accent)"
							onClick={(e) => {
								e.preventDefault();
								e.stopPropagation();
								props.onRemove(props.request.id, props.category);
								(e.target! as any).parentElement.parentElement.remove();
							}}
							selected={false}
						/>
					</Show>
				</Show>
			</div>
		</div>
	);
}

interface ConfiguredPackageProps {
	request: PkgRequest;
	meta: PackageMeta;
	props: PackageProperties;
	isInherited: boolean;
	isInstalled: boolean;
	isConfigured: boolean;
	category: ConfiguredPackageCategory;
	onRemove: (pkg: string, category: ConfiguredPackageCategory) => void;
}

export type PackageConfig =
	| string
	| {
			id: string;
	  };

// Gets the PkgRequest from a PackageConfig
export function getPackageConfigRequest(config: PackageConfig) {
	if (typeof config == "string") {
		return parsePkgRequest(config);
	} else {
		return parsePkgRequest(config.id);
	}
}

interface LockfilePackage {
	addons: LockfileAddon[];
}

interface LockfileAddon {}

export type ConfiguredPackageCategory = "global" | "client" | "server";
