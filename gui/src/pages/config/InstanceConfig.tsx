import { useParams } from "@solidjs/router";
import "./InstanceConfig.css";
import IconTextButton from "../../components/input/IconTextButton";
import { Check } from "../../icons";
import { invoke } from "@tauri-apps/api";
import {
	createEffect,
	createResource,
	createSignal,
	onMount,
	Show,
} from "solid-js";
import "@thisbeyond/solid-select/style.css";
import InlineSelect from "../../components/input/InlineSelect";
import { loadPagePlugins } from "../../plugins";
import { inputError } from "../../errors";
import { beautifyString, stringCompare } from "../../utils";

export default function InstanceConfig(props: InstanceConfigProps) {
	let params = useParams();

	let isInstance = props.mode == ConfigMode.Instance;
	let isProfile = props.mode == ConfigMode.Profile;
	let isGlobalProfile = props.mode == ConfigMode.GlobalProfile;

	let id = isInstance
		? params.instanceId
		: isGlobalProfile
			? "Global Profile"
			: params.profileId;

	onMount(() =>
		loadPagePlugins(
			isInstance
				? "instance_config"
				: isProfile
					? "profile_config"
					: "global_profile_config",
			id
		)
	);

	let [config, configOperations] = createResource(updateConfig);
	let [from, setFrom] = createSignal<string[] | undefined>();
	let [parentConfigs, parentConfigOperations] =
		createResource(updateParentConfig);
	let [supportedModifications, _] = createResource(getSupportedGameModifications);

	let [tab, setTab] = createSignal("basic");

	async function updateConfig() {
		if (props.creating) {
			return undefined;
		}
		// Get the instance or profile
		let method = isInstance
			? "get_instance_config"
			: isGlobalProfile
				? "get_global_profile"
				: "get_profile_config";
		let result = await invoke(method, { id: id });
		let configuration = result as InstanceConfig;

		// Canonicalize this to an array
		setFrom(
			configuration.from == undefined
				? undefined
				: Array.isArray(configuration.from)
					? configuration.from
					: [configuration.from]
		);

		return configuration;
	}

	async function updateParentConfig() {
		let fromValues = from();
		// Get the parent
		let parentResults: InstanceConfig[] = [];
		if (isGlobalProfile) {
			parentResults = [];
		} else if (fromValues == null) {
			let parentResult = await invoke("get_global_profile", {});
			parentResults = [parentResult as InstanceConfig];
		} else {
			for (let profile of fromValues!) {
				let parentResult = await invoke("get_profile_config", {
					id: profile,
				});
				parentResults.push(parentResult as InstanceConfig);
			}
		}

		return parentResults;
	}

	// Input / convenience signals

	// Used to check if we can automatically fill out the ID with the name. We don't want to do this if the user already typed an ID.
	let [isIdDirty, setIsIdDirty] = createSignal(!props.creating);

	// Config signals
	let [newId, setNewId] = createSignal<string | undefined>();
	let [name, setName] = createSignal<string | undefined>();
	let [side, setSide] = createSignal<"client" | "server" | undefined>();
	let [icon, setIcon] = createSignal<string | undefined>();
	let [version, setVersion] = createSignal<string | undefined>();
	let [clientType, setClientType] = createSignal<string | undefined>();
	let [serverType, setServerType] = createSignal<string | undefined>();
	let [gameModVersion, setGameModVersion] = createSignal<string | undefined>();
	let [datapackFolder, setDatapackFolder] = createSignal<string | undefined>();

	let [displayName, setDisplayName] = createSignal("");
	let [message, setMessage] = createSignal("");

	createEffect(() => {
		if (config() != undefined) {
			setName(config()!.name);
			setSide(config()!.type);
			setIcon(config()!.icon);
			setVersion(config()!.version);
			setClientType(config()!.client_type);
			setServerType(config()!.server_type);
			if (config()!.client_type == "none" || config()!.client_type == undefined) {
				setClientType(config()!.modloader);
			}
			if (config()!.server_type == "none" || config()!.server_type == undefined) {
				setServerType(config()!.modloader);
			}
			setGameModVersion(config()!.game_modification_version);
			setDatapackFolder(config()!.datapack_folder);

			setDisplayName(config()!.name == undefined ? id : config()!.name!);
			setMessage(
				isInstance
					? `Instance ${displayName()}`
					: isGlobalProfile
						? "Global Profile"
						: `Profile ${displayName()}`
			);
		}

		if (props.creating && props.mode == "instance") {
			setSide("client");
		}
	});

	// Writes configuration to disk
	async function saveConfig() {
		console.log(from());
		console.log(side());
		console.log(name());
		console.log(icon());
		console.log(version());

		let configId = props.creating ? newId() : id;

		if (!isGlobalProfile && configId == undefined) {
			inputError("id");
			return;
		}
		if (props.creating) {
			if (await idExists(configId!, props.mode)) {
				inputError("id");
				return;
			}
		}

		if (isInstance && side() == undefined) {
			inputError("side");
			return;
		}

		if (isInstance && version() == undefined) {
			inputError("version");
			return;
		}

		let newConfig: InstanceConfig = {
			from: from(),
			type: side(),
			name: undefinedEmpty(name()),
			icon: undefinedEmpty(icon()),
			version: undefinedEmpty(version()),
			modloader: config() != undefined ? config()!.modloader : undefined,
			client_type: clientType(),
			server_type: serverType(),
			game_modification_version: undefinedEmpty(gameModVersion()),
		};

		// Handle extra fields
		if (config() != undefined) {
			for (let key of Object.keys(config()!)) {
				if (!Object.keys(newConfig).includes(key)) {
					newConfig[key] = config()![key];
				}
			}
		}

		if (isInstance) {
			await invoke("write_instance_config", {
				id: configId,
				config: newConfig,
			});
		} else if (isGlobalProfile) {
			await invoke("write_global_profile", { config: newConfig });
		} else {
			await invoke("write_profile_config", { id: configId, config: newConfig });
		}

		configOperations.refetch();
	}

	let createMessage = isInstance ? "Instance" : "Profile";

	return (
		<div class="cont col" style="width:100%">
			<h1 class="noselect">
				{props.creating
					? `Creating New ${createMessage}`
					: `Configuration for ${message()}`}
			</h1>
			<div class="cont">
				<div id="config-tabs">
					<div
						class={`config-tab ${tab() == "basic" ? "selected" : ""}`}
						id="basic-tab"
						onclick={() => {
							setTab("basic");
						}}
					>
						Basic
					</div>
					<div
						class={`config-tab ${tab() == "packages" ? "selected" : ""}`}
						id="packages-tab"
						onclick={() => {
							setTab("packages");
						}}
					>
						Packages
					</div>
					<div
						class={`config-tab ${tab() == "launch" ? "selected" : ""}`}
						id="launch-tab"
						onclick={() => {
							setTab("launch");
						}}
					>
						Launch Settings
					</div>
				</div>
			</div>
			<br />
			<Show when={tab() == "basic"}>
				<div class="fields">
					<div></div>
					<h2>Basic Settings</h2>
					<Show when={!isGlobalProfile && !isProfile}>
						<label for="name" class="label">Display Name</label>
						<input
							type="text"
							id="name"
							name="name"
							placeholder={id}
							value={emptyUndefined(name())}
							onChange={(e) => setName(e.target.value)}
							onKeyUp={(e: any) => {
								if (!isIdDirty()) {
									let value = sanitizeInstanceId(e.target.value);
									(document.getElementById("id")! as any).value = value;
									setNewId(value);
								}
							}}
						></input>
					</Show>
					<Show when={props.creating && !isGlobalProfile}>
						<label for="id" class="label">{`${createMessage} ID`}</label>
						<input
							type="text"
							id="id"
							name="id"
							onChange={(e) => {
								setNewId()
								e.target.value = sanitizeInstanceId(e.target.value);
								setNewId(e.target.value);
							}}
							onKeyUp={(e: any) => {
								setIsIdDirty(true);
								e.target.value = sanitizeInstanceId(e.target.value);
							}}
						></input>
					</Show>
					<Show when={props.creating || isProfile || isGlobalProfile}>
						<label for="side" class="label">Side</label>
						<div class="cont col">
							<div id="side">
								<InlineSelect
									onChange={setSide}
									selected={side()}
									options={[
										{
											value: "client",
											contents: <div class="cont">Client</div>,
											color: "var(--instance)",
										},
										{
											value: "server",
											contents: <div class="cont">Server</div>,
											color: "var(--profile)",
										},
									]}
									columns={isInstance ? 2 : 3}
									allowEmpty={!isInstance}
								/>
							</div>
						</div>
					</Show>
					<label for="version" class="label">Minecraft Version</label>
					<input
						type="text"
						id="version"
						name="version"
						value={emptyUndefined(version())}
						onChange={(e) => setVersion(e.target.value)}
					></input>
					<Show when={(side() == "client" || isProfile) && supportedModifications() != undefined}>
						<label for="client-type" class="label">{`${isProfile ? "Client " : ""}Loader`}</label>
						<div class="cont col">
							<div id="client-type">
								<InlineSelect
									onChange={setClientType}
									selected={clientType() == undefined ? "none" : clientType()}
									options={supportedModifications()!.client_types.map((x) => {
										return {
											value: x,
											contents: <div class="cont">{x == "none" ? "Unset" : beautifyString(x)}</div>,
											color: "var(--fg2)"
										};
									})}
									columns={supportedModifications()!.client_types.length}
									allowEmpty={false}
								/>
							</div>
						</div>
					</Show>
					<Show when={(side() == "server" || isProfile) && supportedModifications() != undefined}>
						<label for="server-type" class="label">{`${isProfile ? "Server " : ""}Loader`}</label>
						<div class="cont col">
							<div id="server-type">
								<InlineSelect
									onChange={setServerType}
									selected={serverType() == undefined ? "none" : serverType()}
									options={supportedModifications()!.server_types.map((x) => {
										return {
											value: x,
											contents: <div class="cont">{x == "none" ? "Unset" : beautifyString(x)}</div>,
											color: "var(--fg2)"
										};
									})}
									columns={supportedModifications()!.server_types.length}
									allowEmpty={false}
								/>
							</div>
						</div>
					</Show>
					<Show when={(clientType() != undefined && clientType() != "none") || (serverType() != undefined && serverType() != "none")}>
						<label for="game-mod-version" class="label">Loader Version</label>
						<input
							type="text"
							id="game-mod-version"
							name="game-mod-version"
							value={emptyUndefined(gameModVersion())}
							onChange={(e) => setGameModVersion(e.target.value)}
						></input>
					</Show>
					<div></div>
					<div></div>
					<div></div>
					<h2>Extra Settings</h2>
					<label for="datapack-folder" class="label">Datapack Folder</label>
					<input
						type="text"
						id="datapack-folder"
						name="datapack-folder"
						value={emptyUndefined(datapackFolder())}
						onChange={(e) => setDatapackFolder(e.target.value)}
					></input>
				</div>
			</Show>
			<br />
			<div class="cont">
				<IconTextButton
					icon={Check}
					size="22px"
					text="Save"
					color="var(--bg2)"
					selectedColor="var(--bg2)"
					onClick={() => {
						saveConfig();
						if (props.creating) {
							// window.location.href = "/";
						}
					}}
					selected={false}
				/>
			</div>
		</div>
	);
}

