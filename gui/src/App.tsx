import { Router, Route, Location } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";
import { createSignal } from "solid-js";
import LaunchFooter, {
	SelectedFooterItem,
} from "./components/launch/LaunchFooter";
import InstanceConfig, { ConfigMode } from "./pages/config/InstanceConfig";
import BrowsePackages from "./pages/package/BrowsePackages";
import ViewPackage from "./pages/package/ViewPackage";
import Sidebar from "./components/navigation/Sidebar";

export default function App() {
	const [selectedItem, setSelectedItem] = createSignal<
		SelectedFooterItem | undefined
	>(undefined);

	return (
		<Router
			root={({ children, location }) => (
				<Layout
					children={children}
					location={location}
					selectedItem={selectedItem()}
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
		</Router>
	);
}

function Layout(props: LayoutProps) {
	let [showSidebar, setShowSidebar] = createSignal(false);

	return (
		<>
			<NavBar
				onSidebarToggle={() => {
					setShowSidebar(!showSidebar());
				}}
			/>
			<Sidebar
				visible={showSidebar()}
				location={props.location}
				setVisible={setShowSidebar}
			/>
			{props.children}
			<LaunchFooter selectedItem={props.selectedItem} />
		</>
	);
}

interface LayoutProps {
	children: any;
	location: Location;
	selectedItem?: SelectedFooterItem;
}
