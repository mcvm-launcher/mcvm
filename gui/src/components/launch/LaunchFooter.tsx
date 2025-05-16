import { For, Show, createEffect, createSignal, onCleanup } from "solid-js";
import "./LaunchFooter.css";
import { UnlistenFn, listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";
import { PasswordPrompt } from "../input/PasswordPrompt";
import { Play, Properties } from "../../icons";
import IconTextButton from "../input/IconTextButton";
import IconButton from "../input/IconButton";
import { AuthDisplayEvent, RunningInstanceInfo } from "../../types";
import MicrosoftAuthInfo from "../input/MicrosoftAuthInfo";
import { getIconSrc } from "../../utils";

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
	createEffect(async () => {
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
	}, []);

	onCleanup(() => {
		for (const unlisten of unlistens()) {
			unlisten();
		}
	});

	async function launch() {
		if (
			props.selectedItem == undefined ||
			props.selectedItem.type != "instance"
		) {
			return;
		}

		// Prevent launching until the current authentication screens are finished
		if (showPasswordPrompt() || authInfo() !== null) {
			return;
		}

		let launchPromise = invoke("launch_game", {
			instanceId: props.selectedItem.id,
			offline: false,
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
		<div class="launch-footer border">
			<div class="launch-footer-section launch-footer-left"></div>
			<div class="launch-footer-section launch-footer-center">
				<div class="launch-footer-center-inner">
					<div class="launch-button-container">
						<div class="launch-footer-config">
							<IconButton
								icon={Properties}
								size="28px"
								color="var(--bg2)"
								selectedColor="var(--accent)"
								onClick={() => {
									if (props.selectedItem != undefined) {
										window.location.href = `/${props.selectedItem.type}_config/${props.selectedItem.id}`;
									}
								}}
								selected={false}
							/>
						</div>
						<div class="launch-button">
							<IconTextButton
								icon={Play}
								text="Launch"
								size="22px"
								color="var(--bg2)"
								selectedColor="var(--instance)"
								onClick={() => {
									launch();
								}}
								selected={props.selectedItem != undefined}
							/>
						</div>
					</div>
				</div>
			</div>
			<div class="launch-footer-section launch-footer-right">
				<RunningInstanceList instances={runningInstances()} onStop={stopGame} />
			</div>

			<Show when={authInfo() !== null}>
				<MicrosoftAuthInfo
					event={authInfo() as AuthDisplayEvent}
					onCancel={() => {
						setAuthInfo(null);
						if (props.selectedItem != undefined) {
							stopGame(props.selectedItem.id);
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
	selectedItem?: SelectedFooterItem;
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

// The object that is selected for the footer, an instance or profile
export interface SelectedFooterItem {
	type: "profile" | "instance";
	id: string;
}
