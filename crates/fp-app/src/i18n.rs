//! UI strings through Fluent (spec §1 language rule). `en-US` is the source
//! of truth; other locales fall back to it key by key.
//!
//! Adding a locale is one file, `locales/<tag>/main.ftl`, and one entry in
//! [`LOCALES`]; the tests check that both exist and that the file has
//! exactly the en-US messages with the same variables.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

/// How a locale's strings were written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Translation {
    /// Written and reviewed by hand (en-US, es-ES).
    Manual,
    /// Generated with AI: the About window says it may contain errors.
    Machine,
}

/// A language the interface is available in.
#[derive(Debug)]
pub struct Locale {
    /// The BCP-47 tag, also the folder under `locales/`.
    pub tag: &'static str,
    /// The language's name in that language, as the selector shows it.
    pub name: &'static str,
    pub translation: Translation,
    /// The embedded `main.ftl`.
    pub source: &'static str,
}

/// The source language, and the fallback of every other one, key by key.
const ENGLISH: Locale = Locale {
    tag: "en-US",
    name: "English",
    translation: Translation::Manual,
    source: include_str!("../locales/en-US/main.ftl"),
};

/// Every embedded locale: en-US first, then by own name (the order of the
/// language selector).
pub const LOCALES: &[Locale] = &[
    ENGLISH,
    Locale {
        tag: "ca-ES",
        name: "Català",
        translation: Translation::Machine,
        source: include_str!("../locales/ca-ES/main.ftl"),
    },
    Locale {
        tag: "de-DE",
        name: "Deutsch",
        translation: Translation::Machine,
        source: include_str!("../locales/de-DE/main.ftl"),
    },
    Locale {
        tag: "es-ES",
        name: "Español",
        translation: Translation::Manual,
        source: include_str!("../locales/es-ES/main.ftl"),
    },
    Locale {
        tag: "eu-ES",
        name: "Euskara",
        translation: Translation::Machine,
        source: include_str!("../locales/eu-ES/main.ftl"),
    },
    Locale {
        tag: "fr-FR",
        name: "Français",
        translation: Translation::Machine,
        source: include_str!("../locales/fr-FR/main.ftl"),
    },
    Locale {
        tag: "gl-ES",
        name: "Galego",
        translation: Translation::Machine,
        source: include_str!("../locales/gl-ES/main.ftl"),
    },
    Locale {
        tag: "it-IT",
        name: "Italiano",
        translation: Translation::Machine,
        source: include_str!("../locales/it-IT/main.ftl"),
    },
    Locale {
        tag: "nl-NL",
        name: "Nederlands",
        translation: Translation::Machine,
        source: include_str!("../locales/nl-NL/main.ftl"),
    },
    Locale {
        tag: "pl-PL",
        name: "Polski",
        translation: Translation::Machine,
        source: include_str!("../locales/pl-PL/main.ftl"),
    },
    Locale {
        tag: "pt-PT",
        name: "Português",
        translation: Translation::Machine,
        source: include_str!("../locales/pt-PT/main.ftl"),
    },
];

/// A registered locale by its exact tag.
pub fn locale(tag: &str) -> Option<&'static Locale> {
    LOCALES.iter().find(|l| l.tag == tag)
}

/// The best of `available` for a requested tag (BCP-47, or POSIX such as
/// `es_AR.UTF-8`): the same tag, else the first with the same language.
pub fn negotiate<'a>(
    requested: &str,
    available: impl IntoIterator<Item = &'a str> + Clone,
) -> Option<&'a str> {
    let tag = requested
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('_', "-");
    if tag.is_empty() {
        return None;
    }
    let language = |t: &str| t.split('-').next().unwrap_or_default().to_ascii_lowercase();
    let wanted = language(&tag);
    let mut all = available.clone().into_iter();
    all.find(|a| a.eq_ignore_ascii_case(&tag))
        .or_else(|| available.into_iter().find(|a| language(a) == wanted))
}

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

fn bundle(locale: &Locale) -> Option<FluentBundle<FluentResource>> {
    bundle_from(locale.tag, locale.source)
}

/// A bundle for `tag` from a Fluent source, `None` when the tag does not
/// parse. Lenient: a syntax error loses only the entries it touches.
pub fn bundle_from(tag: &str, source: &str) -> Option<FluentBundle<FluentResource>> {
    let lang: LanguageIdentifier = tag.parse().ok()?;
    let resource = match FluentResource::try_new(source.to_owned()) {
        Ok(resource) => resource,
        Err((resource, errors)) => {
            // The tests keep the shipped files clean; a stray error loses
            // only the messages it touches.
            tracing::warn!(locale = tag, ?errors, "locale file has syntax errors");
            resource
        }
    };
    let mut bundle = FluentBundle::new(vec![lang]);
    // No Unicode isolation marks: the strings go straight to egui labels.
    bundle.set_use_isolating(false);
    if let Err(errors) = bundle.add_resource(resource) {
        tracing::warn!(locale = tag, ?errors, "locale file has duplicate messages");
    }
    Some(bundle)
}

/// The locale for a configured tag and the OS locale: the configured one
/// when it matches a registered locale, else the system one, else en-US.
pub fn resolve(requested: Option<&str>, system: Option<&str>) -> &'static Locale {
    let pick = |r: &str| negotiate(r, LOCALES.iter().map(|l| l.tag)).and_then(locale);
    requested
        .and_then(pick)
        .or_else(|| system.and_then(pick))
        .unwrap_or(&ENGLISH)
}

pub struct I18n {
    locale: &'static Locale,
    primary: Option<FluentBundle<FluentResource>>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl I18n {
    /// `requested` is the configured language; `None` follows the OS locale.
    pub fn new(requested: Option<&str>) -> Self {
        let system = sys_locale::get_locale();
        let locale = resolve(requested, system.as_deref());
        Self {
            locale,
            primary: bundle(locale),
            fallback: bundle(&ENGLISH),
        }
    }

    /// The tag of the locale in use.
    pub fn lang(&self) -> &'static str {
        self.locale.tag
    }

    /// Whether the strings in use were generated with AI.
    pub fn machine_translated(&self) -> bool {
        self.locale.translation == Translation::Machine
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
