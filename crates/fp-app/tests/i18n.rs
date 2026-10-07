#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The locale registry, language negotiation and the parity of every
//! locale file with the en-US source.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use fp_app::i18n::{I18n, LOCALES, Locale, Translation, locale, negotiate};

/// The variables a pattern uses, selectors included.
fn pattern_vars(pattern: &Pattern<&str>, out: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_vars(expression, out);
        }
    }
}

fn expression_vars(expression: &Expression<&str>, out: &mut BTreeSet<String>) {
    match expression {
        Expression::Select { selector, variants } => {
            inline_vars(selector, out);
            for v in variants {
                pattern_vars(&v.value, out);
            }
        }
        Expression::Inline(inline) => inline_vars(inline, out),
    }
}

fn inline_vars(inline: &InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match inline {
        InlineExpression::VariableReference { id } => {
            out.insert(format!("${}", id.name));
        }
        InlineExpression::MessageReference { id, .. } => {
            out.insert(id.name.to_owned());
        }
        InlineExpression::TermReference { id, arguments, .. } => {
            out.insert(format!("-{}", id.name));
            for a in arguments.iter().flat_map(|a| &a.positional) {
                inline_vars(a, out);
            }
        }
        InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, out);
            }
            for a in &arguments.named {
                inline_vars(&a.value, out);
            }
        }
        InlineExpression::Placeable { expression } => expression_vars(expression, out),
        InlineExpression::StringLiteral { .. } | InlineExpression::NumberLiteral { .. } => {}
    }
}

/// Every message of a source with the variables (and references) it uses.
/// A source that does not parse cleanly fails.
fn messages(name: &str, source: &str) -> BTreeMap<String, BTreeSet<String>> {
    let resource = fluent_syntax::parser::parse(source)
        .unwrap_or_else(|(_, errors)| panic!("{name} does not parse: {errors:?}"));
    let mut out = BTreeMap::new();
    for entry in &resource.body {
        match entry {
            Entry::Message(m) => {
                let mut vars = BTreeSet::new();
                if let Some(value) = &m.value {
                    pattern_vars(value, &mut vars);
                }
                for a in &m.attributes {
                    pattern_vars(&a.value, &mut vars);
                }
                out.insert(m.id.name.to_owned(), vars);
            }
            Entry::Junk { content } => panic!("{name} has junk: {content:?}"),
            _ => {}
        }
    }
    out
}

/// What differs between a translation and the source: missing and unknown
/// ids, and messages whose variables differ.
fn parity(source: &str, name: &str, translation: &str) -> Vec<String> {
    let en = messages("en-US", source);
    let other = messages(name, translation);
    let mut problems = Vec::new();
    for id in en.keys().filter(|id| !other.contains_key(*id)) {
        problems.push(format!("{name}: missing {id}"));
    }
    for id in other.keys().filter(|id| !en.contains_key(*id)) {
        problems.push(format!("{name}: unknown {id}"));
    }
    for (id, vars) in &en {
        if let Some(theirs) = other.get(id)
            && theirs != vars
        {
            problems.push(format!("{name}: {id} uses {theirs:?}, en-US uses {vars:?}"));
        }
    }
    problems
}

fn english() -> &'static Locale {
    locale("en-US").expect("en-US is registered")
}

#[test]
fn every_locale_defines_exactly_the_english_messages_and_variables() {
    let en = english();
    assert!(!messages("en-US", en.source).is_empty());
    let problems: Vec<String> = LOCALES
        .iter()
        .flat_map(|l| parity(en.source, l.tag, l.source))
        .collect();
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn the_parity_check_catches_a_broken_placeable() {
    let en = "a = { $count } tracks\nb = Hi { $name }\n";
    assert!(parity(en, "xx", "a = { $count } pistas\nb = Hola { $name }\n").is_empty());
    // A misspelt variable.
    assert!(!parity(en, "xx", "a = { $cuont } pistas\nb = Hola { $name }\n").is_empty());
    // A variable turned into a message reference.
    assert!(!parity(en, "xx", "a = { count } pistas\nb = Hola { $name }\n").is_empty());
    // A missing id.
    assert!(!parity(en, "xx", "a = { $count } pistas\n").is_empty());
}

#[test]
#[should_panic(expected = "does not parse")]
fn the_parity_check_catches_an_unclosed_placeable() {
    parity("a = { $count } tracks\n", "xx", "a = { $count pistas\n");
}

#[test]
fn every_locale_file_is_registered_and_every_registered_locale_has_one() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
    let on_disk: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().join("main.ftl").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let registered: BTreeSet<String> = LOCALES.iter().map(|l| l.tag.to_owned()).collect();
    assert_eq!(on_disk, registered);
    for l in LOCALES {
        let file = std::fs::read_to_string(dir.join(l.tag).join("main.ftl")).unwrap();
        assert_eq!(file, l.source, "{} embeds its own file", l.tag);
    }
}

