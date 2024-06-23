import { Show, createSignal } from "solid-js";
import "./LaunchFooter.css";
import { UnlistenFn, listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";
import { PasswordPrompt } from "../input/PasswordPrompt";
import { Play, Properties } from "../../icons";
import IconTextButton from "../input/IconTextButton";
import IconButton from "../input/IconButton";
import { AuthDisplayEvent } from "../../types";
import MicrosoftAuthInfo from "../input/MicrosoftAuthInfo";

export default function LaunchFooter(props: LaunchFooterProps) {
	// Basic state
	const [state, setState] = createSignal<LaunchState>("not_started");
	const [isHovered, setIsHovered] = createSignal(false);
	// Prompts
	const [showPasswordPrompt, setShowPasswordPrompt] = createSignal(false);
	const [authInfo, setAuthInfo] = createSignal<AuthDisplayEvent | null>(null);
	const [passwordPromptMessage, setPasswordPromptMessage] = createSignal("");
	// Unlisteners for tauri events
	const [unlistens, setUnlistens] = createSignal<UnlistenFn[]>([]);

	async function launch() {
		if (props.selectedInstance === null) {
			return;
		}

		// Make sure we unlisten from all of the existing listeners
		for (let unlisten of unlistens()) {
			unlisten();
		}
		let launchPromise = invoke("launch_game", {
			instanceId: props.selectedInstance,
			offline: false,
		});

		let prepareLaunchPromise = listen("mcvm_prepare_launch", () => {
			console.log("Preparing");
			setState("preparing");
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

		let launchingPromise = listen("mcvm_launching", () => {
			console.log("Running");
			setState("running");
		});

		let stoppedPromise = listen("game_finished", () => {
			console.log("Stopped");
			setState("not_started");
		});

		let [_, ...eventUnlistens] = await Promise.all([
			launchPromise,
			prepareLaunchPromise,
			authInfoPromise,
			authInfoClosePromise,
			passwordPromise,
			launchingPromise,
			stoppedPromise,
		]);

		setUnlistens(eventUnlistens);
	}

	async function stopGame() {
		setState("not_started");
		setAuthInfo(null);
		setShowPasswordPrompt(false);
		await invoke("stop_game", {});
	}

	return (
		<div class="launch-footer border">
			<div class="launch-footer-config">
				<IconButton
					icon={Properties}
					size="28px"
					color="var(--bg2)"
					selectedColor="var(--accent)"
					onClick={() => {}}
					selected={false}
				/>
			</div>
			<div
				onMouseEnter={() => setIsHovered(true)}
				onMouseLeave={() => setIsHovered(false)}
			>
				<IconTextButton
					icon={Play}
					text={
						state() === "not_started"
							? "Launch"
							: state() === "preparing"
							? "Preparing..."
							: state() === "running"
							? "Running"
							: "Invalid state"
					}
					size="22px"
					color="var(--bg2)"
					selectedColor="var(--accent)"
					onClick={() => {
						launch();
					}}
					selected={props.selectedInstance !== null}
				/>
			</div>

			<Show when={authInfo() !== null}>
				<MicrosoftAuthInfo
					event={authInfo() as AuthDisplayEvent}
					onCancel={() => {
						setAuthInfo(null);
						stopGame();
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

type LaunchState = "not_started" | "preparing" | "running";

export interface LaunchFooterProps {
	selectedInstance: string | null;
}