export interface InstanceConfigProps {
	mode: ConfigMode;
	/* Whether we are creating a new instance or profile */
	creating: boolean;
}

interface InstanceConfig {
	from?: string[];
	type?: "client" | "server";
	name?: string;
	icon?: string;
	version?: string | "latest" | "latest_snapshot";
	modloader?: string;
	client_type?: string;
	server_type?: string;
	game_modification_version?: string;
	datapack_folder?: string;
	[extraKey: string]: any;
}

export enum ConfigMode {
	Instance = "instance",
	Profile = "profile",
	GlobalProfile = "global_profile",
}

function emptyUndefined(value: string | undefined) {
	if (value == undefined) {
		return "";
	} else {
		return value;
	}
}

function undefinedEmpty(value: string | undefined) {
	if (value == "") {
		return undefined;
	} else {
		return value;
	}
}

function sanitizeInstanceId(id: string): string {
	id = id.toLocaleLowerCase();
	id = id.replace(/ /g, "-");
	id = id.replace(/\_/g, "-");
	id = id.replace(/\./g, "-");
	// Remove repeated hyphens
	let regex = new RegExp(/-+/, "g");
	id = id.replace(regex, "-");
	// TODO: Sanitize wild characters
	// let regex = new RegExp(/\W/, "ig");
	// id = id.replace(regex, "");
	return id;
}

async function idExists(id: string, mode: ConfigMode): Promise<boolean> {
	let command = `get_${mode}_config`;
	try {
		let result = await invoke(command, { id: id });
		return result != null;
	} catch (e) {
		console.error(e);
		return false;
	}
}

async function getSupportedGameModifications(): Promise<SupportedGameModifications> {
	let out: SupportedGameModifications = { client_types: [], server_types: [] };
	let results: SupportedGameModifications[] = await invoke("get_supported_game_modifications");
	for (let result of results) {
		out.client_types = out.client_types.concat(result.client_types);
		out.server_types = out.server_types.concat(result.server_types);
	}

	out.client_types.sort(stringCompare);
	out.server_types.sort(stringCompare);
	out.client_types = ["none", "vanilla"].concat(out.client_types);
	out.server_types = ["none", "vanilla"].concat(out.server_types);

	return out;
}

interface SupportedGameModifications {
	client_types: string[];
	server_types: string[];
}
