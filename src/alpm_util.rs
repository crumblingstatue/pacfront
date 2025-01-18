use std::path::Path;

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
