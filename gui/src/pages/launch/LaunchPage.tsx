import "./LaunchPage.css";
import LaunchInstanceList from "../../components/launch/LaunchInstanceList";
import { SelectedFooterItem } from "../../components/launch/LaunchFooter";
import { onMount } from "solid-js";
import { loadPagePlugins } from "../../plugins";

export default function LaunchPage(props: LaunchPageProps) {
	onMount(() => loadPagePlugins("launch"));
	return (
		<div class="container">
			{/* <h1 class="noselect">Launch</h1> */}
			<br />
			<LaunchInstanceList onSelect={props.onSelectItem} />
			<br />
		</div>
	);
}

export interface LaunchPageProps {
	onSelectItem: (item: SelectedFooterItem) => void;
}
