use serde::{Deserialize, Serialize};

/// A language supported by mcvm. Includes all languages in Minecraft.
#[derive(Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Language {
	Afrikaans,
	Arabic,
	Asturian,
	Azerbaijani,
	Bashkir,
	Bavarian,
	Belarusian,
	Bulgarian,
	Breton,
	Brabantian,
	Bosnian,
	Catalan,
	Czech,
	Welsh,
	Danish,
	AustrianGerman,
	SwissGerman,
	German,
	Greek,
	AustralianEnglish,
	CanadianEnglish,
	BritishEnglish,
	NewZealandEnglish,
	PirateSpeak,
	UpsideDown,
	AmericanEnglish,
	Anglish,
	Shakespearean,
	Esperanto,
	ArgentinianSpanish,
	ChileanSpanish,
	EcuadorianSpanish,
	EuropeanSpanish,
	MexicanSpanish,
	UruguayanSpanish,
	VenezuelanSpanish,
	Andalusian,
	Estonian,
	Basque,
	Persian,
	Finnish,
	Filipino,
	Faroese,
	CanadianFrench,
	EuropeanFrench,
	EastFranconian,
	Friulian,
	Frisian,
	Irish,
	ScottishGaelic,
	Galician,
	Hawaiian,
	Hebrew,
	Hindi,
	Croatian,
	Hungarian,
	Armenian,
	Indonesian,
	Igbo,
	Ido,
	Icelandic,
	Interslavic,
	Italian,
	Japanese,
	Lojban,
	Georgian,
	Kazakh,
	Kannada,
	Korean,
	Kolsch,
	Cornish,
	Latin,
	Luxembourgish,
	Limburgish,
	Lombard,
	Lolcat,
	Lithuanian,
	Latvian,
	ClassicalChinese,
	Macedonian,
	Mongolian,
	Malay,
	Maltese,
	Nahuatl,
	LowGerman,
	DutchFlemish,
	Dutch,
	NorwegianNynorsk,
	NorwegianBokmal,
	Occitan,
	Elfdalian,
	Polish,
	BrazilianPortuguese,
	EuropeanPortuguese,
	Quenya,
	Romanian,
	RussianPreRevolutionary,
	Russian,
	Rusyn,
	NorthernSami,
	Slovak,
	Slovenian,
	Somali,
	Albanian,
	Serbian,
	Swedish,
	UpperSaxonGerman,
	Silesian,
	Tamil,
	Thai,
	Tagalog,
	Klingon,
	TokiPona,
	Turkish,
	Tatar,
	Ukrainian,
	Valencian,
	Venetian,
	Vietnamese,
	Yiddish,
	Yoruba,
	ChineseSimplified,
	ChineseTraditionalHongKong,
	ChineseTraditionalTaiwan,
	MalayJawi,
}

impl Default for Language {
	fn default() -> Self {
		match current_locale::current_locale() {
			Ok(locale) => extract_locale_language(&locale).unwrap_or(Self::AmericanEnglish),
			Err(..) => Self::AmericanEnglish,
		}
	}
}

