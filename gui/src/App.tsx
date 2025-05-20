import { Router, Route, Location } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";
import { createSignal, onMount, Show } from "solid-js";
import LaunchFooter, {
	SelectedFooterItem,
} from "./components/launch/LaunchFooter";
import InstanceConfig, { ConfigMode } from "./pages/config/InstanceConfig";
import BrowsePackages from "./pages/package/BrowsePackages";
import ViewPackage from "./pages/package/ViewPackage";
import Sidebar from "./components/navigation/Sidebar";
import Plugins from "./pages/plugin/Plugins";
import Smithed from "./pages/Smithed";
import Docs from "./pages/Docs";
import { loadPagePlugins } from "./plugins";
import { listen } from "@tauri-apps/api/event";

export default function App() {
	const [selectedItem, setSelectedItem] = createSignal<
		SelectedFooterItem | undefined
	>(undefined);

	let [selectedUser, setSelectedUser] = createSignal<string>();
	
	// Window refresh logic
	let [showUi, setShowUi] = createSignal(true);
	listen("refresh_window", () => {
		setShowUi(false);
		setShowUi(true);
	});

	return (
		<Show when={showUi()}>
			<Router
				root={({ children, location }) => (
					<Layout
						children={children}
						location={location}
						selectedItem={selectedItem()}
						onSelectUser={setSelectedUser}
						selectedUser={selectedUser()}
					/>
				)}
			>
				<Route
					path="/"
					component={() => <LaunchPage onSelectItem={setSelectedItem} />}
				/>
				<Route
					path="/instance_config/:instanceId"
					component={() => (
						<InstanceConfig mode={ConfigMode.Instance} creating={false} />
					)}
				/>
				<Route
					path="/profile_config/:profileId"
					component={() => (
						<InstanceConfig mode={ConfigMode.Profile} creating={false} />
					)}
				/>
				<Route
					path="/create_instance"
					component={() => (
						<InstanceConfig mode={ConfigMode.Instance} creating={true} />
					)}
				/>
				<Route
					path="/create_profile"
					component={() => (
						<InstanceConfig mode={ConfigMode.Profile} creating={true} />
					)}
				/>
				<Route
					path="/global_profile_config"
					component={() => (
						<InstanceConfig mode={ConfigMode.GlobalProfile} creating={false} />
					)}
				/>
				<Route path="/packages/:page" component={() => <BrowsePackages />} />
				<Route path="/packages/package/:id" component={() => <ViewPackage />} />
				<Route path="/plugins" component={() => <Plugins />} />
				<Route path="/docs" component={() => <Docs />} />
				<Route path="/smithed" component={() => <Smithed />} />
			</Router>
		</Show>
	);
}

function Layout(props: LayoutProps) {
	let [showSidebar, setShowSidebar] = createSignal(false);

	onMount(() => loadPagePlugins(""));

	return (
		<>
			<NavBar
				onSidebarToggle={() => {
					setShowSidebar(!showSidebar());
				}}
				onSelectUser={props.onSelectUser}
			/>
			<Sidebar
				visible={showSidebar()}
				location={props.location}
				setVisible={setShowSidebar}
			/>
			{props.children}
			<LaunchFooter
				selectedItem={props.selectedItem}
				selectedUser={props.selectedUser}
			/>
		</>
	);
}

interface LayoutProps {
	children: any;
	location: Location;
	selectedItem?: SelectedFooterItem;
	selectedUser?: string;
	onSelectUser: (user: string) => void;
}
