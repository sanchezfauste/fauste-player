//! UI strings through Fluent (spec §1 language rule). `en-US` is the source
//! of truth; other locales fall back to it key by key.

use std::collections::BTreeSet;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

const LOCALES: &[(&str, &str)] = &[
    ("en-US", include_str!("../locales/en-US/main.ftl")),
    ("es-ES", include_str!("../locales/es-ES/main.ftl")),
];
const FALLBACK: &str = "en-US";

/// A value for a Fluent placeholder.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Number(f64),
    Text(String),
}

macro_rules! number_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(v: $t) -> Self {
                Arg::Number(v as f64)
            }
        }
    )*};
}
number_arg!(i32, i64, u16, u32, u64, usize, f32, f64);

impl From<&str> for Arg {
    fn from(v: &str) -> Self {
        Arg::Text(v.to_owned())
    }
}

impl From<String> for Arg {
    fn from(v: String) -> Self {
        Arg::Text(v)
    }
}

fn source(locale: &str) -> Option<&'static str> {
    LOCALES
        .iter()
        .find(|(id, _)| *id == locale)
        .map(|(_, src)| *src)
}

fn bundle(locale: &str) -> Option<FluentBundle<FluentResource>> {
    let lang: LanguageIdentifier = locale.parse().ok()?;
    let resource = FluentResource::try_new(source(locale)?.to_owned()).ok()?;
    let mut bundle = FluentBundle::new(vec![lang]);
    // No Unicode isolation marks: the strings go straight to egui labels.
    bundle.set_use_isolating(false);
    bundle.add_resource(resource).ok()?;
    Some(bundle)
}

/// The best available locale for a BCP-47 tag, by language subtag.
fn negotiate(requested: Option<&str>) -> &'static str {
    let wanted = requested.map(|r| r.split(['-', '_']).next().unwrap_or(r).to_ascii_lowercase());
    wanted
        .and_then(|w| {
            LOCALES
                .iter()
                .find(|(id, _)| id.split('-').next() == Some(w.as_str()))
        })
        .map_or(FALLBACK, |(id, _)| *id)
}

pub struct I18n {
    lang: &'static str,
    primary: Option<FluentBundle<FluentResource>>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl I18n {
    /// `requested` is the configured language; `None` follows the OS locale.
    pub fn new(requested: Option<&str>) -> Self {
        let system = sys_locale::get_locale();
        let lang = negotiate(requested.or(system.as_deref()));
        Self {
            lang,
            primary: bundle(lang),
            fallback: bundle(FALLBACK),
        }
    }

    pub fn lang(&self) -> &'static str {
        self.lang
    }

    pub fn tr(&self, key: &str) -> String {
        self.tr_args(key, &[])
    }

    pub fn tr_args(&self, key: &str, args: &[(&str, Arg)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            match value {
                Arg::Number(n) => fluent_args.set(*name, FluentValue::from(*n)),
                Arg::Text(t) => fluent_args.set(*name, FluentValue::from(t.clone())),
            }
        }
        for bundle in [&self.primary, &self.fallback].into_iter().flatten() {
            if let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                return text.into_owned();
            }
        }
        key.to_owned()
    }
}

/// The message ids a locale defines (for the key-parity test): lines that
/// start at column 0 with `identifier =`.
pub fn message_ids(locale: &str) -> BTreeSet<String> {
    source(locale)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with(|c: char| c.is_ascii_alphabetic()))
        .filter_map(|l| l.split_once(" =").map(|(id, _)| id.trim().to_owned()))
        .collect()
}
