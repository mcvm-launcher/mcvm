import "./LaunchPage.css";
import LaunchInstanceList from "../../components/launch/LaunchInstanceList";
import { createEffect, onMount } from "solid-js";
import { loadPagePlugins } from "../../plugins";
import { FooterData } from "../../App";
import { FooterMode } from "../../components/launch/Footer";

export default function LaunchPage(props: LaunchPageProps) {
	onMount(() => loadPagePlugins("launch"));
	createEffect(() => {
		props.setFooterData({
			mode: FooterMode.Instance,
			selectedItem: undefined,
			action: () => {},
		});
	});
	return (
		<div class="container">
			<br />
			<LaunchInstanceList setFooterData={props.setFooterData} />
			<br />
		</div>
	);
}

export interface LaunchPageProps {
	setFooterData: (data: FooterData) => void;
}
