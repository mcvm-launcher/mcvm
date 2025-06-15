import { Event, listen } from "@tauri-apps/api/event";
import { createResource, createSignal, onCleanup, Show } from "solid-js";
import "./TaskIndicator.css";
import { Spinner } from "../icons";

export default function TaskIndicator(props: TaskIndicatorProps) {
	// Map of tasks to messages
	let [messages, setMessages] = createSignal<TaskMap>({});
	let [taskCount, setTaskCount] = createSignal(0);
	let [taskName, setTaskName] = createSignal<string | undefined>(undefined);
	let [color, setColor] = createSignal<Color>("disabled");

	function createTask(task: string) {
		if (messages()[task] == undefined) {
			setTaskCount((taskCount) => taskCount + 1);
		}
		if (taskCount() == 1) {
			setTaskName(getTaskDisplayName(task));
			setColor(getTaskColor(task));
		}
		setMessages((messages) => {
			messages[task] = [];
			return messages;
		});
	}

	let [eventUnlistens, _] = createResource(async () => {
		let unlisten1 = listen(
			"mcvm_output_create_task",
			(event: Event<string>) => {
				createTask(event.payload);
			}
		);

		(window as any).foo = messages;
		(window as any).bar = taskCount;

		let unlisten2 = listen(
			"mcvm_output_message",
			(event: Event<MessageEvent>) => {
				if (event.payload.task != undefined) {
					setMessages((messages) => {
						if (messages[event.payload.task!] != undefined) {
							messages[event.payload.task!]!.push({
								type: "message",
								message: event.payload.message,
								messageType: event.payload.type,
							});
						}
						return messages;
					});
				}
			}
		);

		let unlisten3 = listen(
			"mcvm_output_finish_task",
			(event: Event<string>) => {
				if (messages()[event.payload] != undefined) {
					setTaskCount((taskCount) => taskCount - 1);
				}
				setMessages((messages) => {
					delete messages[event.payload];
					return messages;
				});
				if (taskCount() == 0) {
					setColor("disabled");
				} else if (taskCount() == 1) {
					setTaskName(getTaskDisplayName(Object.keys(messages())[0]!));
				}
			}
		);

		return await Promise.all([unlisten1, unlisten2, unlisten3]);
	});

	onCleanup(() => {
		if (eventUnlistens() != undefined) {
			for (let unlisten of eventUnlistens()!) {
				unlisten();
			}
		}
	});

	return (
		<div id="task-indicator" style={`border-color:${getColors(color())[0]}`}>
			<div id="task-indicator-preview" style={`color:${getColors(color())[1]}`}>
				<Show
					when={taskCount() > 0}
					fallback={
						<div class="cont">
							<div
								id="task-indicator-dot"
								style={`background-color:${getColors(color())[0]}`}
							></div>
						</div>
					}
				>
					<div
						class="cont rotating"
						id="task-indicator-spinner"
						style={`color:${getColors(color())[0]}`}
					>
						<Spinner />
					</div>
				</Show>
				<div class="cont">
					<Show
						when={taskCount() == 1}
						fallback={`${taskCount()} ${
							taskCount() == 1 ? "task" : "tasks"
						} running`}
					>
						{taskName()}
					</Show>
					{/* {`${taskCount()} ${taskCount() == 1 ? "task" : "tasks"} running`} */}
				</div>
			</div>
		</div>
	);
}

export interface TaskIndicatorProps {}

function Message(props: MessageProps) {
	return <div></div>;
}

interface MessageProps {
	data: MessageData;
}

type MessageData = {
	type: "message";
	message: string;
	messageType: MessageType;
};

type TaskMap = {
	[task: string]: MessageData[] | undefined;
};

export interface MessageEvent {
	message: string;
	type: MessageType;
	task?: string;
}

enum MessageType {
	Simple = "simple",
	Header = "header",
}

function getTaskDisplayName(task: string) {
	if (task == "get_plugins") {
		return "Getting plugins";
	} else if (task == "update_instance") {
		return "Updating instance";
	} else if (task.startsWith("launch_instance")) {
		return "Launching";
	} else if (task == "search_packages") {
		return "Searching packages";
	} else if (task == "load_packages") {
		return "Loading packages";
	} else if (task == "sync_packages") {
		return "Syncing packages";
	}
	return task;
}

function getTaskColor(task: string) {
	if (task == "get_plugins") {
		return "plugin";
	} else if (task.startsWith("launch_instance")) {
		return "instance";
	} else if (task == "update_instance") {
		return "profile";
	} else if (
		task == "search_packages" ||
		task == "load_packages" ||
		task == "sync_packages"
	) {
		return "package";
	}

	return "running";
}

type Color =
	| "disabled"
	| "running"
	| "instance"
	| "profile"
	| "package"
	| "plugin";

// Gets the border and text colors of a color preset
function getColors(color: Color) {
	if (color == "running") {
		return ["var(--bg3)", "var(--fg)"];
	} else if (color == "instance") {
		return ["var(--instance)", "var(--instance)"];
	} else if (color == "profile") {
		return ["var(--profile)", "var(--profile)"];
	} else if (color == "package") {
		return ["var(--package)", "var(--package)"];
	} else if (color == "plugin") {
		return ["var(--plugin)", "var(--pluginfg)"];
	}
	return ["var(--bg3)", "var(--fg3)"];
}
