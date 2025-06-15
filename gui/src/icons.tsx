import PinIcon from "./assets/icons/pin.svg?component-solid";
import BoxIcon from "./assets/icons/box.svg?component-solid";
import FolderIcon from "./assets/icons/folder.svg?component-solid";
import PlayIcon from "./assets/icons/play.svg?component-solid";
import PropertiesIcon from "./assets/icons/properties.svg?component-solid";
import CopyIcon from "./assets/icons/copy.svg?component-solid";
import CheckIcon from "./assets/icons/check.svg?component-solid";
import GlobeIcon from "./assets/icons/globe.svg?component-solid";
import CrossIcon from "./assets/icons/cross.svg?component-solid";
import SpinnerIcon from "./assets/icons/spinner.svg?component-solid";
import AngleLeftIcon from "./assets/icons/angle_left.svg?component-solid";
import AngleRightIcon from "./assets/icons/angle_right.svg?component-solid";
import PlusIcon from "./assets/icons/plus.svg?component-solid";
import SearchIcon from "./assets/icons/search.svg?component-solid";
import DeleteIcon from "./assets/icons/delete.svg?component-solid";
import EditIcon from "./assets/icons/edit.svg?component-solid";
import JigsawIcon from "./assets/icons/jigsaw.svg?component-solid";
import MenuIcon from "./assets/icons/menu.svg?component-solid";
import HomeIcon from "./assets/icons/home.svg?component-solid";
import LogoIcon from "./assets/icons/logo.svg?component-solid";
import RefreshIcon from "./assets/icons/refresh.svg?component-solid";
import UploadIcon from "./assets/icons/upload.svg?component-solid";
import DownloadIcon from "./assets/icons/download.svg?component-solid";
import WarningIcon from "./assets/icons/warning.svg?component-solid";
import ErrorIcon from "./assets/icons/error.svg?component-solid";
import ScrollIcon from "./assets/icons/scroll.svg?component-solid";
import KeyIcon from "./assets/icons/key.svg?component-solid";
import CurlyBracesIcon from "./assets/icons/curly_braces.svg?component-solid";
import UserIcon from "./assets/icons/user.svg?component-solid";
import HeartIcon from "./assets/icons/heart.svg?component-solid";
import BookIcon from "./assets/icons/book.svg?component-solid";
import AudioIcon from "./assets/icons/audio.svg?component-solid";
import FullscreenIcon from "./assets/icons/fullscreen.svg?component-solid";
import GearIcon from "./assets/icons/gear.svg?component-solid";
import GraphIcon from "./assets/icons/graph.svg?component-solid";
import LanguageIcon from "./assets/icons/language.svg?component-solid";
import LinkIcon from "./assets/icons/link.svg?component-solid";
import MapPinIcon from "./assets/icons/map_pin.svg?component-solid";
import MicrophoneIcon from "./assets/icons/microphone.svg?component-solid";
import MinecraftIcon from "./assets/icons/minecraft.svg?component-solid";
import MoonIcon from "./assets/icons/moon.svg?component-solid";
import PaletteIcon from "./assets/icons/palette.svg?component-solid";
import PictureIcon from "./assets/icons/picture.svg?component-solid";
import StarIcon from "./assets/icons/star.svg?component-solid";
import SunIcon from "./assets/icons/sun.svg?component-solid";
import TextIcon from "./assets/icons/text.svg?component-solid";
import WindowIcon from "./assets/icons/window.svg?component-solid";
import { HasWidthHeight } from "./components/Icon";

export function Pin({ width, height, viewBox }: HasWidthHeight) {
	return <PinIcon width={width} height={height} viewBox={viewBox} />;
}

export function Box({ width, height, viewBox }: HasWidthHeight) {
	return <BoxIcon width={width} height={height} viewBox={viewBox} />;
}

export function Folder({ width, height, viewBox }: HasWidthHeight) {
	return <FolderIcon width={width} height={height} viewBox={viewBox} />;
}

export function Play({ width, height, viewBox }: HasWidthHeight) {
	return <PlayIcon width={width} height={height} viewBox={viewBox} />;
}

export function Properties({ width, height, viewBox }: HasWidthHeight) {
	return <PropertiesIcon width={width} height={height} viewBox={viewBox} />;
}

export function Copy({ width, height, viewBox }: HasWidthHeight) {
	return <CopyIcon width={width} height={height} viewBox={viewBox} />;
}

export function Check({ width, height, viewBox }: HasWidthHeight) {
	return <CheckIcon width={width} height={height} viewBox={viewBox} />;
}

export function Globe({ width, height, viewBox }: HasWidthHeight) {
	return <GlobeIcon width={width} height={height} viewBox={viewBox} />;
}

export function Cross({ width, height, viewBox }: HasWidthHeight) {
	return <CrossIcon width={width} height={height} viewBox={viewBox} />;
}

export function Spinner({ width, height, viewBox }: HasWidthHeight) {
	return <SpinnerIcon width={width} height={height} viewBox={viewBox} />;
}

export function AnimatedSpinner({ width, height, viewBox }: HasWidthHeight) {
	return (
		<div class="rotating">
			<SpinnerIcon width={width} height={height} viewBox={viewBox} />
		</div>
	);
}

export function AngleLeft({ width, height, viewBox }: HasWidthHeight) {
	return <AngleLeftIcon width={width} height={height} viewBox={viewBox} />;
}

