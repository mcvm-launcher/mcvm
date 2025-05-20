import { useParams } from "@solidjs/router";
import { invoke } from "@tauri-apps/api";
import { createResource } from "solid-js";

export default function CustomPluginPage() {
	let params = useParams();

	let [html, _] = createResource(async () => {
		let html: string | undefined = await invoke("get_plugin_page", {
			page: params.page,
		});
		return html;
	});

	return <div id="custom-plugin-page" innerHTML={html()}></div>;
}