impl Language {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"afrikaans" => Some(Self::Afrikaans),
			"arabic" => Some(Self::Arabic),
			"asturian" => Some(Self::Asturian),
			"azerbaijani" => Some(Self::Azerbaijani),
			"bashkir" => Some(Self::Bashkir),
			"bavarian" => Some(Self::Bavarian),
			"belarusian" => Some(Self::Belarusian),
			"bulgarian" => Some(Self::Bulgarian),
			"breton" => Some(Self::Breton),
			"brabantian" => Some(Self::Brabantian),
			"bosnian" => Some(Self::Bosnian),
			"catalan" => Some(Self::Catalan),
			"czech" => Some(Self::Czech),
			"welsh" => Some(Self::Welsh),
			"danish" => Some(Self::Danish),
			"austrian_german" => Some(Self::AustrianGerman),
			"swiss_german" => Some(Self::SwissGerman),
			"german" => Some(Self::German),
			"greek" => Some(Self::Greek),
			"australian_english" => Some(Self::AustralianEnglish),
			"canadian_english" => Some(Self::CanadianEnglish),
			"british_english" => Some(Self::BritishEnglish),
			"new_zealand_english" => Some(Self::NewZealandEnglish),
			"pirate_speak" => Some(Self::PirateSpeak),
			"upside_down" => Some(Self::UpsideDown),
			"american_english" => Some(Self::AmericanEnglish),
			"anglish" => Some(Self::Anglish),
			"shakespearean" => Some(Self::Shakespearean),
			"esperanto" => Some(Self::Esperanto),
			"argentinian_spanish" => Some(Self::ArgentinianSpanish),
			"chilean_spanish" => Some(Self::ChileanSpanish),
			"ecuadorian_spanish" => Some(Self::EcuadorianSpanish),
			"european_spanish" => Some(Self::EuropeanSpanish),
			"mexican_spanish" => Some(Self::MexicanSpanish),
			"uruguayan_spanish" => Some(Self::UruguayanSpanish),
			"venezuelan_spanish" => Some(Self::VenezuelanSpanish),
			"andalusian" => Some(Self::Andalusian),
			"estonian" => Some(Self::Estonian),
			"basque" => Some(Self::Basque),
			"persian" => Some(Self::Persian),
			"finnish" => Some(Self::Finnish),
			"filipino" => Some(Self::Filipino),
			"faroese" => Some(Self::Faroese),
			"canadian_french" => Some(Self::CanadianFrench),
			"european_french" => Some(Self::EuropeanFrench),
			"east_franconian" => Some(Self::EastFranconian),
			"friulian" => Some(Self::Friulian),
			"frisian" => Some(Self::Frisian),
			"irish" => Some(Self::Irish),
			"scottish_gaelic" => Some(Self::ScottishGaelic),
			"galician" => Some(Self::Galician),
			"hawaiian" => Some(Self::Hawaiian),
			"hebrew" => Some(Self::Hebrew),
			"hindi" => Some(Self::Hindi),
			"croatian" => Some(Self::Croatian),
			"hungarian" => Some(Self::Hungarian),
			"armenian" => Some(Self::Armenian),
			"indonesian" => Some(Self::Indonesian),
			"igbo" => Some(Self::Igbo),
			"ido" => Some(Self::Ido),
			"icelandic" => Some(Self::Icelandic),
			"interslavic" => Some(Self::Interslavic),
			"italian" => Some(Self::Italian),
			"japanese" => Some(Self::Japanese),
			"lojban" => Some(Self::Lojban),
			"georgian" => Some(Self::Georgian),
			"kazakh" => Some(Self::Kazakh),
			"kannada" => Some(Self::Kannada),
			"korean" => Some(Self::Korean),
			"kolsch" => Some(Self::Kolsch),
			"cornish" => Some(Self::Cornish),
			"latin" => Some(Self::Latin),
			"luxembourgish" => Some(Self::Luxembourgish),
			"limburgish" => Some(Self::Limburgish),
			"lombard" => Some(Self::Lombard),
			"lolcat" => Some(Self::Lolcat),
			"lithuanian" => Some(Self::Lithuanian),
			"latvian" => Some(Self::Latvian),
			"classical_chinese" => Some(Self::ClassicalChinese),
			"macedonian" => Some(Self::Macedonian),
			"mongolian" => Some(Self::Mongolian),
			"malay" => Some(Self::Malay),
			"maltese" => Some(Self::Maltese),
			"nahuatl" => Some(Self::Nahuatl),
			"low_german" => Some(Self::LowGerman),
			"dutch_flemish" => Some(Self::DutchFlemish),
			"dutch" => Some(Self::Dutch),
			"norwegian_nynorsk" => Some(Self::NorwegianNynorsk),
			"norwegian_bokmal" => Some(Self::NorwegianBokmal),
			"occitan" => Some(Self::Occitan),
			"elfdalian" => Some(Self::Elfdalian),
			"polish" => Some(Self::Polish),
			"brazilian_portuguese" => Some(Self::BrazilianPortuguese),
			"european_portuguese" => Some(Self::EuropeanPortuguese),
			"quenya" => Some(Self::Quenya),
			"romanian" => Some(Self::Romanian),
			"russian_pre_revolutionary" => Some(Self::RussianPreRevolutionary),
			"russian" => Some(Self::Russian),
			"rusyn" => Some(Self::Rusyn),
			"northern_sami" => Some(Self::NorthernSami),
			"slovak" => Some(Self::Slovak),
			"slovenian" => Some(Self::Slovenian),
			"somali" => Some(Self::Somali),
			"albanian" => Some(Self::Albanian),
			"serbian" => Some(Self::Serbian),
			"swedish" => Some(Self::Swedish),
			"upper_saxon_german" => Some(Self::UpperSaxonGerman),
			"silesian" => Some(Self::Silesian),
			"tamil" => Some(Self::Tamil),
			"thai" => Some(Self::Thai),
			"tagalog" => Some(Self::Tagalog),
			"klingon" => Some(Self::Klingon),
			"toki_pona" => Some(Self::TokiPona),
			"turkish" => Some(Self::Turkish),
			"tatar" => Some(Self::Tatar),
			"ukrainian" => Some(Self::Ukrainian),
			"valencian" => Some(Self::Valencian),
			"venetian" => Some(Self::Venetian),
			"vietnamese" => Some(Self::Vietnamese),
			"yiddish" => Some(Self::Yiddish),
			"yoruba" => Some(Self::Yoruba),
			"chinese_simplified" => Some(Self::ChineseSimplified),
			"chinese_traditional_hong_kong" => Some(Self::ChineseTraditionalHongKong),
			"chinese_traditional_taiwan" => Some(Self::ChineseTraditionalTaiwan),
			"malay_jawi" => Some(Self::MalayJawi),
			_ => None,
		}
	}
}

/// Extract a `Language` value from a locale. Not all locales and languages are supported
pub fn extract_locale_language(locale: &str) -> Option<Language> {
	let stripped = strip_locale(locale);
	match stripped {
		"en-US" | "C" => Some(Language::AmericanEnglish),
		_ => None,
	}
}

/// Strip extensions and other stuff from an IETF language tag
pub fn strip_locale(locale: &str) -> &str {
	locale
		.split_at(locale.find('.').unwrap_or(locale.len() - 1))
		.0
}
