//! A notes document: title, heading and subheading blocks, a checklist whose boxes toggle on a
//! press, Markdown block shortcuts, and two other editors' selections and carets.
//!
//! Run headless to capture the still:
//! `cargo run -p rune-kit --example notes_editor -- --snap notes.png`

use rune::tok::type_;
use rune::{AppSpec, Block, Cx, El, Mark, MarkRange, Presence, TextEdit, col, text_input};
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
        text_input(&s.edit, type_::BODY)
            .key("note")
            .w_full()
            .color(k.c.text_base)
            .presence(presence)
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
