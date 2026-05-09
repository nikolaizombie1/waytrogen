use iced::Theme;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::default::Default;

#[derive(Clone)]
pub struct WaytrogenTheme(pub Theme);

impl Serialize for WaytrogenTheme {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let name = match &self.0 {
            Theme::Light => "Light",
            Theme::Dark | Theme::Custom(_) => "Dark",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::SolarizedLight => "SolarizedLight",
            Theme::SolarizedDark => "SolarizedDark",
            Theme::GruvboxLight => "GruvboxLight",
            Theme::GruvboxDark => "GruvboxDark",
            Theme::CatppuccinLatte => "CatppuccinLatte",
            Theme::CatppuccinFrappe => "CatppuccinFrappe",
            Theme::CatppuccinMacchiato => "CatppuccinMacchiato",
            Theme::CatppuccinMocha => "CatppuccinMocha",
            Theme::TokyoNight => "TokyoNight",
            Theme::TokyoNightStorm => "TokyoNightStorm",
            Theme::TokyoNightLight => "TokyoNightLight",
            Theme::KanagawaWave => "KanagawaWave",
            Theme::KanagawaDragon => "KanagawaDragon",
            Theme::KanagawaLotus => "KanagawaLotus",
            Theme::Moonfly => "Moonfly",
            Theme::Nightfly => "Nightfly",
            Theme::Oxocarbon => "Oxocarbon",
            Theme::Ferra => "Ferra",
        };
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for WaytrogenTheme {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let theme = match s.as_str() {
            "Light" => Theme::Light,
            "Dracula" => Theme::Dracula,
            "Nord" => Theme::Nord,
            "SolarizedLight" => Theme::SolarizedLight,
            "SolarizedDark" => Theme::SolarizedDark,
            "GruvboxLight" => Theme::GruvboxLight,
            "GruvboxDark" => Theme::GruvboxDark,
            "CatppuccinLatte" => Theme::CatppuccinLatte,
            "CatppuccinFrappe" => Theme::CatppuccinFrappe,
            "CatppuccinMacchiato" => Theme::CatppuccinMacchiato,
            "CatppuccinMocha" => Theme::CatppuccinMocha,
            "TokyoNight" => Theme::TokyoNight,
            "TokyoNightStorm" => Theme::TokyoNightStorm,
            "TokyoNightLight" => Theme::TokyoNightLight,
            "KanagawaWave" => Theme::KanagawaWave,
            "KanagawaDragon" => Theme::KanagawaDragon,
            "KanagawaLotus" => Theme::KanagawaLotus,
            "Moonfly" => Theme::Moonfly,
            "Nightfly" => Theme::Nightfly,
            "Oxocarbon" => Theme::Oxocarbon,
            "Ferra" => Theme::Ferra,
            _ => Theme::Dark, // fallback for unknown variants
        };
        Ok(WaytrogenTheme(theme))
    }
}

impl From<WaytrogenTheme> for Theme {
    fn from(t: WaytrogenTheme) -> Self {
        t.0
    }
}

impl From<Theme> for WaytrogenTheme {
    fn from(t: Theme) -> Self {
        WaytrogenTheme(t)
    }
}

impl Default for WaytrogenTheme {
    fn default() -> Self {
	WaytrogenTheme(Theme::Dark)
    }
}
