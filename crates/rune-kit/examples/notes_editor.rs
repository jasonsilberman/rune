//! A notes document: title, heading and subheading blocks, a checklist whose boxes toggle on a
//! press, Markdown block shortcuts, and two other editors' selections and carets.
//!
//! Run headless to capture the still:
//! `cargo run -p rune-kit --example notes_editor -- --snap notes.png`

use rune::tok::type_;
use rune::{AppSpec, Block, BlockLook, Color, Cx, El, HeadingLook, Mark, MarkRange, Presence, TextEdit, TextStyle, col, text_input};
use rune_kit::Kit;

struct State {
    edit: TextEdit,
}

impl Default for State {
    fn default() -> Self {
        let lines = [
            (Block::Heading(1), "Launch plan"),
            (Block::Paragraph, "Everything we need before the beta goes out on Friday."),
            (Block::Heading(2), "Checklist"),
            (Block::Check(true), "Freeze the API"),
            (Block::Check(false), "Write the release notes"),
            (Block::Check(false), "Record the demo video"),
            (Block::Heading(3), "Open questions"),
            (Block::Bullet(0), "Who owns the status page?"),
            (Block::Bullet(0), "Do we ship the importer?"),
        ];
        let text = lines.iter().map(|(_, t)| *t).collect::<Vec<_>>().join("\n");
        let blocks = lines.iter().map(|(b, _)| *b).collect();
        let bold = text.find("beta").map_or(0, |at| at);
        let marks = vec![MarkRange::new(bold..bold + 4, Mark::Bold)];
        let mut edit = TextEdit::from_parts(text, marks, blocks).markdown_shortcuts();
        edit.focused = true;
        edit.set_caret(11);
        Self { edit }
    }
}

fn heading(size: f32, weight: f32, line_height: f32, tracking: f32, before: f32, after: f32) -> HeadingLook {
    HeadingLook { size, weight, line_height, tracking, before, after }
}

fn document(ink: Color, faint: Color, page: Color) -> BlockLook {
    BlockLook {
        headings: [heading(28.0, 700.0, 34.0, -0.3, 32.0, 8.0), heading(21.0, 650.0, 28.0, -0.2, 24.0, 6.0), heading(17.0, 600.0, 24.0, 0.0, 16.0, 4.0)],
        paragraph_gap: 6.0,
        list_gap: 2.0,
        list_indent: 28.0,
        level_indent: 24.0,
        marker_gap: 8.0,
        check_size: 16.0,
        check_ring: Some(faint),
        check_fill: Some(ink),
        check_tick: Some(page),
        quote_indent: 16.0,
        quote_bar: 3.0,
    }
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let k = Kit::neutral(true);
    let at = |needle: &str| s.edit.text.find(needle).unwrap_or(0);
    let release = at("release");
    let owns = at("owns");
    let presence = [
        Presence::new(release, release + "release".len(), rune::oklch(0.66, 0.2, 0.0)),
        Presence::new(owns, owns, rune::oklch(0.68, 0.16, 150.0)),
    ];
    col().w_full().h_full().fill(k.c.page).padding(28.0).child(
        text_input(&s.edit, TextStyle { size: 15.0, line_height: 24.0, ..type_::BODY })
            .key("note")
            .w_full()
            .color(k.c.text_base)
            .presence(presence)
            .block_look(document(k.c.text_base, k.c.text_muted, k.c.page))
            .on_edit(|s: &mut State| &mut s.edit),
    )
}

fn main() {
    rune::run(AppSpec {
        fonts: &[
            include_bytes!("../../rune/assets/Inter-Regular.ttf"),
            include_bytes!("../../rune/assets/Inter-Medium.ttf"),
            include_bytes!("../../rune/assets/Inter-SemiBold.ttf"),
            include_bytes!("../../rune/assets/IBMPlexMono-Regular.ttf"),
        ],
        families: ("Inter", "IBM Plex Mono"),
        ..AppSpec::new("Notes editor", (620.0, 460.0), State::default, view)
    });
}
