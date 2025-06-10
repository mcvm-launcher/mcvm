import { Router, Route, Location } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";
import { createSignal, onMount, Show } from "solid-js";
import InstanceConfig, { ConfigMode } from "./pages/config/InstanceConfig";
import BrowsePackages from "./pages/package/BrowsePackages";
import ViewPackage from "./pages/package/ViewPackage";
import Sidebar from "./components/navigation/Sidebar";
import Plugins from "./pages/plugin/Plugins";
import Docs from "./pages/Docs";
import { loadPagePlugins } from "./plugins";
import { listen } from "@tauri-apps/api/event";
import CustomPluginPage from "./pages/CustomPluginPage";
import Footer, { FooterMode } from "./components/launch/Footer";

export default function App() {
	const [footerData, setFooterData] = createSignal<FooterData>({
		selectedItem: undefined,
		mode: FooterMode.Instance,
		action: () => {},
	});

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
						footerData={footerData()}
						onSelectUser={setSelectedUser}
						selectedUser={selectedUser()}
					/>
				)}
			>
				<Route
					path="/"
					component={() => <LaunchPage setFooterData={setFooterData} />}
				/>
				<Route
					path="/instance_config/:instanceId"
					component={() => (
						<InstanceConfig
							mode={ConfigMode.Instance}
							creating={false}
							setFooterData={setFooterData}
						/>
					)}
				/>
				<Route
					path="/profile_config/:profileId"
					component={() => (
						<InstanceConfig
							mode={ConfigMode.Profile}
							creating={false}
							setFooterData={setFooterData}
						/>
					)}
				/>
				<Route
					path="/create_instance"
					component={() => (
						<InstanceConfig
							mode={ConfigMode.Instance}
							creating={true}
							setFooterData={setFooterData}
						/>
					)}
				/>
				<Route
					path="/create_profile"
					component={() => (
						<InstanceConfig
							mode={ConfigMode.Profile}
							creating={true}
							setFooterData={setFooterData}
						/>
					)}
				/>
				<Route
					path="/global_profile_config"
					component={() => (
						<InstanceConfig
							mode={ConfigMode.GlobalProfile}
							creating={false}
							setFooterData={setFooterData}
						/>
					)}
				/>
				<Route
					path="/packages/:page"
					component={() => <BrowsePackages setFooterData={setFooterData} />}
				/>
				<Route path="/packages/package/:id" component={() => <ViewPackage />} />
				<Route path="/plugins" component={() => <Plugins />} />
				<Route path="/docs" component={() => <Docs />} />
				<Route path="/custom/:page" component={() => <CustomPluginPage />} />
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
			{props.children}
			<Sidebar
				visible={showSidebar()}
				location={props.location}
				setVisible={setShowSidebar}
			/>
			<Footer
				selectedItem={props.footerData.selectedItem}
				mode={props.footerData.mode}
				selectedUser={props.selectedUser}
				action={props.footerData.action}
			/>
		</>
	);
}

interface LayoutProps {
	children: any;
	location: Location;
	footerData: FooterData;
	selectedUser?: string;
	onSelectUser: (user: string) => void;
}

export interface FooterData {
	selectedItem?: string;
	mode: FooterMode;
	action: () => void;
}
