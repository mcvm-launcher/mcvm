import { useParams } from "@solidjs/router";
import "./InstanceConfig.css";
import IconTextButton from "../../components/input/IconTextButton";
import { AngleLeft, Check } from "../../icons";
import { invoke } from "@tauri-apps/api";
import { createEffect, createResource, createSignal, Show } from "solid-js";
import { Select } from "@thisbeyond/solid-select";
import "@thisbeyond/solid-select/style.css";

export default function InstanceConfig(props: InstanceConfigProps) {
	let params = useParams();

	// Global profile config if both IDs are null
	let isInstance = props.mode == ConfigMode.Instance;
	let isProfile = props.mode == ConfigMode.Profile;
	let isGlobalProfile = props.mode == ConfigMode.GlobalProfile;

	let id = isInstance
		? params.instanceId
		: isGlobalProfile
		? "Global Profile"
		: params.profileId;

	let [config, configOperations] = createResource(updateConfig);
	let [from, setFrom] = createSignal<string[] | undefined>();
	let [parentConfigs, parentConfigOperations] =
		createResource(updateParentConfig);

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

	// Config signals
	let [newId, setNewId] = createSignal<string | undefined>();
	let [name, setName] = createSignal<string | undefined>();
	let [side, setSide] = createSignal<"client" | "server" | undefined>();
	let [icon, setIcon] = createSignal<string | undefined>();

	let [displayName, setDisplayName] = createSignal("");
	let [message, setMessage] = createSignal("");

	createEffect(() => {
		if (config() != undefined) {
			setName(config()!.name);
			setSide(config()!.side);
			setIcon(config()!.icon);

			setDisplayName(config()!.name == undefined ? id : config()!.name!);
			setMessage(
				isInstance
					? `Instance ${displayName()}`
					: isGlobalProfile
					? "Global Profile"
					: `Profile ${displayName()}`
			);
		}
	});

	// Writes configuration to disk
	async function saveConfig() {
		console.log(from());
		console.log(side());
		console.log(name());
		console.log(icon());

		let newConfig: InstanceConfig = {
			from: from(),
			side: side(),
			name: name(),
			icon: icon(),
		};

		// Handle extra fields
		if (config() != undefined) {
			for (let key of Object.keys(config()!)) {
				if (!Object.keys(newConfig).includes(key)) {
					newConfig[key] = config()![key];
				}
			}
		}

		let configId = props.creating ? newId() : id;

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
			<div class="back-button">
				<IconTextButton
					icon={AngleLeft}
					size="22px"
					text="Back"
					color="var(--bg2)"
					selectedColor="var(--bg2)"
					onClick={() => {
						history.back();
					}}
					selected={false}
				/>
			</div>
			<br />
			<div id="fields" class="cont col">
				<div class="cont">
					<Show when={props.creating && !isGlobalProfile}>
						<label for="id">{`${createMessage} ID`}</label>
						<input
							type="text"
							id="id"
							name="id"
							onChange={(e) => setNewId(e.target.value)}
						></input>
					</Show>
				</div>
				<div class="cont">
					<label for="name">Display Name</label>
					<input
						type="text"
						id="name"
						name="name"
						placeholder={id}
						value={emptyUndefined(name())}
						onChange={(e) => setName(e.target.value)}
					></input>
				</div>
				<Show when={props.creating}>
					<div class="cont">
						<label for="side">Side</label>
						<Select
							class="select"
							options={[
								{ name: "client", label: "Client" },
								{ name: "server", label: "Server" },
							]}
							format={optionFormat}
							initialValue={side()}
							onChange={(value) => {
								// This is dumb
								setSide(value.name);
							}}
						/>
					</div>
				</Show>
			</div>
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
	side?: "client" | "server";
	name?: string;
	icon?: string;
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

function optionFormat(item: Option, type: string) {
	if (type == "option") {
		return item.label;
	} else {
		return item.name;
	}
}

interface Option {
	name: string;
	label: string;
}
