//! Schema upgrades. Each document carries `schema_version`; migrations turn
//! version k+1 into k+2 until the current version is reached.

use serde_json::{Value, json};

use crate::atomic::ParseError;

pub type Migration = fn(Value) -> Result<Value, String>;

pub fn upgrade(
    mut doc: Value,
    current: u32,
    migrations: &[Migration],
) -> Result<Value, ParseError> {
    let version = doc
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| ParseError::Corrupt("missing or invalid schema_version".into()))?;
    if version == 0 {
        return Err(ParseError::Corrupt("schema_version 0 is invalid".into()));
    }
    if version > current {
        return Err(ParseError::TooNew(format!(
            "written by a newer version of the app (schema {version}, this build reads up to {current})"
        )));
    }
    for from in version..current {
        let step = usize::try_from(from - 1).map_err(|e| ParseError::Corrupt(e.to_string()))?;
        let migration = migrations
            .get(step)
            .ok_or_else(|| ParseError::Corrupt(format!("no migration from schema {from}")))?;
        doc = migration(doc).map_err(ParseError::Corrupt)?;
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("schema_version".into(), json!(from + 1));
        }
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn add_field(mut v: Value) -> Result<Value, String> {
        v.as_object_mut()
            .ok_or("not an object")?
            .insert("added".into(), json!(true));
        Ok(v)
    }

    #[test]
    fn upgrades_step_by_step_and_stamps_the_version() {
        let out = upgrade(json!({"schema_version": 1}), 2, &[add_field]).unwrap();
        assert_eq!(out, json!({"schema_version": 2, "added": true}));
    }

    #[test]
    fn current_version_passes_through() {
        let out = upgrade(json!({"schema_version": 1, "x": 1}), 1, &[]).unwrap();
        assert_eq!(out, json!({"schema_version": 1, "x": 1}));
    }

    #[test]
    fn newer_and_invalid_versions_are_rejected() {
        assert!(matches!(
            upgrade(json!({"schema_version": 5}), 1, &[]),
            Err(ParseError::TooNew(_))
        ));
        assert!(matches!(
            upgrade(json!({"schema_version": 0}), 1, &[]),
            Err(ParseError::Corrupt(_))
        ));
        assert!(matches!(
            upgrade(json!({}), 1, &[]),
            Err(ParseError::Corrupt(_))
        ));
        assert!(matches!(
            upgrade(json!({"schema_version": 1}), 3, &[add_field]),
            Err(ParseError::Corrupt(_))
        ));
    }
}
