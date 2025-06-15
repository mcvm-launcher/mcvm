import {
	AngleRight,
	Audio,
	Book,
	Box,
	Folder,
	Fullscreen,
	Gear,
	Globe,
	Graph,
	Home,
	Jigsaw,
	Key,
	Language,
	Link,
	MapPin,
	Microphone,
	Minecraft,
	Moon,
	Palette,
	Picture,
	Plus,
	Properties,
	Search,
	Star,
	Sun,
	Text,
	User,
	Window,
} from "./icons";
import { beautifyString } from "./utils";

export enum PackageCategory {
	Adventure = "adventure",
	Atmosphere = "atmosphere",
	Audio = "audio",
	Blocks = "blocks",
	Building = "building",
	Cartoon = "cartoon",
	Challenge = "challenge",
	Combat = "combat",
	Compatability = "compatability",
	Decoration = "decoration",
	Economy = "economy",
	Entities = "entities",
	Equipment = "equipment",
	Exploration = "exploration",
	Extensive = "extensive",
	Fantasy = "fantasy",
	Fonts = "fonts",
	Food = "food",
	GameMechanics = "game_mechanics",
	Gui = "gui",
	Items = "items",
	Language = "language",
	Library = "library",
	Lightweight = "lightweight",
	Magic = "magic",
	Minigame = "minigame",
	Mobs = "mobs",
	Multiplayer = "multiplayer",
	Optimization = "optimization",
	Realistic = "realistic",
	Simplistic = "simplistic",
	Space = "space",
	Social = "social",
	Storage = "storage",
	Structures = "structures",
	Technology = "technology",
	Transportation = "transportation",
	Tweaks = "tweaks",
	Utility = "utility",
	VanillaPlus = "vanilla_plus",
	Worldgen = "worldgen",
}

export function packageCategoryDisplayName(category: PackageCategory) {
	switch (category) {
		case PackageCategory.Adventure:
			return "Adventure";
		case PackageCategory.Atmosphere:
			return "Atmosphere";
		case PackageCategory.Audio:
			return "Audio";
		case PackageCategory.Blocks:
			return "Blocks";
		case PackageCategory.Building:
			return "Building";
		case PackageCategory.Cartoon:
			return "Cartoon";
		case PackageCategory.Challenge:
			return "Challenge";
		case PackageCategory.Combat:
			return "Combat";
		case PackageCategory.Compatability:
			return "Compatability";
		case PackageCategory.Decoration:
			return "Decoration";
		case PackageCategory.Economy:
			return "Economy";
		case PackageCategory.Entities:
			return "Entities";
		case PackageCategory.Equipment:
			return "Equipment";
		case PackageCategory.Exploration:
			return "Exploration";
		case PackageCategory.Extensive:
			return "Extensive";
		case PackageCategory.Fantasy:
			return "Fantasy";
		case PackageCategory.Fonts:
			return "Fonts";
		case PackageCategory.Food:
			return "Food";
		case PackageCategory.GameMechanics:
			return "Game Mechanics";
		case PackageCategory.Gui:
			return "Gui";
		case PackageCategory.Items:
			return "Items";
		case PackageCategory.Language:
			return "Language";
		case PackageCategory.Library:
			return "Library";
		case PackageCategory.Lightweight:
			return "Lightweight";
		case PackageCategory.Magic:
			return "Magic";
		case PackageCategory.Minigame:
			return "Minigame";
		case PackageCategory.Mobs:
			return "Mobs";
		case PackageCategory.Multiplayer:
			return "Multiplayer";
		case PackageCategory.Optimization:
			return "Optimization";
		case PackageCategory.Realistic:
			return "Realistic";
		case PackageCategory.Simplistic:
			return "Simplistic";
		case PackageCategory.Space:
			return "Space";
		case PackageCategory.Social:
			return "Social";
		case PackageCategory.Storage:
			return "Storage";
		case PackageCategory.Structures:
			return "Structures";
		case PackageCategory.Technology:
			return "Technology";
		case PackageCategory.Transportation:
			return "Transportation";
		case PackageCategory.Tweaks:
			return "Tweaks";
		case PackageCategory.Utility:
			return "Utility";
		case PackageCategory.VanillaPlus:
			return "Vanilla+";
		case PackageCategory.Worldgen:
			return "Worldgen";
	}

	return beautifyString(category);
}

export function packageCategoryIcon(category: PackageCategory) {
	switch (category) {
		case PackageCategory.Adventure:
			return Minecraft;
		case PackageCategory.Atmosphere:
			return Sun;
		case PackageCategory.Audio:
			return Audio;
		case PackageCategory.Blocks:
			return Box;
		case PackageCategory.Building:
			return Box;
		case PackageCategory.Cartoon:
			return Palette;
		case PackageCategory.Challenge:
			return Star;
		case PackageCategory.Combat:
			return Key;
		case PackageCategory.Compatability:
			return Link;
		case PackageCategory.Decoration:
			return Sun;
		case PackageCategory.Economy:
			return Graph;
		case PackageCategory.Entities:
			return Box;
		case PackageCategory.Equipment:
			return Key;
		case PackageCategory.Exploration:
			return Search;
		case PackageCategory.Extensive:
			return Home;
		case PackageCategory.Fantasy:
			return Star;
		case PackageCategory.Fonts:
			return Text;
		case PackageCategory.Food:
			return Box;
		case PackageCategory.GameMechanics:
			return Gear;
		case PackageCategory.Gui:
			return Window;
		case PackageCategory.Items:
			return Box;
		case PackageCategory.Language:
			return Language;
		case PackageCategory.Library:
			return Book;
		case PackageCategory.Lightweight:
			return Fullscreen;
		case PackageCategory.Magic:
			return Star;
		case PackageCategory.Minigame:
			return Jigsaw;
		case PackageCategory.Mobs:
			return User;
		case PackageCategory.Multiplayer:
			return User;
		case PackageCategory.Optimization:
			return AngleRight;
		case PackageCategory.Realistic:
			return Picture;
		case PackageCategory.Simplistic:
			return Fullscreen;
		case PackageCategory.Space:
			return Moon;
		case PackageCategory.Social:
			return Microphone;
		case PackageCategory.Storage:
			return Folder;
		case PackageCategory.Structures:
			return MapPin;
		case PackageCategory.Technology:
			return Properties;
		case PackageCategory.Transportation:
			return AngleRight;
		case PackageCategory.Tweaks:
			return Properties;
		case PackageCategory.Utility:
			return Gear;
		case PackageCategory.VanillaPlus:
			return Plus;
		case PackageCategory.Worldgen:
			return Globe;
	}

	return Box;
}
