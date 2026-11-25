use super::*;

struct Doc {
    top: crate::TextEdit,
    bottom: crate::TextEdit,
    over: Vec<crate::DragOver>,
}

fn view(s: &Doc, _: &mut Cx<Doc>) -> El {
    let style = crate::TextStyle::new(14.0, 400.0).lh(20.0);
    col().w_full().padding(8.0).gap(8.0)
        .child(crate::text_input(&s.top, style).key("top").label("Top").size(280.0, 40.0)
            .on_edit(|s: &mut Doc| &mut s.top)
            .on_input_drag(|s: &mut Doc, over, _| s.over.push(over)))
        .child(el().key("between").size(280.0, 40.0).on_press(|_: &mut Doc| {}))
        .child(crate::text_input(&s.bottom, style).key("bottom").label("Bottom").size(280.0, 40.0)
            .on_edit(|s: &mut Doc| &mut s.bottom))
}

/// A selection drag that starts in one field hears what the pointer passes over: its own
/// field, a pressable keyed element between, and another field with the byte under the pointer there.
#[test]
fn a_field_drag_reports_the_keyed_element_and_byte_under_the_pointer() {
    let doc = Doc { top: crate::TextEdit::new("first words"), bottom: crate::TextEdit::new("second words"), over: Vec::new() };
    let mut rt = Runtime::new(doc, view, (320.0, 300.0), Env::new(true, 1.0));
    rt.frame(0.0);
    let rect = |rt: &Runtime<Doc>, key: &str| rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap().rect;
    let (top, between, bottom) = (rect(&rt, "top"), rect(&rt, "between"), rect(&rt, "bottom"));
    rt.pointer_down(top[0] + 1.0, top[1] + 10.0);
    rt.frame(0.02);
    for (i, [x, y]) in [[top[0] + 30.0, top[1] + 10.0], [between[0] + 30.0, between[1] + 20.0], [bottom[0] + 1.0, bottom[1] + 10.0], [bottom[2] - 1.0, bottom[1] + 10.0]].into_iter().enumerate() {
        rt.pointer_move(x, y);
        rt.frame(0.04 + i as f64 * 0.02);
    }
    rt.pointer_up(bottom[2] - 1.0, bottom[1] + 10.0);
    rt.frame(0.2);
    let keys: Vec<Option<&str>> = rt.state.over.iter().map(|o| o.key.as_deref()).collect();
    assert_eq!(keys, [Some("top"), Some("between"), Some("bottom"), Some("bottom")]);
    assert!(rt.state.over[0].byte.is_some_and(|b| b > 0));
    assert_eq!(rt.state.over[1].byte, None);
    assert_eq!(rt.state.over[2].byte, Some(0));
    assert_eq!(rt.state.over[3].byte, Some("second words".len()));
    assert!(rt.state.top.focused && rt.state.top.anchor == 0 && rt.state.top.focus > 0, "the field still selects its own text");
}
