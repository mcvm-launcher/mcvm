import { createSignal, For, Match, Show, Switch } from "solid-js";
import { GroupInfo, InstanceInfo, InstanceMap } from "../../types";
import { invoke } from "@tauri-apps/api";
import "./LaunchInstanceList.css";
import { Box, Edit, Folder, Pin, Plus } from "../../icons";
import Icon from "../Icon";
import IconButton from "../input/IconButton";
import { getIconSrc } from "../../utils";
import IconTextButton from "../input/IconTextButton";
import { FooterData } from "../../App";
import { FooterMode } from "./Footer";

export default function LaunchInstanceList(props: LaunchInstanceListProps) {
	const [instances, setInstances] = createSignal<InstanceInfo[]>([]);
	const [profiles, setProfiles] = createSignal<InstanceInfo[]>([]);
	const [pinned, setPinned] = createSignal<InstanceInfo[]>([]);
	const [groups, setGroups] = createSignal<GroupSectionData[]>([]);
	const [selectedItem, setSelectedItem] = createSignal<
		SelectedItem | undefined
	>(undefined);
	const [selectedSection, setSelectedSection] = createSignal<string | null>(
		null
	);
	const [instancesOrProfiles, setInstancesOrProfiles] = createSignal<
		"instance" | "profile"
	>("instance");

	async function updateItems() {
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
		const profiles = (await invoke("get_profiles")) as InstanceInfo[];
		let profileMap: InstanceMap = {};
		for (let profile of profiles) {
			profileMap[profile.id] = profile;
		}
		setProfiles(profiles);

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

	updateItems();

	function onSelect(item: SelectedItem, section: string) {
		setSelectedItem(item);
		setSelectedSection(section);
		props.setFooterData({
			selectedItem: item.id,
			mode: item.type as FooterMode,
			action: () => {},
		});
		console.log("Selected item: " + selectedItem());
	}

	return (
		<>
			<div id="launch-instance-list">
				<div class="cont">
					<div id="launch-instance-list-header">
						<div
							class={`launch-instance-list-header-item instances${
								instancesOrProfiles() == "instance" ? " selected" : ""
							}`}
							onclick={() => {
								setInstancesOrProfiles("instance");
							}}
						>
							Instances
						</div>
						<div
							class={`launch-instance-list-header-item profiles${
								instancesOrProfiles() == "profile" ? " selected" : ""
							}`}
							onclick={() => {
								setInstancesOrProfiles("profile");
							}}
						>
							Profiles
						</div>
					</div>
				</div>
				<br />
				<Switch>
					<Match when={instancesOrProfiles() == "instance"}>
						<Show when={pinned().length > 0}>
							<Section
								id="pinned"
								kind="pinned"
								header="PINNED"
								items={pinned()}
								selectedItem={selectedItem()}
								selectedSection={selectedSection()}
								onSelectItem={onSelect}
								updateList={updateItems}
								itemType="instance"
							/>
						</Show>
						<For each={groups()}>
							{(item) => (
								<Section
									id={`group-${item.id}`}
									kind="group"
									header={item.id.toLocaleUpperCase()}
									items={item.instances}
									selectedItem={selectedItem()}
									selectedSection={selectedSection()}
									onSelectItem={onSelect}
									updateList={updateItems}
									itemType="instance"
								/>
							)}
						</For>
						<Section
							id="all"
							kind="all"
							header="ALL INSTANCES"
							items={instances()}
							selectedItem={selectedItem()}
							selectedSection={selectedSection()}
							onSelectItem={onSelect}
							updateList={updateItems}
							itemType="instance"
						/>
					</Match>
					<Match when={instancesOrProfiles() == "profile"}>
						<br />
						<div class="cont">
							<IconTextButton
								icon={Edit}
								text="Edit Global Profile"
								size="20px"
								color="var(--bg2)"
								selectedColor="var(--instance)"
								onClick={() => {
									window.location.href = "/global_profile_config";
								}}
								selected={false}
							/>
						</div>
						<br />
						<Section
							id="profiles"
							kind="profiles"
							header="ALL PROFILES"
							items={profiles()}
							selectedItem={selectedItem()}
							selectedSection={selectedSection()}
							onSelectItem={onSelect}
							updateList={updateItems}
							itemType="profile"
						/>
					</Match>
				</Switch>
			</div>
		</>
	);
}

// A section of items, like pinned or an MCVM instance group
function Section(props: SectionProps) {
	const HeaderIcon = () => (
		<Switch>
			<Match when={props.kind == "all" || props.kind == "profiles"}>
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
		<div class="cont col">
			<div class="cont col launch-instance-list-section-container">
				<div class="cont launch-instance-list-section-header">
					<HeaderIcon />
					<h2>{props.header}</h2>
				</div>
				<div class="launch-instance-list-section">
					<For each={props.items}>
						{(item) => (
							<Item
								instance={item}
								selected={
									props.selectedSection !== null &&
									props.selectedSection === props.id &&
									props.selectedItem?.id === item.id
								}
								onSelect={() => {
									props.onSelectItem(
										{ id: item.id, type: props.itemType },
										props.id
									);
								}}
								sectionKind={props.kind}
								itemKind={props.itemType}
								updateList={props.updateList}
							/>
						)}
					</For>
					{/* Button for creating a new instance */}
					<Show when={props.kind == "all" || props.kind == "profiles"}>
						<div
							class="input-shadow launch-instance-list-item noselect"
							onclick={() => {
								let target =
									props.itemType == "instance"
										? "create_instance"
										: "create_profile";
								window.location.href = target;
							}}
						>
							<div class="cont launch-instance-list-icon">
								<Plus />
							</div>
							<div style="" class="bold">
								{`Create ${
									props.itemType == "instance" ? "Instance" : "Profile"
								}`}
							</div>
						</div>
					</Show>
				</div>
			</div>
		</div>
	);
}

interface SectionProps {
	id: string;
	kind: SectionKind;
	itemType: "instance" | "profile";
	header: string;
	items: InstanceInfo[];
	selectedItem?: SelectedItem;
	selectedSection: string | null;
	onSelectItem: (item: SelectedItem, section: string) => void;
	updateList: () => void;
}

type SectionKind = "pinned" | "group" | "all" | "profiles";

interface GroupSectionData {
	id: string;
	instances: InstanceInfo[];
}

function Item(props: ItemProps) {
	const [isHovered, setIsHovered] = createSignal(false);

	return (
		<div
			class={`input-shadow launch-instance-list-item noselect ${
				props.selected ? "selected" : ""
			} ${props.itemKind}`}
			onClick={props.onSelect}
			onMouseEnter={() => setIsHovered(true)}
			onMouseLeave={() => setIsHovered(false)}
		>
			{/* Don't show the pin button when the instance is already pinned and we aren't in the pinned section */}
			<Show
				when={
					isHovered() &&
					props.itemKind == "instance" &&
					!(props.instance.pinned && props.sectionKind !== "pinned")
				}
			>
				<div class="launch-instance-list-pin">
					<IconButton
						icon={Pin}
						size="22px"
						color="var(--bg2)"
						selectedColor="var(--instance)"
						iconColor={
							props.sectionKind == "pinned" ? "var(--bg2)" : "var(--fg)"
						}
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
			<div class="launch-instance-list-item-details">
				<div style="" class="bold">
					{props.instance.name !== null
						? props.instance.name
						: props.instance.id}
				</div>
				<Show when={props.instance.name !== null}>
					<div style="color: var(--fg3)">{props.instance.id}</div>
				</Show>
			</div>
		</div>
	);
}

interface ItemProps {
	instance: InstanceInfo;
	selected: boolean;
	sectionKind: SectionKind;
	itemKind: "instance" | "profile";
	onSelect: () => void;
	updateList: () => void;
}

export interface LaunchInstanceListProps {
	setFooterData: (data: FooterData) => void;
}

interface SelectedItem {
	id?: string;
	type: "instance" | "profile";
}
