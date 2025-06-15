import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, For, Show } from "solid-js";
import "./Plugins.css";
import IconTextButton from "../../components/input/IconTextButton";
import { Refresh } from "../../icons";
import { emit } from "@tauri-apps/api/event";
import { errorToast, successToast } from "../../components/dialog/Toasts";

export default function Plugins() {
	let [plugins, methods] = createResource(updatePlugins);
	let [isRemote, setIsRemote] = createSignal(false);
	let [restartNeeded, setRestartNeeded] = createSignal(false);

	async function updatePlugins() {
		let plugins: PluginInfo[] = await invoke("get_plugins");
		return plugins;
	}

	return (
		<div id="plugins">
			<div id="plugins-header">
				<div class="cont">
					<IconTextButton
						icon={Refresh}
						text="Refresh Launcher"
						size="22px"
						color="var(--bg2)"
						selectedColor="var(--plugin)"
						onClick={() => {
							emit("refresh_window");
						}}
						selected={restartNeeded()}
					/>
				</div>
				<h1 class="noselect">Plugins</h1>
				<div></div>
			</div>
			<div class="cont">
				<div id="plugins-subheader">
					<div
						class={`plugins-header-item ${isRemote() ? "" : " selected"}`}
						onclick={() => {
							setIsRemote(false);
						}}
					>
						Installed
					</div>
					<div
						class={`plugins-header-item ${isRemote() ? " selected" : ""}`}
						onclick={() => {
							setIsRemote(true);
						}}
					>
						Available
					</div>
				</div>
			</div>
			<br />
			<div class="cont col" id="plugin-list">
				<For each={plugins()}>
					{(info) => {
						let pluginIsRemote = !info.installed;
						// Hide the remote version of a plugin if it is installed locally
						let idCount = plugins()!.filter((x) => x.id == info.id).length;
						let isRemoteHidden = pluginIsRemote && idCount > 1;
						let isCorrectPage = () => isRemote() == pluginIsRemote;
						return (
							<Show when={isCorrectPage() && !isRemoteHidden}>
								<Plugin
									info={info}
									updatePluginList={() => {
										methods.refetch();
										setRestartNeeded(true);
									}}
								/>
							</Show>
						);
					}}
				</For>
			</div>
			<br />
			<br />
			<br />
			<br />
		</div>
	);
}

function Plugin(props: PluginProps) {
	let isDisabled = () => !props.info.enabled && props.info.installed;

	let [inProgress, setInProgress] = createSignal(false);

	return (
		<div
			class={`cont col input-shadow plugin ${isDisabled() ? "disabled" : ""}`}
		>
			<div class="plugin-top">
				<div class="cont plugin-header">
					<div class="plugin-name">{props.info.name}</div>
					<div class="plugin-id">{props.info.id}</div>
				</div>
				<div class="cont plugin-buttons">
					<Show when={props.info.installed}>
						<IconTextButton
							text={props.info.enabled ? "Disable" : "Enable"}
							size="22px"
							color="var(--bg2)"
							selectedColor="var(--instance)"
							onClick={() => {
								invoke("enable_disable_plugin", {
									plugin: props.info.id,
									enabled: !props.info.enabled,
								}).then(() => {
									successToast(
										`Plugin ${props.info.enabled ? "disabled" : "enabled"}`
									);
									props.updatePluginList();
								});
							}}
							selected={false}
							shadow={false}
						/>
					</Show>
					<IconTextButton
						text={
							props.info.installed
								? "Uninstall"
								: inProgress()
								? "Installing..."
								: "Install"
						}
						size="22px"
						color="var(--bg2)"
						selectedColor="var(--instance)"
						onClick={() => {
							setInProgress(true);
							let method = props.info.installed
								? "uninstall_plugin"
								: "install_plugin";
							invoke(method, {
								plugin: props.info.id,
							}).then(
								() => {
									setInProgress(false);
									successToast(
										`Plugin ${
											props.info.installed ? "uninstalled" : "installed"
										}`
									);
									props.updatePluginList();
								},
								(e) => {
									setInProgress(false);
									errorToast(
										`Failed to ${
											props.info.installed ? "uninstall" : "install"
										} plugin: ${e}`
									);
								}
							);
						}}
						selected={false}
						shadow={false}
					/>
				</div>
			</div>
			<div class="cont" style="justify-content:flex-start;width:100%">
				<div class="plugin-description">{props.info.description}</div>
			</div>
		</div>
	);
}

interface PluginProps {
	info: PluginInfo;
	updatePluginList: () => void;
}

interface PluginInfo {
	id: string;
	name?: string;
	description?: string;
	enabled: boolean;
	installed: boolean;
}
