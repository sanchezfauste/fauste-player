#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fp_app::i18n::{I18n, message_ids};

#[test]
fn both_locales_define_the_same_keys() {
    let en = message_ids("en-US");
    let es = message_ids("es-ES");
    assert!(!en.is_empty());
    let missing: Vec<_> = en.difference(&es).collect();
    let extra: Vec<_> = es.difference(&en).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "missing in es-ES: {missing:?}; unknown in es-ES: {extra:?}"
    );
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
