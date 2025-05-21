import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, For, Show } from "solid-js";
import { AngleRight } from "../icons";
import "./UserWidget.css";
import { stringCompare } from "../utils";

export default function UserWidget(props: UserWidgetProps) {
	let [userData, methods] = createResource(updateUsers);

	let [isOpen, setIsOpen] = createSignal(false);

	async function updateUsers() {
		let [currentUser, users] = (await invoke("get_users")) as [
			string | undefined,
			UserMap
		];

		let currentUserInfo =
			currentUser == undefined ? undefined : users[currentUser];

		if (currentUser != undefined) {
			users[currentUser] = undefined;
		}

		let userList = [];
		for (let user of Object.values(users)) {
			if (user != undefined) {
				userList.push(user);
			}
		}
		userList.sort((a, b) => stringCompare(a.id, b.id));

		return {
			currentUser: currentUserInfo,
			users: userList,
		} as UserData;
	}

	return (
		<div id="user-widget">
			<div
				id="user-widget-head"
				class={isOpen() ? "open" : ""}
				onclick={() => setIsOpen(!isOpen())}
			>
				<Show
					when={userData() != undefined && userData()!.currentUser != undefined}
					fallback={
						<div class="cont" id="user-widget-placeholder">
							No User Selected
						</div>
					}
				>
					<UserTile user={userData()!.currentUser!} onclick={() => {}} />
				</Show>
				<div class="cont" id="user-widget-dropdown-button">
					<AngleRight />
				</div>
			</div>
			<Show when={isOpen() && userData() != undefined}>
				<div id="user-widget-dropdown">
					<For each={userData()!.users}>
						{(user) => (
							<Show when={user != undefined}>
								<div class="user-widget-dropdown-item">
									<UserTile
										user={user!}
										onclick={(user) => {
											invoke("select_user", { user: user }).then(() => {
												methods.refetch();
											});
										}}
									/>
								</div>
							</Show>
						)}
					</For>
				</div>
			</Show>
		</div>
	);
}

export interface UserWidgetProps {
	onSelect: (user: string) => void;
}

interface UserData {
	currentUser?: UserInfo;
	users: UserInfo[];
}

type UserMap = { [id: string]: UserInfo | undefined };

export interface UserInfo {
	id: string;
	type: "microsoft" | "demo" | "other";
	username?: string;
	uuid?: string;
}

function UserTile(props: UserTileProps) {
	return (
		<div class="user-tile" onclick={() => props.onclick(props.user.id)}>
			{/* <Show when={props.user.uuid != undefined} fallback={<div></div>}> */}
			<div class="cont">
				<img
					class="user-tile-image"
					src={
						props.user.uuid == undefined
							? "/default_skin.png"
							: `https://crafatar.com/avatars/${props.user.uuid}?overlay`
					}
				/>
			</div>
			{/* </Show> */}
			<div class="cont user-tile-name">
				{props.user.username == undefined ? props.user.id : props.user.username}
			</div>
		</div>
	);
}

interface UserTileProps {
	user: UserInfo;
	onclick: (user: string) => void;
}
