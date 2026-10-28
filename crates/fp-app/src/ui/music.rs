//! The phone's Music screen (phone layout spec §9): the device's music by
//! folder, artist or album, a search, and a selection to add to a player's
//! playlist. This part decides what to list; the screen draws it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::media_browser::MediaFile;

/// How the screen groups the music (PL17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Browse {
    #[default]
    Folders,
    Artists,
    Albums,
}

/// A row that opens more tracks: a sub-folder, an artist or an album
/// (`None`: the files without that tag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Group {
    Folder(PathBuf),
    Artist(Option<String>),
    Album(Option<String>),
}

/// What the Music screen remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MusicView {
    pub browse: Browse,
    /// The folder shown; `None` is the top, where every file is.
    pub folder: Option<PathBuf>,
    /// The artist or album shown; `None` lists them.
    pub at: Option<Group>,
    pub search: String,
    /// The selected files, in the order they were tapped (PL19).
    pub ticked: Vec<PathBuf>,
}

/// What to show: groups to open, then tracks (indices into the files).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Listing {
    pub groups: Vec<Group>,
    pub tracks: Vec<usize>,
}

impl MusicView {
    /// One step up (PL2): a folder to its parent, an artist or album to the
    /// list. False at the top.
    pub fn back(&mut self, files: &[MediaFile]) -> bool {
        if self.at.take().is_some() {
            return true;
        }
        let Some(folder) = self.folder.take() else {
            return false;
        };
        let top = top_folder(files);
        self.folder = folder
            .parent()
            .filter(|p| top.as_deref().is_none_or(|t| p.starts_with(t) && *p != t))
            .map(Path::to_path_buf);
        true
    }

    /// Selects `path`, or leaves it out when it was selected.
    pub fn toggle(&mut self, path: PathBuf) {
        match self.ticked.iter().position(|p| *p == path) {
            Some(at) => {
                self.ticked.remove(at);
            }
            None => self.ticked.push(path),
        }
    }

    /// Drops selected files that are not on the device any more.
    pub fn retain_listed(&mut self, files: &[MediaFile]) {
        if self.ticked.is_empty() {
            return;
        }
        let listed: BTreeSet<&Path> = files.iter().map(|f| f.path.as_path()).collect();
        self.ticked.retain(|p| listed.contains(p.as_path()));
    }
}

/// The deepest folder that holds every file, directly or below.
pub fn top_folder(files: &[MediaFile]) -> Option<PathBuf> {
    let mut parents = files.iter().filter_map(|f| f.path.parent());
    let mut top = parents.next()?.to_path_buf();
    for parent in parents {
        while !parent.starts_with(&top) {
            top = top.parent()?.to_path_buf();
        }
    }
    Some(top)
}

/// What the screen lists for `view`.
pub fn listing(files: &[MediaFile], view: &MusicView) -> Listing {
    let search = view.search.trim().to_lowercase();
    let mut listing = if !search.is_empty() {
        Listing {
            groups: Vec::new(),
            tracks: indices(files, |f| found(f, &search)),
        }
    } else {
        match (view.browse, &view.at) {
            (Browse::Folders, _) => folder_listing(files, view.folder.clone()),
            (Browse::Artists, Some(Group::Artist(name))) => Listing {
                groups: Vec::new(),
                tracks: indices(files, |f| f.artist == *name),
            },
            (Browse::Albums, Some(Group::Album(name))) => Listing {
                groups: Vec::new(),
                tracks: indices(files, |f| f.album == *name),
            },
            (Browse::Artists, _) => names(files, |f| &f.artist, Group::Artist),
            (Browse::Albums, _) => names(files, |f| &f.album, Group::Album),
        }
    };
    listing.tracks.sort_by_cached_key(|&i| {
        files
            .get(i)
            .map(|f| (f.title_or_name().to_lowercase(), f.path.clone()))
    });
    listing
}

fn indices(files: &[MediaFile], keep: impl Fn(&MediaFile) -> bool) -> Vec<usize> {
    (0..files.len())
        .filter(|&i| files.get(i).is_some_and(&keep))
        .collect()
}

/// Title, artist, album or file name holds `search` (lower case).
fn found(file: &MediaFile, search: &str) -> bool {
    let name = file
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    [
        file.title.as_ref(),
        file.artist.as_ref(),
        file.album.as_ref(),
        name.as_ref(),
    ]
    .into_iter()
    .flatten()
    .any(|t| t.to_lowercase().contains(search))
}

fn folder_listing(files: &[MediaFile], folder: Option<PathBuf>) -> Listing {
    let Some(at) = folder.or_else(|| top_folder(files)) else {
        return Listing::default();
    };
    let mut subfolders = BTreeSet::new();
    let mut tracks = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let Some(parent) = file.path.parent() else {
            continue;
        };
        if parent == at {
            tracks.push(i);
        } else if let Ok(rest) = parent.strip_prefix(&at)
            && let Some(first) = rest.components().next()
        {
            let sub = at.join(first);
            subfolders.insert((sub.to_string_lossy().to_lowercase(), sub));
        }
    }
    Listing {
        groups: subfolders
            .into_iter()
            .map(|(_, f)| Group::Folder(f))
            .collect(),
        tracks,
    }
}

fn names(
    files: &[MediaFile],
    tag: impl Fn(&MediaFile) -> &Option<String>,
    group: impl Fn(Option<String>) -> Group,
) -> Listing {
    let set: BTreeSet<(Option<String>, Option<String>)> = files
        .iter()
        .map(|f| {
            let name = tag(f).clone();
            (name.as_ref().map(|n| n.to_lowercase()), name)
        })
        .collect();
    Listing {
        groups: set.into_iter().map(|(_, n)| group(n)).collect(),
        tracks: Vec::new(),
    }
}