export function AngleRight({ width, height, viewBox }: HasWidthHeight) {
	return <AngleRightIcon width={width} height={height} viewBox={viewBox} />;
}

export function Plus({ width, height, viewBox }: HasWidthHeight) {
	return <PlusIcon width={width} height={height} viewBox={viewBox} />;
}

export function Search({ width, height, viewBox }: HasWidthHeight) {
	return <SearchIcon width={width} height={height} viewBox={viewBox} />;
}

export function Delete({ width, height, viewBox }: HasWidthHeight) {
	return <DeleteIcon width={width} height={height} viewBox={viewBox} />;
}

export function Edit({ width, height, viewBox }: HasWidthHeight) {
	return <EditIcon width={width} height={height} viewBox={viewBox} />;
}

export function Jigsaw({ width, height, viewBox }: HasWidthHeight) {
	return <JigsawIcon width={width} height={height} viewBox={viewBox} />;
}

export function Menu({ width, height, viewBox }: HasWidthHeight) {
	return <MenuIcon width={width} height={height} viewBox={viewBox} />;
}

export function Home({ width, height, viewBox }: HasWidthHeight) {
	return <HomeIcon width={width} height={height} viewBox={viewBox} />;
}

export function Logo({ width, height, viewBox }: HasWidthHeight) {
	return <LogoIcon width={width} height={height} viewBox={viewBox} />;
}

export function Refresh({ width, height, viewBox }: HasWidthHeight) {
	return <RefreshIcon width={width} height={height} viewBox={viewBox} />;
}

export function Upload({ width, height, viewBox }: HasWidthHeight) {
	return <UploadIcon width={width} height={height} viewBox={viewBox} />;
}

export function Download({ width, height, viewBox }: HasWidthHeight) {
	return <DownloadIcon width={width} height={height} viewBox={viewBox} />;
}

export function Warning({ width, height, viewBox }: HasWidthHeight) {
	return <WarningIcon width={width} height={height} viewBox={viewBox} />;
}

export function Error({ width, height, viewBox }: HasWidthHeight) {
	return <ErrorIcon width={width} height={height} viewBox={viewBox} />;
}

export function Scroll({ width, height, viewBox }: HasWidthHeight) {
	return <ScrollIcon width={width} height={height} viewBox={viewBox} />;
}

export function Key({ width, height, viewBox }: HasWidthHeight) {
	return <KeyIcon width={width} height={height} viewBox={viewBox} />;
}

export function CurlyBraces({ width, height, viewBox }: HasWidthHeight) {
	return <CurlyBracesIcon width={width} height={height} viewBox={viewBox} />;
}

export function User({ width, height, viewBox }: HasWidthHeight) {
	return <UserIcon width={width} height={height} viewBox={viewBox} />;
}

export function Heart({ width, height, viewBox }: HasWidthHeight) {
	return <HeartIcon width={width} height={height} viewBox={viewBox} />;
}

export function Book({ width, height, viewBox }: HasWidthHeight) {
	return <BookIcon width={width} height={height} viewBox={viewBox} />;
}

export function Audio({ width, height, viewBox }: HasWidthHeight) {
	return <AudioIcon width={width} height={height} viewBox={viewBox} />;
}
export function Fullscreen({ width, height, viewBox }: HasWidthHeight) {
	return <FullscreenIcon width={width} height={height} viewBox={viewBox} />;
}
export function Gear({ width, height, viewBox }: HasWidthHeight) {
	return <GearIcon width={width} height={height} viewBox={viewBox} />;
}
export function Graph({ width, height, viewBox }: HasWidthHeight) {
	return <GraphIcon width={width} height={height} viewBox={viewBox} />;
}
export function Language({ width, height, viewBox }: HasWidthHeight) {
	return <LanguageIcon width={width} height={height} viewBox={viewBox} />;
}
export function Link({ width, height, viewBox }: HasWidthHeight) {
	return <LinkIcon width={width} height={height} viewBox={viewBox} />;
}
export function MapPin({ width, height, viewBox }: HasWidthHeight) {
	return <MapPinIcon width={width} height={height} viewBox={viewBox} />;
}
export function Microphone({ width, height, viewBox }: HasWidthHeight) {
	return <MicrophoneIcon width={width} height={height} viewBox={viewBox} />;
}
export function Minecraft({ width, height, viewBox }: HasWidthHeight) {
	return <MinecraftIcon width={width} height={height} viewBox={viewBox} />;
}
export function Moon({ width, height, viewBox }: HasWidthHeight) {
	return <MoonIcon width={width} height={height} viewBox={viewBox} />;
}
export function Palette({ width, height, viewBox }: HasWidthHeight) {
	return <PaletteIcon width={width} height={height} viewBox={viewBox} />;
}
export function Picture({ width, height, viewBox }: HasWidthHeight) {
	return <PictureIcon width={width} height={height} viewBox={viewBox} />;
}
export function Star({ width, height, viewBox }: HasWidthHeight) {
	return <StarIcon width={width} height={height} viewBox={viewBox} />;
}
export function Sun({ width, height, viewBox }: HasWidthHeight) {
	return <SunIcon width={width} height={height} viewBox={viewBox} />;
}
export function Text({ width, height, viewBox }: HasWidthHeight) {
	return <TextIcon width={width} height={height} viewBox={viewBox} />;
}
export function Window({ width, height, viewBox }: HasWidthHeight) {
	return <WindowIcon width={width} height={height} viewBox={viewBox} />;
}
