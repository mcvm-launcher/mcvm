import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import "./Footer.css";
import { UnlistenFn, listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";
import { PasswordPrompt } from "../input/PasswordPrompt";
import { Box, Check, Play, Properties, Upload } from "../../icons";
import IconButton from "../input/IconButton";
import { AuthDisplayEvent, RunningInstanceInfo } from "../../types";
import MicrosoftAuthInfo from "../input/MicrosoftAuthInfo";
import { getIconSrc } from "../../utils";
import TaskIndicator from "../TaskIndicator";

export default function LaunchFooter(props: LaunchFooterProps) {
	// Basic state
	const [runningInstances, setRunningInstances] = createSignal<
		RunningInstanceInfo[]
	>([]);

	// Prompts
	const [showPasswordPrompt, setShowPasswordPrompt] = createSignal(false);
	const [authInfo, setAuthInfo] = createSignal<AuthDisplayEvent | null>(null);
	const [passwordPromptMessage, setPasswordPromptMessage] = createSignal("");
	// Unlisteners for tauri events
	const [unlistens, setUnlistens] = createSignal<UnlistenFn[]>([]);

	async function updateRunningInstances() {
		setRunningInstances(await invoke("get_running_instances"));
	}

	// Setup and clean up event listeners for updating state
	onMount(async () => {
		updateRunningInstances();

		for (let unlisten of unlistens()) {
			unlisten();
		}

		let updateStatePromise = listen("update_run_state", () => {
			console.log("Updating run state");
			updateRunningInstances();
		});

		let authInfoPromise = listen(
			"mcvm_display_auth_info",
			(event: Event<AuthDisplayEvent>) => {
				setAuthInfo(event.payload);
			}
		);

		let authInfoClosePromise = listen("mcvm_close_auth_info", () => {
			setAuthInfo(null);
		});

		let passwordPromise = listen(
			"mcvm_display_password_prompt",
			(event: Event<string>) => {
				setShowPasswordPrompt(true);
				setPasswordPromptMessage(event.payload);
			}
		);

		let stoppedPromise = listen("game_finished", (event: Event<string>) => {
			console.log("Stopped instance " + event.payload);
			stopGame(event.payload);
		});

		let eventUnlistens = await Promise.all([
			updateStatePromise,
			authInfoPromise,
			authInfoClosePromise,
			passwordPromise,
			stoppedPromise,
		]);

		setUnlistens(eventUnlistens);
	});

	onCleanup(() => {
		for (const unlisten of unlistens()) {
			unlisten();
		}
	});

	async function launch() {
		if (props.selectedItem == undefined || props.mode != FooterMode.Instance) {
			return;
		}

		// Prevent launching until the current authentication screens are finished
		if (showPasswordPrompt() || authInfo() !== null) {
			return;
		}

		let launchPromise = invoke("launch_game", {
			instanceId: props.selectedItem,
			offline: false,
			user: props.selectedUser,
		});

		await Promise.all([launchPromise]);

		updateRunningInstances();
	}

	async function stopGame(instance: string) {
		setAuthInfo(null);
		setShowPasswordPrompt(false);
		await invoke("stop_game", { instance: instance });
		updateRunningInstances();
	}

	return (
		<div class="footer">
			<div id="footer-left" class="footer-section">
				<Show
					when={props.selectedItem != undefined && props.selectedItem != ""}
				>
					<div class="cont" id="footer-selection-indicator">
						{`Selected: ${props.selectedItem}`}
					</div>
				</Show>
			</div>
			<div id="footer-center" class="cont footer-section">
				<div id="footer-center-inner">
					<Show
						when={
							props.mode == FooterMode.Instance &&
							props.selectedItem != undefined
						}
						fallback={<div></div>}
					>
						<div class="cont">
							<div class="cont footer-update">
								<IconButton
									icon={Upload}
									size="28px"
									color="var(--bg2)"
									selectedColor="var(--accent)"
									onClick={() => {
										if (props.selectedItem != undefined) {
											invoke("update_instance", {
												instanceId: props.selectedItem,
											});
										}
									}}
									selected={false}
								/>
							</div>
							<div class="cont footer-config">
								<IconButton
									icon={Properties}
									size="28px"
									color="var(--bg2)"
									selectedColor="var(--accent)"
									onClick={() => {
										if (props.selectedItem != undefined) {
											window.location.href = `/${props.mode}_config/${props.selectedItem}`;
										}
									}}
									selected={false}
								/>
							</div>
						</div>
					</Show>
					<ActionButton
						selected={props.selectedItem != undefined}
						mode={props.mode}
						onClick={() => {
							if (props.mode == FooterMode.Instance) {
								launch();
							} else if (props.mode == FooterMode.Profile) {
								if (props.selectedItem != undefined) {
									window.location.href = `/profile_config/${props.selectedItem}`;
								}
							} else {
								props.action();
							}
						}}
					/>
				</div>
			</div>
			<div id="footer-right" class="footer-section">
				<div class="cont">
					<TaskIndicator />
				</div>
				<div class="cont">
					<RunningInstanceList
						instances={runningInstances()}
						onStop={stopGame}
					/>
				</div>
			</div>

			<Show when={authInfo() !== null}>
				<MicrosoftAuthInfo
					event={authInfo() as AuthDisplayEvent}
					onCancel={() => {
						setAuthInfo(null);
						if (props.selectedItem != undefined) {
							stopGame(props.selectedItem);
						}
					}}
				/>
			</Show>
			<Show when={showPasswordPrompt()}>
				<PasswordPrompt
					onSubmit={() => setShowPasswordPrompt(false)}
					message={passwordPromptMessage()}
				/>
			</Show>
		</div>
	);
}

export interface LaunchFooterProps {
	selectedItem?: string;
	mode: FooterMode;
	selectedUser?: string;
	action: () => void;
}

function ActionButton(props: ActionButtonProps) {
	let backgroundColor = () => {
		if (props.selected) {
			if (
				props.mode == FooterMode.Instance ||
				props.mode == FooterMode.SaveInstanceConfig
			) {
				return "var(--instancebg)";
			} else if (
				props.mode == FooterMode.Profile ||
				props.mode == FooterMode.SaveProfileConfig
			) {
				return "var(--profilebg)";
			} else if (props.mode == FooterMode.PreviewPackage) {
				return "var(--packagebg)";
			}
		}
		return "black";
	};
	let borderColor = () => {
		if (props.selected) {
			if (
				props.mode == FooterMode.Instance ||
				props.mode == FooterMode.SaveInstanceConfig
			) {
				return "var(--instance)";
			} else if (
				props.mode == FooterMode.Profile ||
				props.mode == FooterMode.SaveProfileConfig
			) {
				return "var(--profile)";
			} else if (props.mode == FooterMode.PreviewPackage) {
				return "var(--package)";
			}
		}
		return "var(--bg3)";
	};
	let message = () => {
		if (props.mode == FooterMode.Instance) {
			return "Launch";
		} else if (props.mode == FooterMode.Profile) {
			return "Edit";
		} else if (
			props.mode == FooterMode.SaveInstanceConfig ||
			props.mode == FooterMode.SaveProfileConfig
		) {
			return "Save";
		} else if (props.mode == FooterMode.PreviewPackage) {
			return "Open";
		}
	};
	let Icon = () => {
		if (props.mode == FooterMode.Instance) {
			return <Play />;
		} else if (props.mode == FooterMode.Profile) {
			return <Properties />;
		} else if (
			props.mode == FooterMode.SaveInstanceConfig ||
			props.mode == FooterMode.SaveProfileConfig
		) {
			return <Check />;
		} else if (props.mode == FooterMode.PreviewPackage) {
			return <Box />;
		}
	};

	let backgroundStyle = () => `background-color:${backgroundColor()}`;
	let borderStyle = () => `border-color:${borderColor()}`;

	return (
		<div id="footer-action-button-container">
			<div class="footer-action-button-decorations">
				<div
					class="footer-action-button-decoration left"
					style={`${backgroundStyle()};${borderStyle()}`}
				></div>
				<div
					class="footer-action-button-decoration right"
					style={borderStyle()}
				></div>
				<div
					class="footer-action-button-decoration left"
					style={`${backgroundStyle()};${borderStyle()}`}
				></div>
				<div
					class="footer-action-button-decoration right"
					style={borderStyle()}
				></div>
			</div>
			<div
				id="footer-action-button"
				class="cont"
				onclick={props.onClick}
				style={`background-color:${backgroundColor()};border-color:${borderColor()};color:${borderColor()}`}
			>
				{Icon()}
				{message()}
			</div>
			<div class="footer-action-button-decorations">
				<div
					class="footer-action-button-decoration left"
					style={`border-top-width:0px;border-top-left-radius:0px;${borderStyle()}`}
				></div>
				<div
					class="footer-action-button-decoration right"
					style={`${backgroundStyle()};${borderStyle()}`}
				></div>
				<div
					class="footer-action-button-decoration left"
					style={borderStyle()}
				></div>
				<div
					class="footer-action-button-decoration right"
					style={`${backgroundStyle()};${borderStyle()}`}
				></div>
			</div>
		</div>
	);
}

interface ActionButtonProps {
	selected: boolean;
	mode: FooterMode;
	onClick: () => void;
}

// Displays a list of instance icons that can be interacted with
function RunningInstanceList(props: RunningInstanceListProps) {
	return (
		<div class="running-instance-list">
			<For each={props.instances}>
				{(instance) => (
					<img
						src={getIconSrc(instance.info.icon)}
						class="running-instance-list-icon border"
						title={
							instance.info.name != null ? instance.info.name : instance.info.id
						}
						onclick={() => {
							props.onStop(instance.info.id);
						}}
					/>
				)}
			</For>
		</div>
	);
}

interface RunningInstanceListProps {
	instances: RunningInstanceInfo[];
	onStop: (instance: string) => void;
}

// The mode for the footer
export enum FooterMode {
	Instance = "instance",
	Profile = "profile",
	SaveInstanceConfig = "save_instance_config",
	SaveProfileConfig = "save_profile_config",
	PreviewPackage = "preview_package",
}
