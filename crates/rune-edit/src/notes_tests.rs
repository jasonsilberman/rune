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
        ("[] ", Block::Check(false, 0)),
        ("- [ ] ", Block::Check(false, 0)),
        ("[x] ", Block::Check(true, 0)),
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
    let mut e = note("Ship", vec![Block::Check(true, 0)]);
    e.key(&k(Key::Enter), None);
    assert_eq!(e.blocks(), &[Block::Check(true, 0), Block::Check(false, 0)]);
    e.key(&k(Key::Enter), None);
    assert_eq!(e.blocks(), &[Block::Check(true, 0), Block::Paragraph]);

    let mut h = note("Title", vec![Block::Heading(1)]);
    h.key(&k(Key::Enter), None);
    assert_eq!(h.blocks(), &[Block::Heading(1), Block::Paragraph]);
    h.set_caret(2);
    h.key(&k(Key::Enter), None);
    assert_eq!(h.blocks()[..2], [Block::Heading(1), Block::Heading(1)]);
}

#[test]
fn backspace_at_the_start_of_a_heading_or_item_makes_it_body_text() {
    for block in [Block::Heading(2), Block::Check(true, 0)] {
        let mut e = note("a\nb", vec![Block::Paragraph, block]);
        e.set_caret(2);
        e.key(&k(Key::Backspace), None);
        assert_eq!((e.text.as_str(), e.blocks()), ("a\nb", &[Block::Paragraph, Block::Paragraph][..]));
    }
}

#[test]
fn a_checklist_item_nests_like_a_list_item_and_keeps_its_level_and_state() {
    let mut e = note("Ship\nNotes", vec![Block::Check(false, 0), Block::Check(true, 0)]);
    e.set_caret(6);
    e.key(&k(Key::Tab), None);
    assert_eq!(e.blocks(), &[Block::Check(false, 0), Block::Check(true, 1)]);
    e.key(&k(Key::End), None);
    e.key(&k(Key::Enter), None);
    assert_eq!(e.blocks()[2], Block::Check(false, 1));
    e.toggle_check(1);
    assert_eq!(e.blocks()[1], Block::Check(false, 1));
    e.key(&k(Key::Backspace), None);
    assert_eq!(e.blocks()[2], Block::Check(false, 0));
    let m = BlockMetrics::new(15.0, None);
    assert!(Block::Check(false, 1).indent_pt(m) > Block::Check(false, 0).indent_pt(m));
}

#[test]
fn toggle_check_flips_one_item_as_an_undo_step_and_keeps_the_caret() {
    let mut e = note("a\nb", vec![Block::Check(false, 0), Block::Paragraph]);
    e.set_caret(3);
    e.toggle_check(0);
    e.toggle_check(1);
    assert_eq!((e.blocks(), e.range()), (&[Block::Check(true, 0), Block::Paragraph][..], (3, 3)));
    e.undo();
    assert_eq!(e.blocks()[0], Block::Check(false, 0));
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

fn look() -> BlockLook {
    let h = |size, before, after| HeadingLook { size, weight: 600.0, line_height: size + 6.0, tracking: 0.0, before, after };
    BlockLook {
        headings: [h(28.0, 32.0, 8.0), h(21.0, 24.0, 6.0), h(17.0, 16.0, 4.0)],
        paragraph_gap: 6.0,
        list_gap: 2.0,
        list_indent: 28.0,
        level_indent: 24.0,
        marker_gap: 8.0,
        check_size: 16.0,
        check_ring: None,
        check_fill: None,
        check_tick: None,
        quote_indent: 16.0,
        quote_bar: 3.0,
    }
}

#[test]
fn a_look_collapses_gaps_aligns_list_text_and_leaves_other_fields_as_they_were() {
    let m = BlockMetrics::new(15.0, Some(look()));
    assert_eq!(Block::Heading(2).gap_after(Block::Paragraph, m), 24.0);
    assert_eq!(Block::Paragraph.gap_after(Block::Heading(1), m), 8.0);
    assert_eq!(Block::Check(false, 0).gap_after(Block::Bullet(0), m), 2.0);
    assert_eq!(Block::Paragraph.gap_after(Block::Check(true, 0), m), 6.0);
    assert_eq!(Block::Quote.gap_after(Block::Quote, m), 0.0);
    for b in [Block::Bullet(0), Block::Numbered(0), Block::Check(false, 0)] {
        assert_eq!(b.indent_pt(m), 28.0, "{b:?}");
    }
    assert_eq!(Block::Bullet(2).indent_pt(m), 76.0);
    let plain = BlockMetrics::new(14.0, None);
    assert_eq!(Block::Paragraph.gap_after(Block::Heading(1), plain), 0.0);
    assert_eq!(Block::Bullet(0).indent_pt(plain), 20.0);
    assert_eq!(Block::Heading(1).heading_look(plain).map(|h| (h.size, h.weight)), Some((25.0, 700.0)));
}
