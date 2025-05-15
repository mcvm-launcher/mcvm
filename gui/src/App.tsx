import { Router, Route } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";
import { createSignal } from "solid-js";
import LaunchFooter from "./components/launch/LaunchFooter";
import InstanceConfig, { ConfigMode } from "./pages/config/InstanceConfig";
import BrowsePackages from "./pages/package/BrowsePackages";
import ViewPackage from "./pages/package/ViewPackage";

export default function App() {
	const [selectedInstance, setSelectedInstance] = createSignal<string | null>(
		null
	);

	return (
		<Router
			root={({ children }) => (
				<Layout children={children} selectedInstance={selectedInstance()} />
			)}
		>
			<Route
				path="/"
				component={() => <LaunchPage onSelectInstance={setSelectedInstance} />}
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
			<Route path="/packages/:page" component={() => <BrowsePackages />} />
			<Route path="/package/:id" component={() => <ViewPackage />} />
		</Router>
	);
}

function Layout(props: LayoutProps) {
	return (
		<>
			<NavBar />
			{props.children}
			<LaunchFooter selectedInstance={props.selectedInstance} />
		</>
	);
}

interface LayoutProps {
	selectedInstance: string | null;
	children: any;
}
