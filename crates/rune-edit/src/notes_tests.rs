use super::*;

fn k(key: Key) -> KeyEvent {
    KeyEvent { key, mods: Mods::default() }
}

fn typed(e: &mut TextEdit, s: &str) {
    for c in s.chars() {
        e.key(&k(Key::Text(c.to_string().into())), None);
    }
}

fn note(text: &str, blocks: Vec<Block>) -> TextEdit {
    let mut e = TextEdit::from_parts(text, Vec::new(), blocks).markdown_shortcuts();
    e.focused = true;
    e
}

#[test]
fn block_prefixes_and_a_space_become_blocks_only_where_shortcuts_are_on() {
    for (prefix, block) in [
        ("# ", Block::Heading(1)),
        ("## ", Block::Heading(2)),
        ("### ", Block::Heading(3)),
        ("- ", Block::Bullet(0)),
        ("1. ", Block::Numbered(0)),
        ("> ", Block::Quote),
        ("[] ", Block::Check(false)),
        ("- [ ] ", Block::Check(false)),
        ("[x] ", Block::Check(true)),
    ] {
        let mut e = note("", vec![Block::Paragraph]);
        typed(&mut e, prefix);
        typed(&mut e, "go");
        assert_eq!((e.text.as_str(), e.blocks()[0]), ("go", block), "{prefix:?}");
        let mut plain = TextEdit::new("").rich();
        plain.focused = true;
        typed(&mut plain, prefix);
        assert_eq!((plain.text.as_str(), plain.blocks()[0]), (prefix, Block::Paragraph), "{prefix:?}");
    }
}

#[test]
fn return_continues_a_checklist_unchecked_ends_it_when_empty_and_leaves_a_heading_for_body() {
    let mut e = note("Ship", vec![Block::Check(true)]);
    e.key(&k(Key::Enter), None);
    assert_eq!(e.blocks(), &[Block::Check(true), Block::Check(false)]);
    e.key(&k(Key::Enter), None);
    assert_eq!(e.blocks(), &[Block::Check(true), Block::Paragraph]);

    let mut h = note("Title", vec![Block::Heading(1)]);
    h.key(&k(Key::Enter), None);
    assert_eq!(h.blocks(), &[Block::Heading(1), Block::Paragraph]);
    h.set_caret(2);
    h.key(&k(Key::Enter), None);
    assert_eq!(h.blocks()[..2], [Block::Heading(1), Block::Heading(1)]);
}

#[test]
fn backspace_at_the_start_of_a_heading_or_item_makes_it_body_text() {
    for block in [Block::Heading(2), Block::Check(true)] {
        let mut e = note("a\nb", vec![Block::Paragraph, block]);
        e.set_caret(2);
        e.key(&k(Key::Backspace), None);
        assert_eq!((e.text.as_str(), e.blocks()), ("a\nb", &[Block::Paragraph, Block::Paragraph][..]));
    }
}

#[test]
fn toggle_check_flips_one_item_as_an_undo_step_and_keeps_the_caret() {
    let mut e = note("a\nb", vec![Block::Check(false), Block::Paragraph]);
    e.set_caret(3);
    e.toggle_check(0);
    e.toggle_check(1);
    assert_eq!((e.blocks(), e.range()), (&[Block::Check(true), Block::Paragraph][..], (3, 3)));
    e.undo();
    assert_eq!(e.blocks()[0], Block::Check(false));
}

#[test]
fn heading_levels_toggle_exactly_and_survive_the_history_round_trip() {
    let mut e = note("Title", vec![Block::Heading(1)]);
    e.toggle_block(Block::Heading(2));
    assert_eq!(e.blocks()[0], Block::Heading(2));
    e.toggle_block(Block::Heading(2));
    assert_eq!(e.blocks()[0], Block::Paragraph);
    e.undo();
    e.undo();
    assert_eq!(e.blocks()[0], Block::Heading(1));
    let html = e.to_html();
    assert!(html.contains("<h1>Title</h1>"), "{html}");
}
