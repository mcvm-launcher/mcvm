import { useParams } from "@solidjs/router";
import { invoke } from "@tauri-apps/api";
import { createEffect, createResource, createSignal } from "solid-js";
import { setupPluginFunctions } from "../plugins";

export default function CustomPluginPage() {
	let params = useParams();

	let [html, _] = createResource(
		() => params.page,
		async (page) => {
			let html: string | undefined = await invoke("get_plugin_page", {
				page: page,
			});
			return html;
		}
	);

	// We need to run script tags
	// https://stackoverflow.com/questions/2592092/executing-script-elements-inserted-with-innerhtml
	createEffect(() => {
		setupPluginFunctions();
		let page = document.getElementById("custom-plugin-page")!;
		if (html() == undefined) {
			return;
		}
		page.innerHTML = html()!;

		let scripts = page.getElementsByTagName("script");
		if (scripts != undefined) {
			console.log("not undefined");
			for (let oldScriptEl of scripts) {
				console.log("script");
				const newScriptEl = document.createElement("script");

				Array.from(oldScriptEl.attributes).forEach((attr) => {
					newScriptEl.setAttribute(attr.name, attr.value);
				});

				const scriptText = document.createTextNode(oldScriptEl.innerHTML);
				newScriptEl.appendChild(scriptText);

				oldScriptEl.parentNode!.replaceChild(newScriptEl, oldScriptEl);
			}
		}
	});

	return <div id="custom-plugin-page"></div>;
}
