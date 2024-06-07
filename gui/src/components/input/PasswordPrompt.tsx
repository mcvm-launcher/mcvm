import { invoke } from "@tauri-apps/api";
import { createSignal } from "solid-js";

export function PasswordPrompt(props: PasswordPromptProps) {
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

export interface PasswordPromptProps {
	message: string;
	onSubmit: () => void;
}
