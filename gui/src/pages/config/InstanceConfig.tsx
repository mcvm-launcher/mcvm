import { useParams } from "@solidjs/router";
import "./InstanceConfig.css";
import IconTextButton from "../../components/input/IconTextButton";
import { AngleLeft } from "../../icons";
import { invoke } from "@tauri-apps/api";
import { createEffect, createResource, createSignal } from "solid-js";

export default function InstanceConfig(props: InstanceConfigProps) {
	let params = useParams();
	// let [beforeProfiles, setBeforeProfiles] = createSignal<InstanceConfig>();
	// let [afterProfiles, setAfterProfiles] = createSignal<InstanceConfig>();

	let [configs] = createResource(updateConfig);

	async function updateConfig() {
		let result = await invoke("get_instance_config", { instance: params.id });
		let [before_profiles, after_profiles] = result as InstanceConfig[];

		return [before_profiles, after_profiles];
	}

	let [displayName, setDisplayName] = createSignal("");

	createEffect(() => {
		if (configs() != undefined) {
			let [beforeProfiles, afterProfiles] = configs() as InstanceConfig[];

			setDisplayName(
				afterProfiles.name == null ? params.id : afterProfiles.name
			);
		}
	});

	return (
		<div class="container" style="width:100%">
			<h1 class="noselect">Configuration for Instance {displayName()}</h1>
			<div class="back-button">
				<IconTextButton
					icon={AngleLeft}
					size="22px"
					text="Back"
					color="var(--bg2)"
					selectedColor="var(--bg2)"
					onClick={() => {
						history.back();
					}}
					selected={false}
				/>
			</div>
			<br />
		</div>
	);
}

export interface InstanceConfigProps {}

interface InstanceConfig {
	side?: "client" | "server";
	name?: string;
	icon?: string;
}
