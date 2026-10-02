//! The columns of the track table (feedback 2 spec O24): which exist, which
//! are required, and the pure rules that edit the ordered list the operator
//! keeps in `ui.table_columns`.

use serde::{Deserialize, Serialize};

/// A column of the track table. The names are the names of the data they
/// show; `Number` is the `#` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableColumn {
    /// The position of the entry in its playlist, with its status icon.
    Number,
    Title,
    Artist,
    Album,
    Date,
    Genre,
    Duration,
    /// How long the intro lasts, from the start of the audible part.
    Intro,
    FileName,
}

impl TableColumn {
    /// Every column, in the order Settings lists the hidden ones.
    pub const ALL: [TableColumn; 9] = [
        TableColumn::Number,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Album,
        TableColumn::Date,
        TableColumn::Genre,
        TableColumn::Duration,
        TableColumn::Intro,
        TableColumn::FileName,
    ];

    /// Title and Duration are always shown.
    pub fn is_required(self) -> bool {
        matches!(self, TableColumn::Title | TableColumn::Duration)
    }

    /// The name used in files and as the suffix of the `col-` messages.
    pub fn name(self) -> &'static str {
        match self {
            TableColumn::Number => "number",
            TableColumn::Title => "title",
            TableColumn::Artist => "artist",
            TableColumn::Album => "album",
            TableColumn::Date => "date",
            TableColumn::Genre => "genre",
            TableColumn::Duration => "duration",
            TableColumn::Intro => "intro",
            TableColumn::FileName => "file_name",
        }
    }
}

impl TableColumn {
    /// The column called `name` in a file, if this version has one.
    pub fn from_name(name: &str) -> Option<TableColumn> {
        TableColumn::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// The columns of a table that was never configured.
pub fn default_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::Number,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Duration,
    ]
}

/// A column list the table can use: every column at most once (the first
/// wins) and the required ones present. A missing Title goes first, after
/// `#` when that leads the list; a missing Duration goes last.
pub fn normalize_columns(list: &[TableColumn]) -> Vec<TableColumn> {
    let mut out: Vec<TableColumn> = Vec::with_capacity(list.len() + 2);
    for column in list {
        if !out.contains(column) {
            out.push(*column);
        }
    }
    if !out.contains(&TableColumn::Title) {
        let at = usize::from(out.first() == Some(&TableColumn::Number));
        out.insert(at, TableColumn::Title);
    }
    if !out.contains(&TableColumn::Duration) {
        out.push(TableColumn::Duration);
    }
    out
}

/// `list` with `column` shown (appended at the end) or hidden. A required
/// column cannot be hidden.
pub fn with_column_shown(
    list: &[TableColumn],
    column: TableColumn,
    shown: bool,
) -> Vec<TableColumn> {
    let mut out = normalize_columns(list);
    let present = out.contains(&column);
    if shown && !present {
        out.push(column);
    } else if !shown && present && !column.is_required() {
        out.retain(|c| *c != column);
    }
    out
}

/// `list` with the column at position `from` moved so that it ends at
/// position `to` (both clamped to the list). Any column can move.
pub fn move_column(list: &[TableColumn], from: usize, to: usize) -> Vec<TableColumn> {
    let mut out = normalize_columns(list);
    if out.is_empty() {
        return out;
    }
    let last = out.len() - 1;
    let (from, to) = (from.min(last), to.min(last));
    let column = out.remove(from);
    out.insert(to, column);
    out
}

/// The rows of the Settings list: the shown columns in their order (`true`),
/// then the hidden ones in the order of [`TableColumn::ALL`] (`false`).
pub fn column_rows(list: &[TableColumn]) -> Vec<(TableColumn, bool)> {
    let shown = normalize_columns(list);
    let hidden = TableColumn::ALL.iter().filter(|c| !shown.contains(c));
    shown
        .iter()
        .map(|c| (*c, true))
        .chain(hidden.map(|c| (*c, false)))
        .collect()
}
