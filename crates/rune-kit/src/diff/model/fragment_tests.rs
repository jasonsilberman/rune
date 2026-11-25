use super::*;
use crate::diff::state::DiffState;
use crate::diff::view::InlineDiff;

fn kinds(f: &FileDiff) -> Vec<(LineKind, Option<u32>, Option<u32>, &str)> {
    f.hunks.iter().flat_map(|h| &h.lines).map(|l| (l.kind, l.old, l.new, l.text.as_str())).collect()
}

#[test]
fn bare_lines_number_from_one_and_pair_with_word_changes() {
    let files = parse_fragment("-let total = 1;\n+let total = 2;\n keep();\n");
    assert_eq!(files.len(), 1);
    let f = &files[0];
    assert_eq!(kinds(f), vec![
        (LineKind::Removed, Some(1), None, "let total = 1;"),
        (LineKind::Added, None, Some(1), "let total = 2;"),
        (LineKind::Context, Some(2), Some(2), "keep();"),
    ]);
    let added = &f.hunks[0].lines[1];
    assert_eq!(&added.text[added.changes[0].clone()], "2");
    assert_eq!((f.hunks[0].old_count, f.hunks[0].new_count), (2, 2));
}

#[test]
fn hunk_headers_number_their_lines_and_file_lines_name_the_file() {
    let f = &parse_fragment("@@ -40,2 +41,3 @@ fn f() {\n a\n+b\n c\n@@ trailing @@\n-d\n")[0];
    assert_eq!(f.hunks.len(), 2);
    assert_eq!(f.hunks[0].header(), "@@ -40,2 +41,3 @@ fn f() {");
    assert_eq!(f.hunks[1].section, "trailing");
    assert_eq!(kinds(f)[3], (LineKind::Removed, Some(42), None, "d"));
    assert_eq!((f.hunks[1].old_start, f.hunks[1].old_count, f.hunks[1].new_start, f.hunks[1].new_count), (42, 1, 43, 0));
    let named = &parse_fragment("--- a/src/lib.rs\n+++ b/src/lib.rs\n a\n+b\n")[0];
    assert_eq!(named.path(), "src/lib.rs");
    assert_eq!(named.language(), Some("rust"));
    assert_eq!(kinds(named), vec![(LineKind::Context, Some(1), Some(1), "a"), (LineKind::Added, None, Some(2), "b")]);
}

#[test]
fn a_whole_patch_reads_as_parse_reads_it_and_nothing_reads_as_no_files() {
    let patch = "diff --git a/x.txt b/x.txt\n--- a/x.txt\n+++ b/x.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n";
    let (a, b) = (parse_fragment(patch), parse(patch));
    assert_eq!(a.len(), b.len());
    assert_eq!(a[0].hunks, b[0].hunks);
    assert!(parse_fragment("").is_empty());
    assert!(parse_fragment("--- a/x\n+++ b/x\n").is_empty());
}

#[test]
fn an_inline_diff_draws_no_header_or_unknown_leading_lines() {
    let state = DiffState::new(parse_fragment("@@ -40,2 +40,2 @@\n a\n-b\n+c\n"));
    assert_eq!(InlineDiff::rows(&state), 3);
    let two = parse_fragment("--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n--- a/y\n+++ b/y\n@@ -1 +1 @@\n-c\n+d\n");
    assert_eq!(two.len(), 2);
    assert_eq!(InlineDiff::rows(&DiffState::new(two)), 6);
    let texts = DiffState::new(vec![FileDiff::from_texts("a.txt", "1\n2\n3\n4\n5\n6\n", "1\n2\n3\n4\n5\nsix\n")]);
    assert_eq!(InlineDiff::rows(&texts), 6);
}
