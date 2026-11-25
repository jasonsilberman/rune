//! Diffs as people write them into a message: a whole patch, hunks under `@@` headers without
//! the file lines before them, or only lines marked `+`, `-` and ` `.

use super::parse::{hunk_header, path_of};
use super::{FileDiff, Hunk, Line, LineKind, Side, parse};
use crate::diff::pairs;

/// The files of a diff as a message shows one: what [`parse`] reads when it reads a hunk, and
/// otherwise one file of every line. There `@@` headers start hunks and number their lines (a
/// header without numbers carries on from the hunk before it), lines start at 1 without one,
/// `---`/`+++` lines name the file, and a line marked neither `+` nor `-` is unchanged, without
/// its leading space if it has one. Changed lines pair and get word changes as
/// [`FileDiff::from_texts`] pairs them.
pub fn parse_fragment(src: &str) -> Vec<FileDiff> {
    let files = parse(src);
    if files.iter().any(|f| !f.hunks.is_empty()) {
        return files;
    }
    let src = src.strip_suffix('\n').unwrap_or(src);
    if src.is_empty() {
        return Vec::new();
    }
    let lines: Vec<&str> = src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    let mut file = FileDiff::empty(None, None);
    let mut hunk: Option<Hunk> = None;
    let (mut old, mut new) = (1u32, 1u32);
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        i += 1;
        if l.starts_with("--- ") && lines.get(i).is_some_and(|n| n.starts_with("+++ ")) {
            file.old_path = path_of(&l[4..]);
            file.new_path = path_of(&lines[i][4..]);
            i += 1;
            continue;
        }
        if l.starts_with("@@") {
            file.hunks.extend(hunk.take());
            let h = hunk_header(l).map_or_else(|| open(old, new, l.trim_matches('@').trim()), |h| open(h.first(Side::Old), h.first(Side::New), &h.section));
            (old, new) = (h.old_start, h.new_start);
            hunk = Some(h);
            continue;
        }
        if l.starts_with("diff ") || l.starts_with("index ") {
            continue;
        }
        let h = hunk.get_or_insert_with(|| open(old, new, ""));
        if l.starts_with('\\') {
            if let Some(last) = h.lines.last_mut() {
                last.no_newline = true;
            }
            continue;
        }
        let line = match l.as_bytes().first() {
            Some(b'+') => {
                h.new_count += 1;
                new = new.saturating_add(1);
                Line::new(LineKind::Added, None, Some(new - 1), &l[1..])
            }
            Some(b'-') => {
                h.old_count += 1;
                old = old.saturating_add(1);
                Line::new(LineKind::Removed, Some(old - 1), None, &l[1..])
            }
            first => {
                h.old_count += 1;
                h.new_count += 1;
                old = old.saturating_add(1);
                new = new.saturating_add(1);
                Line::new(LineKind::Context, Some(old - 1), Some(new - 1), if first == Some(&b' ') { &l[1..] } else { l })
            }
        };
        h.lines.push(line);
    }
    file.hunks.extend(hunk);
    file.hunks.retain(|h| !h.lines.is_empty());
    for h in &mut file.hunks {
        if h.old_count == 0 {
            h.old_start = h.old_start.saturating_sub(1);
        }
        if h.new_count == 0 {
            h.new_start = h.new_start.saturating_sub(1);
        }
    }
    if file.hunks.is_empty() {
        return Vec::new();
    }
    pairs::pair_lines(&mut file.hunks);
    vec![file]
}

fn open(old: u32, new: u32, section: &str) -> Hunk {
    Hunk { old_start: old, old_count: 0, new_start: new, new_count: 0, section: section.to_owned(), lines: Vec::new() }
}

#[cfg(test)]
#[path = "fragment_tests.rs"]
mod tests;
