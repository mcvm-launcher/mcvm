import { createSignal } from "solid-js";
import "./LaunchFooter.css";
import { UnlistenFn, listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";
import { PasswordPrompt } from "../input/PasswordPrompt";

export default function LaunchFooter(props: LaunchFooterProps) {
	// Prompts
	const [showPasswordPrompt, setShowPasswordPrompt] = createSignal(false);
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

		let passwordPromise = listen(
			"mcvm_display_password_prompt",
			(event: Event<string>) => {
				setShowPasswordPrompt(true);
				setPasswordPromptMessage(event.payload);
			}
		);

		let [_, ...eventUnlistens] = await Promise.all([
			launchPromise,
			passwordPromise,
		]);

		setUnlistens(eventUnlistens);
	}

	return (
		<div class="launch-footer border">
			<button
				class={`launch-button ${props.selectedInstance === null ? "" : "selected"}`}
				onClick={(e) => {
					e.preventDefault();
					launch();
				}}
			>
				Launch
			</button>
			<div style={`display:${showPasswordPrompt() ? "block" : "none"}`}>
				<PasswordPrompt
					onSubmit={() => setShowPasswordPrompt(false)}
					message={passwordPromptMessage()}
				/>
			</div>
		</div>
	);
}

export interface LaunchFooterProps {
	selectedInstance?: string;
}
