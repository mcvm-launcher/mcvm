import { createSignal } from "solid-js";
import "./LaunchPage.css";
import LaunchInstanceList from "../../components/launch/LaunchInstanceList";
import LaunchFooter from "../../components/launch/LaunchFooter";

export default function LaunchPage() {
	const [selected, setSelected] = createSignal("");

	return (
		<div class="container">
			<h1 class="noselect">Launch</h1>
			<br />
			<LaunchInstanceList onSelectInstance={setSelected} />

			<LaunchFooter selectedInstance={selected()} />
		</div>
	);
}
