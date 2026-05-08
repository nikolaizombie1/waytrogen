use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use fluent_langneg::{LanguageIdentifier, convert_vec_str_to_langids_lossy, negotiate_languages};
use fluent_templates::langid;
use log::error;
use std::default::Default;
use std::sync::LazyLock;
use sys_locale::get_locale;

pub struct Translation {
    pub bundle: FluentBundle<FluentResource>,
}

pub static TRANSLATION: LazyLock<Translation> = LazyLock::new(Translation::default);

impl Default for Translation {
    fn default() -> Self {
        let system_locale =
            convert_vec_str_to_langids_lossy(&[get_locale().unwrap_or_else(|| "en".to_string())]);
        let available = convert_vec_str_to_langids_lossy(["en", "es"]);
        let default = "en-US".parse::<LanguageIdentifier>().unwrap_or_default();

        let supported = negotiate_languages(
            &system_locale,
            &available,
            Some(&default),
            fluent_langneg::NegotiationStrategy::Filtering,
        );

        let language_id = match supported.first() {
            Some(l) => (*l).clone(),
            None => "en".parse::<LanguageIdentifier>().unwrap(),
        };

        let ftl_file = match language_id.language.as_str() {
            "es" => {
                include_str!("../locales/es.ftl")
            }
            _ => {
                include_str!("../locales/en.ftl")
            }
        };
        let resource = FluentResource::try_new(ftl_file.to_string()).unwrap();
        let language_id = language_id
            .language
            .as_str()
            .parse()
            .unwrap_or(langid!("en"));
        let mut bundle = FluentBundle::new_concurrent(vec![language_id]);
        bundle.add_resource(resource).unwrap();
        Self { bundle }
    }
}

impl Translation {
    pub fn get_translation(&self, id: &str) -> String {
        let mut errors = vec![];
        self.bundle
            .get_message(id)
            .and_then(|m| m.value())
            .map_or_else(
                || {
                    let ret = id.to_string();
                    error!("Failed to find \"{ret}\" in bundle");
                    ret
                },
                |p| self.bundle.format_pattern(p, None, &mut errors).to_string(),
            )
    }
}
