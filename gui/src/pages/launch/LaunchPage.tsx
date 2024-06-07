import { createSignal } from "solid-js";
import { UnlistenFn, listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";
import { PasswordPrompt } from "../../components/input/PasswordPrompt";
import "./LaunchPage.css";

export default function LaunchPage() {
	const [name, setName] = createSignal("");
	// Prompts
	const [showPasswordPrompt, setShowPasswordPrompt] = createSignal(false);
	const [passwordPromptMessage, setPasswordPromptMessage] = createSignal("");
	// Unlisteners for tauri events
	const [unlistens, setUnlistens] = createSignal<UnlistenFn[]>([]);

	async function greet() {
		// Make sure we unlisten from all of the existing listeners
		for (let unlisten of unlistens()) {
			unlisten();
		}
		let launchPromise = invoke("launch_game", {
			instanceId: name(),
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
		<div class="container">
			<h1>Launch</h1>
			<br />

			<form
				class="row"
				onSubmit={(e) => {
					e.preventDefault();
					greet();
				}}
			>
				<input
					id="greet-input"
					onChange={(e) => setName(e.currentTarget.value)}
					placeholder="Enter an instance to launch..."
				/>
				<button type="submit">Launch</button>
			</form>
			<br />

			<div style={`display:${showPasswordPrompt() ? "block" : "hidden"}`}>
				<PasswordPrompt
					onSubmit={() => setShowPasswordPrompt(false)}
					message={passwordPromptMessage()}
				/>
			</div>
		</div>
	);
}
