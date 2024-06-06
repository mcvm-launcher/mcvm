import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { Event, UnlistenFn, listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
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
			<h1>MCVM</h1>
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

export default App;

function PasswordPrompt(props: PasswordPromptProps) {
	const [answer, setAnswer] = createSignal<string | null>(null);

	return (
		<div class="password-prompt">
			<form
				class="row"
				onSubmit={(e) => {
					if (answer === null) {
						return;
					}

					e.preventDefault();
					props.onSubmit();
          console.log("Got");

					invoke("answer_password_prompt", { answer: answer() });
				}}
			>
				<input
					type="password"
					id="password-prompt-input"
					onChange={(e) => setAnswer(e.currentTarget.value)}
					placeholder="Enter your password..."
				/>
				<button type="submit">Submit</button>
			</form>
		</div>
	);
}

interface PasswordPromptProps {
	message: string;
	onSubmit: () => void;
}
