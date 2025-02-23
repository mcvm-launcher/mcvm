import { invoke } from "@tauri-apps/api";
import { createSignal } from "solid-js";
import "./PasswordPrompt.css";
import PageBlock from "../PageBlock";

export function PasswordPrompt(props: PasswordPromptProps) {
	const [answer, setAnswer] = createSignal<string | null>(null);

	return (
		<>
			<PageBlock />
			<div class="password-prompt border">
				<form
					class="row"
					onSubmit={(e) => {
						if (answer === null) {
							return;
						}

						e.preventDefault();
						props.onSubmit();

						invoke("answer_password_prompt", { answer: answer() });

						// Clear the input
						setAnswer(null);
					}}
				>
					<input
						type="password"
						class="password-prompt-input"
						onChange={(e) => setAnswer(e.currentTarget.value)}
						placeholder="Enter your passkey..."
					/>
					<button class="password-prompt-submit" type="submit">
						Submit
					</button>
				</form>
			</div>
		</>
	);
}

export interface PasswordPromptProps {
	message: string;
	onSubmit: () => void;
}
