//! Lenient reading of a hand-edited `config.json`. Every field is taken on its
//! own, so one bad value falls back to its default instead of discarding the
//! whole file. Numbers are coerced towards integer fields (rounded, negative
//! to zero, saturated) and range checks are left to `Config::validate`.

use fp_model::Config;
use serde_json::Value;

/// Builds a `Config` from the user's JSON object, keeping every valid field.
pub fn config_from_value(user: &Value, warnings: &mut Vec<String>) -> Config {
    let Ok(mut root) = serde_json::to_value(Config::default()) else {
        return Config::default();
    };
    merge(&mut root, "", user, warnings);
    serde_json::from_value(root).unwrap_or_default()
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn accepts(root: &Value) -> bool {
    serde_json::from_value::<Config>(root.clone()).is_ok()
}

fn set(root: &mut Value, parent: &str, key: &str, value: Option<Value>) {
    if let Some(Value::Object(map)) = root.pointer_mut(parent) {
        match value {
            Some(v) => {
                map.insert(key.to_owned(), v);
            }
            None => {
                map.remove(key);
            }
        }
    }
}

fn merge(root: &mut Value, parent: &str, user: &Value, warnings: &mut Vec<String>) {
    let Some(fields) = user.as_object() else {
        if !user.is_null() {
            warnings.push(format!(
                "config{parent}: expected an object; using the defaults"
            ));
        }
        return;
    };
    for (key, value) in fields {
        let field = format!("{parent}/{}", escape(key));
        let default = root.pointer(&field).cloned();
        // A field this version does not have (dropped, or from a newer
        // version): its meaning is unknown here, so it is not kept.
        if default.is_none() {
            warnings.push(format!("config{field}: not used by this version; ignored"));
            continue;
        }
        if let (Some(Value::Object(_)), Value::Object(_)) = (&default, value) {
            merge(root, &field, value, warnings);
            continue;
        }
        let accepted = candidates(value, default.as_ref())
            .into_iter()
            .any(|candidate| {
                set(root, parent, key, Some(candidate));
                accepts(root)
            });
        if !accepted && let Value::Array(items) = value {
            // Keep every element that fits on its own (for example one
            // shortcut written by a newer version must not discard the rest).
            let mut kept = Vec::new();
            for item in items {
                kept.push(item.clone());
                set(root, parent, key, Some(Value::Array(kept.clone())));
                if !accepts(root) {
                    kept.pop();
                    warnings.push(format!("config{field}: invalid element {item} ignored"));
                }
            }
            set(root, parent, key, Some(Value::Array(kept)));
            continue;
        }
        if !accepted {
            set(root, parent, key, default);
            warnings.push(format!(
                "config{field}: invalid value {value}; using the default"
            ));
        }
    }
}

/// Values to try, most faithful first. For integer fields a number is
/// rounded, negatives become zero and it is saturated to decreasing widths
/// until one fits the field's type.
fn candidates(value: &Value, default: Option<&Value>) -> Vec<Value> {
    let wants_integer = default.is_some_and(|d| d.is_u64() || d.is_i64());
    match value.as_f64() {
        Some(f) if wants_integer => {
            let n = if f.is_nan() || f <= 0.0 {
                0
            } else if f >= u64::MAX as f64 {
                u64::MAX
            } else {
                f.round() as u64
            };
            [
                u64::MAX,
                u64::from(u32::MAX),
                u64::from(u16::MAX),
                u64::from(u8::MAX),
            ]
            .iter()
            .map(|cap| Value::from(n.min(*cap)))
            .collect()
        }
        _ => vec![value.clone()],
    }
}

#[cfg(test)]
mod tests {
    use super::config_from_value;

    #[test]
    fn a_missing_or_bad_use_cue_markers_loads_as_on() {
        let old: serde_json::Value =
            serde_json::from_str(r#"{"players":{"fade_ms":2000}}"#).unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&old, &mut warnings);
        assert!(c.players.use_cue_markers);
        assert!(warnings.is_empty(), "{warnings:?}");

        let bad: serde_json::Value =
            serde_json::from_str(r#"{"players":{"fade_ms":2000,"use_cue_markers":"yes"}}"#)
                .unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&bad, &mut warnings);
        assert!(c.players.use_cue_markers);
        assert_eq!(c.players.fade_ms, 2000, "the other fields are kept");
        assert_eq!(warnings.len(), 1, "{warnings:?}");

        let off: serde_json::Value =
            serde_json::from_str(r#"{"players":{"use_cue_markers":false}}"#).unwrap();
        let c = config_from_value(&off, &mut Vec::new());
        assert!(!c.players.use_cue_markers);
    }

    #[test]
    fn a_bad_remote_port_keeps_the_other_remote_fields() {
        let user: serde_json::Value =
            serde_json::from_str(r#"{"remote":{"http":{"enabled":true,"port":"x"}}}"#).unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&user, &mut warnings);
        assert!(c.remote.http.enabled);
        assert_eq!(c.remote.http.port, 7380);
        assert_eq!(warnings.len(), 1);
    }
}
