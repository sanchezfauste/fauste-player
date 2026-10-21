//! Load notes for the session's `layout` (resizable layout spec RL19).
//! `fp_model::layout::lenient_layout` loads every part that does not parse
//! as its default; these notes say which, for the log.

use fp_model::{Dock, PlayerId};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::lenient::quoted;

/// One note per part of `layout` (the raw `"layout"` value, if any) that
/// does not parse, and per element of its lists that does not.
pub(crate) fn layout_notes(layout: Option<&Value>) -> Vec<String> {
    let mut notes = Vec::new();
    let Some(layout) = layout else {
        return notes;
    };
    let Some(parts) = layout.as_object() else {
        notes.push(format!(
            "session: the saved layout {} is not valid; the default layout is used",
            quoted(layout)
        ));
        return notes;
    };
    items::<(PlayerId, f32)>(parts.get("player_shares"), "player_shares", &mut notes);
    items::<PlayerId>(parts.get("hidden_players"), "hidden_players", &mut notes);
    if let Some(cartwall) = parts.get("cartwall") {
        match cartwall.as_object() {
            Some(c) => {
                part::<Dock>(c.get("dock"), "cartwall.dock", &mut notes);
                part::<Option<f32>>(c.get("size"), "cartwall.size", &mut notes);
                part::<bool>(c.get("hidden"), "cartwall.hidden", &mut notes);
            }
            None => bad("cartwall", cartwall, &mut notes),
        }
    }
    part::<bool>(parts.get("locked"), "locked", &mut notes);
    notes
}

fn part<T: DeserializeOwned>(value: Option<&Value>, name: &str, notes: &mut Vec<String>) {
    if let Some(v) = value
        && serde_json::from_value::<T>(v.clone()).is_err()
    {
        bad(name, v, notes);
    }
}

fn items<T: DeserializeOwned>(value: Option<&Value>, name: &str, notes: &mut Vec<String>) {
    let Some(v) = value else {
        return;
    };
    match v.as_array() {
        Some(list) => {
            for item in list {
                if serde_json::from_value::<T>(item.clone()).is_err() {
                    notes.push(format!(
                        "session: layout.{name}: invalid element {} ignored",
                        quoted(item)
                    ));
                }
            }
        }
        None => bad(name, v, notes),
    }
}

fn bad(name: &str, value: &Value, notes: &mut Vec<String>) {
    notes.push(format!(
        "session: layout.{name}: invalid value {}; using the default",
        quoted(value)
    ));
}
