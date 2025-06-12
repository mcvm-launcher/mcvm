import "./Loaders.css";

// A list of loaders displayed for package previews and such
export default function Loaders(props: LoadersProps) {
	let fullNames = props.fullNames == undefined ? false : props.fullNames;

	let allLoaders = () => {
		let out: Loader[] = [];
		for (let loader of props.loaders) {
			out.concat(getLoaders(loader));
		}
		return out;
	};

	return <div class="cont loaders"></div>;
}

function LoaderEntry(props: LoaderEntryProps) {
	return <div class="cont loader-entry"></div>;
}

interface LoaderEntryProps {
	loader: Loader;
}

export interface LoadersProps {
	loaders: string[];
	fullNames?: boolean;
}

export enum Loader {
	Fabric = "fabric",
	Quilt = "quilt",
	Forge = "forge",
	NeoForge = "neoforge",
	Sponge = "sponge",
	SpongeForge = "spongeforge",
}

function getLoaders(modification: string) {
	if (modification == "fabriclike") {
		return [Loader.Fabric, Loader.Quilt];
	} else if (modification == "forgelike") {
		return [Loader.Forge, Loader.NeoForge, Loader.SpongeForge];
	}
	return [modification as Loader];
}

function getLoaderImage(loader: Loader) {}
