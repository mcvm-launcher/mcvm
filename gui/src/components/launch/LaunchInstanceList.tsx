import { createSignal, For } from "solid-js";
import { InstanceInfo } from "../../types";
import { invoke } from "@tauri-apps/api";
import "./LaunchInstanceList.css";

export default function LaunchInstanceList(props: LaunchInstanceListProps) {
	const [instances, setInstances] = createSignal<InstanceInfo[]>([]);
	const [selected, setSelected] = createSignal<string | null>(null);

	async function updateInstances() {
		const instances = await invoke("get_instances");
		setInstances(instances as InstanceInfo[]);
		console.log(instances);
	}

	updateInstances();

	return (
		<div id="launch-instance-list-container">
			<div id="launch-instance-list">
				<For each={instances()}>
					{(item) => (
						<div
							class={`launch-instance-list-item noselect border ${
								item.id == selected() ? "selected" : ""
							}`}
							onClick={() => {
								setSelected(item.id);
								props.onSelectInstance(item.id);
							}}
						>
							{item.id}
						</div>
					)}
				</For>
			</div>
		</div>
	);
}

export interface LaunchInstanceListProps {
	onSelectInstance: (instance: string) => void;
}
