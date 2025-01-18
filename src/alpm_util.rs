use {smol_str::SmolStr, std::path::Path};

/// Filters out items from the package file list that are fully contained by the next item
/// (e.g. `/usr/bin`) is removed if the next item is `/usr/bin/cat`
pub fn deduped_files(list: &[alpm::File]) -> impl Iterator<Item = &alpm::File> {
    list.array_windows()
        .filter_map(|[a, b]| {
            let retain = !path_contains_other_path(b.name().as_ref(), a.name().as_ref());
            (retain).then_some(a)
        })
        .chain(list.last())
}

fn path_contains_other_path(haystack: &Path, needle: &Path) -> bool {
    haystack.parent() == Some(needle)
}

/// A unique identifier for a package (db/name)
#[derive(PartialEq, Eq)]
pub struct PkgId {
    pub db: SmolStr,
    pub name: SmolStr,
}
impl PkgId {
    pub fn local(name: &str) -> Self {
        Self {
            db: "local".into(),
            name: name.into(),
        }
    }
    pub fn qualified(db: &str, name: &str) -> Self {
        Self {
            db: db.into(),
            name: name.into(),
        }
    }
    pub fn is_remote(&self) -> bool {
        self.db != "local"
    }
    pub fn matches_pkg(&self, pkg: &alpm::Package) -> bool {
        pkg.db().is_some_and(|db| db.name() == self.db) && pkg.name() == self.name
    }
}

impl std::fmt::Display for PkgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&[&self.db, "/", &self.name].concat())
    }
}
