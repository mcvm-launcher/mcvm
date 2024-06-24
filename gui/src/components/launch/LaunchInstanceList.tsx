import { createSignal, For, Match, Show, Switch } from "solid-js";
import { GroupInfo, InstanceInfo, InstanceMap } from "../../types";
import { invoke } from "@tauri-apps/api";
import "./LaunchInstanceList.css";
import { Box, Folder, Pin } from "../../icons";
import Icon from "../Icon";
import IconButton from "../input/IconButton";
import { getIconSrc } from "../../utils";

export default function LaunchInstanceList(props: LaunchInstanceListProps) {
	const [instances, setInstances] = createSignal<InstanceInfo[]>([]);
	const [pinned, setPinned] = createSignal<InstanceInfo[]>([]);
	const [groups, setGroups] = createSignal<GroupSectionData[]>([]);
	const [selectedInstance, setSelectedInstance] = createSignal<string | null>(
		null
	);
	const [selectedSection, setSelectedSection] = createSignal<string | null>(
		null
	);

	async function updateInstances() {
		const instances = (await invoke("get_instances")) as InstanceInfo[];

		// Create map of instances and put pinned instances in their section
		let newPinned = [];
		let instanceMap: InstanceMap = {};
		for (let instance of instances) {
			if (instance.pinned) {
				newPinned.push(instance);
			}
			instanceMap[instance.id] = instance;
		}
		setPinned(newPinned);
		setInstances(instances);

		// Create groups
		const groups = (await invoke("get_instance_groups")) as GroupInfo[];
		let newGroups: GroupSectionData[] = [];
		for (let group of groups) {
			let newInstances = [];
			for (let instanceId of group.contents) {
				try {
					let instance = instanceMap[instanceId];
					newInstances.push(instance);
				} catch (e) {
					console.error(
						"Failed to fetch instance '" + instanceId + "' from map"
					);
				}
			}
			const newGroup: GroupSectionData = {
				id: group.id,
				instances: newInstances,
			};
			newGroups.push(newGroup);
		}
		setGroups(newGroups);
	}

	updateInstances();

	function onSelect(instance: string, section: string) {
		setSelectedInstance(instance);
		setSelectedSection(section);
		props.onSelectInstance(instance);
		console.log("Instance: " + selectedInstance());
	}

	return (
		<>
			<div id="launch-instance-list">
				<Show when={pinned().length > 0}>
					<Section
						id="pinned"
						kind="pinned"
						header="Pinned"
						instances={pinned()}
						selectedInstance={selectedInstance()}
						selectedSection={selectedSection()}
						onSelectInstance={onSelect}
						updateList={updateInstances}
					/>
				</Show>
				<For each={groups()}>
					{(item) => (
						<Section
							id={`group-${item.id}`}
							kind="group"
							header={item.id}
							instances={item.instances}
							selectedInstance={selectedInstance()}
							selectedSection={selectedSection()}
							onSelectInstance={onSelect}
							updateList={updateInstances}
						/>
					)}
				</For>
				<Section
					id="all"
					kind="all"
					header="All Instances"
					instances={instances()}
					selectedInstance={selectedInstance()}
					selectedSection={selectedSection()}
					onSelectInstance={onSelect}
					updateList={updateInstances}
				/>
			</div>
		</>
	);
}

// A section of instances, like pinned or an MCVM instance group
function Section(props: SectionProps) {
	const HeaderIcon = () => (
		<Switch>
			<Match when={props.kind == "all"}>
				<Icon icon={Box} size="18px" />
			</Match>
			<Match when={props.kind == "pinned"}>
				<Icon icon={Pin} size="18px" />
			</Match>
			<Match when={props.kind == "group"}>
				<Icon icon={Folder} size="18px" />
			</Match>
		</Switch>
	);

	return (
		<div class="launch-instance-list-section-container">
			<div class="launch-instance-list-section-header">
				<HeaderIcon />
				<h2>{props.header}</h2>
			</div>
			<div class="launch-instance-list-section">
				<For each={props.instances}>
					{(item) => (
						<Item
							instance={item}
							selected={
								props.selectedSection !== null &&
								props.selectedSection === props.id &&
								props.selectedInstance === item.id
							}
							onSelect={() => {
								props.onSelectInstance(item.id, props.id);
							}}
							sectionKind={props.kind}
							updateList={props.updateList}
						/>
					)}
				</For>
			</div>
		</div>
	);
}

interface SectionProps {
	id: string;
	kind: SectionKind;
	header: string;
	instances: InstanceInfo[];
	selectedInstance: string | null;
	selectedSection: string | null;
	onSelectInstance: (instance: string, section: string) => void;
	updateList: () => void;
}

type SectionKind = "pinned" | "group" | "all";

interface GroupSectionData {
	id: string;
	instances: InstanceInfo[];
}

function Item(props: ItemProps) {
	const [isHovered, setIsHovered] = createSignal(false);

	return (
		<div
			class={`launch-instance-list-item noselect border border-big ${
				props.selected ? "selected" : ""
			}`}
			onClick={props.onSelect}
			onMouseEnter={() => setIsHovered(true)}
			onMouseLeave={() => setIsHovered(false)}
		>
			{/* Don't show the pin button when the instance is already pinned and we aren't in the pinned section */}
			<Show
				when={
					isHovered() &&
					!(props.instance.pinned && props.sectionKind !== "pinned")
				}
			>
				<div class="launch-instance-list-pin">
					<IconButton
						icon={Pin}
						size="22px"
						color="var(--bg2)"
						selectedColor="var(--accent)"
						onClick={(e) => {
							// Don't select the instance
							e.stopPropagation();

							invoke("pin_instance", {
								instanceId: props.instance.id,
								pin: !props.instance.pinned,
							}).then(props.updateList);
						}}
						selected={props.sectionKind === "pinned"}
					/>
				</div>
			</Show>
			<img
				src={getIconSrc(props.instance.icon)}
				class="launch-instance-list-icon"
			/>
			<div style="" class="bold">
				{props.instance.name !== null ? props.instance.name : props.instance.id}
			</div>
			<Show when={props.instance.name !== null}>
				<div style="color: var(--fg3)">{props.instance.id}</div>
			</Show>
		</div>
	);
}

interface ItemProps {
	instance: InstanceInfo;
	selected: boolean;
	sectionKind: SectionKind;
	onSelect: () => void;
	updateList: () => void;
}

export interface LaunchInstanceListProps {
	onSelectInstance: (instance: string) => void;
}
