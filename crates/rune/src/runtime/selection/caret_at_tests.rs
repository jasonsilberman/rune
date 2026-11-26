use crate::{Block, Cx, El, Env, Runtime, TextEdit, TextStyle, col, el, text, text_input};

const OFFSETS: [usize; 6] = [0, 3, 6, 11, 14, 999];

struct S {
    edit: TextEdit,
    key: &'static str,
}

fn view(s: &S, cx: &mut Cx<S>) -> El {
    let probes = OFFSETS.iter().enumerate().map(|(i, &offset)| {
        let read = cx.caret_layout(s.key, offset);
        let label = read.map_or_else(|| "none".to_string(), |r| format!("{},{},{},{}", r[0], r[1], r[2], r[3]));
        el().key(format!("probe{i}")).size(1.0, 1.0).label(label)
    });
    col().size(400.0, 300.0).padding(20.0)
        .child(text_input(&s.edit, TextStyle::new(15.0, 400.0)).key("field").w(300.0).on_edit(|s: &mut S| &mut s.edit))
        .child(text("plain text").key("plain"))
        .children(probes)
}

fn reads(edit: TextEdit, key: &'static str) -> (Vec<Option<[f32; 4]>>, [f32; 4]) {
    let mut rt = Runtime::new(S { edit, key }, view, (400.0, 300.0), Env::new(true, 1.0));
    rt.frame(0.0);
    for _ in 0..4 { rt.frame(1.0 / 60.0); }
    let els = rt.elements();
    let field = els.iter().find(|e| e.key.as_deref() == Some(key)).map_or([0.0; 4], |e| e.rect);
    let probes = (0..OFFSETS.len())
        .map(|i| {
            let label = els.iter().find(|e| e.key.as_deref() == Some(&format!("probe{i}"))).and_then(|e| e.label.clone())?;
            let parts: Vec<f32> = label.split(',').map(|p| p.parse().ok()).collect::<Option<_>>()?;
            Some([parts[0], parts[1], parts[2], parts[3]])
        })
        .collect();
    (probes, field)
}

#[test]
fn caret_layout_follows_offsets_down_and_across_a_field() {
    let (probes, field) = reads(TextEdit::new("first line\nsecond"), "field");
    let [start, three, six, end_first, second, past] = probes[..] else { panic!("six probes") };
    let (start, three, six) = (start.expect("offset 0"), three.expect("offset 3"), six.expect("offset 6"));
    let (end_first, second, past) = (end_first.expect("offset 11"), second.expect("offset 14"), past.expect("past the end"));
    assert!((start[0] - field[0]).abs() < 1.0, "offset 0 sits at the field's left edge: {start:?} in {field:?}");
    assert!(start[1] >= field[1] - 1.0 && start[3] > 0.0, "a caret has height inside the field: {start:?}");
    assert!(start[0] < three[0] && three[0] < six[0], "carets on one line advance: {start:?} {three:?} {six:?}");
    assert!((three[1] - start[1]).abs() < 0.5, "carets on one line share a top");
    assert!(second[1] > start[1] + start[3] * 0.5, "an offset on the second line sits below the first: {second:?}");
    assert!((end_first[1] - second[1]).abs() < 0.5, "the newline's offset starts the second line");
    assert!(past[0] >= second[0] && (past[1] - second[1]).abs() < 0.5, "an offset past the end reads the end");
}

#[test]
fn caret_layout_reads_a_rich_fields_edit_text_not_its_markers() {
    let plain = TextEdit::new("item");
    let mut listed = TextEdit::from_parts("item".to_string(), Vec::new(), vec![Block::Bullet(0)]).rich();
    listed.focused = false;
    let (bare, _) = reads(plain, "field");
    let (bullet, _) = reads(listed, "field");
    let (a, b) = (bare[0].expect("plain offset 0"), bullet[0].expect("listed offset 0"));
    assert!(b[0] > a[0] + 4.0, "offset 0 of a list item sits after its marker: {a:?} {b:?}");
}

#[test]
fn caret_layout_reads_plain_text_and_nothing_for_a_missing_key() {
    let (plain, _) = reads(TextEdit::new("x"), "plain");
    assert!(plain[0].is_some() && plain[3].is_some(), "a text element has carets");
    let (missing, _) = reads(TextEdit::new("x"), "nowhere");
    assert!(missing.iter().all(Option::is_none), "no element, no caret");
}