#[test]
fn the_registry_lists_english_first_then_by_own_name() {
    assert_eq!(LOCALES.first().map(|l| l.tag), Some("en-US"));
    let names: Vec<&str> = LOCALES.iter().skip(1).map(|l| l.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
    assert_eq!(locale("en-US").map(|l| l.name), Some("English"));
    assert_eq!(locale("es-ES").map(|l| l.name), Some("Español"));
}

#[test]
fn only_english_and_spanish_are_written_by_hand() {
    for l in LOCALES {
        let by_hand = matches!(l.tag, "en-US" | "es-ES");
        assert_eq!(l.translation == Translation::Manual, by_hand, "{}", l.tag);
    }
    assert!(!I18n::new(Some("es-ES")).machine_translated());
    assert!(!I18n::new(Some("en-US")).machine_translated());
}

#[test]
fn negotiation_matches_the_full_tag_then_the_language() {
    let tags = [
        "en-US", "es-ES", "fr-FR", "de-DE", "pt-PT", "pt-BR", "ca-ES", "eu-ES", "gl-ES",
    ];
    let pick = |r: &str| negotiate(r, tags.iter().copied());
    assert_eq!(pick("pt-BR"), Some("pt-BR"));
    assert_eq!(pick("pt-AO"), Some("pt-PT"));
    assert_eq!(pick("fr-CA"), Some("fr-FR"));
    assert_eq!(pick("de-AT"), Some("de-DE"));
    assert_eq!(pick("ca-AD"), Some("ca-ES"));
    assert_eq!(pick("eu"), Some("eu-ES"));
    assert_eq!(pick("gl"), Some("gl-ES"));
    assert_eq!(pick("es_AR.UTF-8"), Some("es-ES"));
    assert_eq!(pick("ES-es"), Some("es-ES"));
    assert_eq!(pick("ja-JP"), None);
    assert_eq!(pick(""), None);
}

#[test]
fn spanish_is_used_when_requested() {
    assert_eq!(I18n::new(Some("es-ES")).tr("status-on-air"), "Sonando");
    assert_eq!(I18n::new(Some("es")).tr("next-caption"), "SIG.");
    assert_eq!(I18n::new(Some("es-AR")).lang(), "es-ES");
}

#[test]
fn unknown_locales_fall_back_to_english() {
    let i = I18n::new(Some("ja-JP"));
    assert_eq!(i.lang(), "en-US");
    assert_eq!(i.tr("status-on-air"), "On air");
}

#[test]
fn missing_keys_fall_back_per_key() {
    assert_eq!(I18n::new(Some("es-ES")).tr("no-such-key"), "no-such-key");
}

/// The plural categories of every target language come from CLDR through
/// fluent-bundle: Polish has `few` and `many`, the others `one`/`other`.
#[test]
fn plural_rules_work_for_every_target_language() {
    let ftl = "n = { $n ->\n    [one] one\n    [few] few\n    [many] many\n   *[other] other\n}\n";
    let pick = |tag: &str, n: u32| {
        let mut bundle = FluentBundle::new(vec![tag.parse().unwrap()]);
        bundle.set_use_isolating(false);
        bundle
            .add_resource(FluentResource::try_new(ftl.to_owned()).unwrap())
            .unwrap();
        let mut args = FluentArgs::new();
        args.set("n", n);
        let pattern = bundle.get_message("n").unwrap().value().unwrap();
        let mut errors = Vec::new();
        let out = bundle.format_pattern(pattern, Some(&args), &mut errors);
        assert!(errors.is_empty(), "{tag}: {errors:?}");
        out.into_owned()
    };
    assert_eq!(
        [1, 2, 3, 5, 12, 22, 25].map(|n| pick("pl-PL", n)),
        ["one", "few", "few", "many", "many", "few", "many"]
    );
    for tag in [
        "en-US", "es-ES", "fr-FR", "de-DE", "it-IT", "pt-PT", "nl-NL", "ca-ES", "eu-ES", "gl-ES",
    ] {
        assert_eq!(pick(tag, 1), "one", "{tag}");
        assert_eq!(pick(tag, 7), "other", "{tag}");
    }
}

/// Polish needs `one`, `few` and `many`: a real message picks each form.
#[test]
fn polish_track_counts_use_the_right_plural_form() {
    let pl = I18n::new(Some("pl-PL"));
    assert_eq!(pl.lang(), "pl-PL");
    let count = |n: u32| pl.tr_args("footer-count", &[("count", n.into())]);
    assert_eq!(
        [1, 2, 5, 22].map(count),
        ["1 utwór", "2 utwory", "5 utworów", "22 utwory"]
    );
}

#[test]
fn arguments_are_substituted() {
    let en = I18n::new(Some("en-US"));
    assert_eq!(
        en.tr_args("footer-count", &[("count", 3.into())]),
        "3 tracks"
    );
    assert_eq!(
        en.tr_args("footer-count", &[("count", 1.into())]),
        "1 track"
    );
    let es = I18n::new(Some("es-ES"));
    assert_eq!(
        es.tr_args("intro-badge", &[("secs", "11.4".into())]),
        "INTRO 11.4"
    );
}

#[test]
fn the_stop_all_label_carries_its_count_in_both_languages() {
    let en = I18n::new(Some("en-US"));
    assert_eq!(en.tr("cartwall-stop-all"), "Stop all");
    assert_eq!(
        en.tr_args("cartwall-stop-all-count", &[("count", 2.into())]),
        "Stop all (2)"
    );
    let es = I18n::new(Some("es-ES"));
    assert_eq!(es.tr("cartwall-stop-all"), "Parar todo");
    assert_eq!(
        es.tr_args("cartwall-stop-all-count", &[("count", 12.into())]),
        "Parar todo (12)"
    );
}

/// The en-US messages that select on a number, with the variable they
/// select on.
fn plural_messages() -> Vec<(String, String)> {
    let resource = fluent_syntax::parser::parse(english().source).unwrap();
    let mut out = Vec::new();
    for entry in &resource.body {
        if let Entry::Message(m) = entry
            && let Some(value) = &m.value
        {
            for element in &value.elements {
                if let PatternElement::Placeable {
                    expression:
                        Expression::Select {
                            selector: InlineExpression::VariableReference { id },
                            ..
                        },
                } = element
                {
                    out.push((m.id.name.to_owned(), id.name.to_owned()));
                }
            }
        }
    }
    out
}

/// Every plural variant shows the count itself: a literal "1" in a `one`
/// variant is wrong where 0 is also `one` (French), and a word such as
/// "One" hides the number.
#[test]
fn every_plural_variant_shows_the_count_it_was_given() {
    let plurals = plural_messages();
    for id in [
        "footer-count",
        "playlist-imported",
        "playlist-streams-skipped",
        "tags-others-kept",
        "tags-others-kept-more",
        "outdated-body",
    ] {
        assert!(
            plurals.iter().any(|(m, _)| m == id),
            "{id} selects on a number"
        );
    }
    let mut problems = Vec::new();
    for l in LOCALES {
        let i18n = I18n::new(Some(l.tag));
        assert_eq!(i18n.lang(), l.tag);
        for (id, var) in &plurals {
            for n in [0u32, 1, 2, 5] {
                let text = i18n.tr_args(
                    id,
                    &[
                        (var.as_str(), n.into()),
                        ("name", "list".into()),
                        ("error", "oops".into()),
                    ],
                );
                let numbers: Vec<&str> = text
                    .split(|c: char| !c.is_ascii_digit())
                    .filter(|s| !s.is_empty())
                    .collect();
                if numbers != [n.to_string()] {
                    problems.push(format!("{} {id} ({n}): {text}", l.tag));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
