//! A diff in the flow of a document: its rows one after another at their own height, without
//! file headers, a virtual list or handlers, long lines scrolling sideways under the gutter.

use std::any::Any;
use std::ops::RangeInclusive;
use std::rc::Rc;

use rune::{Color, El, TextStyle, col, row, text};

use super::look::Look;
use super::{DiffBind, FoldRow, GutterDrag, View};
use crate::Kit;
use crate::diff::state::{Annotation, DiffState, Expand, LineRef, Row};
use crate::diff::model::Side;
use crate::syntax::SyntaxTheme;

/// Rows an [`InlineDiff`] draws unless [`InlineDiff::most_rows`] says otherwise.
pub const INLINE_ROWS: usize = 400;

const EDGE_FADE: f32 = 16.0;
const PATH_H: f32 = 28.0;

/// Draws a [`DiffState`] inside a document, a message or a card: as tall as its rows, with no
/// file header or totals, folded unchanged ranges that say how many lines they hold, and word
/// changes and syntax colours as [`DiffView`](super::DiffView) draws them. It reads the state
/// and never changes it, so one state can be drawn in many places; a diff with more than
/// [`INLINE_ROWS`] rows draws its first ones ([`InlineDiff::rows`] says how many there are),
/// for the app to offer the whole diff in a [`DiffView`](super::DiffView). Several files each
/// start under their path.
///
/// ```ignore
/// let state = DiffState::new(diff::parse_fragment(fence));
/// InlineDiff::new(&state).key("reply-3-diff").numbers(false).fill(card).build(&kit)
/// ```
pub struct InlineDiff<'a> {
    state: &'a DiffState,
    key: String,
    numbers: bool,
    most: usize,
    fill: Option<Color>,
    syntax: Option<SyntaxTheme>,
    first: [u32; 2],
    marks: Vec<RangeInclusive<u32>>,
    font: Option<TextStyle>,
}

impl<'a> InlineDiff<'a> {
    pub fn new(state: &'a DiffState) -> InlineDiff<'a> {
        InlineDiff { state, key: "diff".into(), numbers: true, most: INLINE_ROWS, fill: None, syntax: None, first: [1, 1], marks: Vec::new(), font: None }
    }

    /// The element key, unique in the window (`"diff"` by default).
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Show line numbers in the gutter (on by default), or only the bar marking changes.
    pub fn numbers(mut self, on: bool) -> Self {
        self.numbers = on;
        self
    }

    /// Draw at most `rows` rows ([`INLINE_ROWS`] by default).
    pub fn most_rows(mut self, rows: usize) -> Self {
        self.most = rows;
        self
    }

    /// The colour it sits on, which unchanged lines show and the change tints are mixed from
    /// (the kit's `bg_base` by default).
    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Colour code with `theme` instead of [`SyntaxTheme::from_kit`].
    pub fn syntax(mut self, theme: SyntaxTheme) -> Self {
        self.syntax = Some(theme);
        self
    }

    /// Number the old side's lines from `old` and the new side's from `new` (both 1 by
    /// default), for a diff of two excerpts of a file.
    pub fn first_lines(mut self, old: u32, new: u32) -> Self {
        self.first = [old.max(1), new.max(1)];
        self
    }

    /// Code and line numbers in `font`, each row as tall as its line height (13/20 mono by
    /// default, as [`DiffView`](super::DiffView) draws them).
    pub fn font(mut self, font: TextStyle) -> Self {
        self.font = Some(font);
        self
    }

    /// Draw these new-side lines, numbered as shown, in the selected lines' tints: the lines a
    /// message points at.
    pub fn marked(mut self, lines: impl IntoIterator<Item = RangeInclusive<u32>>) -> Self {
        self.marks = lines.into_iter().collect();
        self
    }

    /// The rows `state` draws without a limit.
    pub fn rows(state: &DiffState) -> usize {
        state.rows.iter().filter(|&&r| drawn(state, r)).count()
    }

    pub fn build(self, k: &Kit) -> El {
        let InlineDiff { state: st, key, numbers, most, fill, syntax, first, marks, font } = self;
        let look = Look::new(k, fill.unwrap_or(k.c.bg_base), syntax.unwrap_or_else(|| SyntaxTheme::from_kit(k)));
        let look = match font {
            Some(font) => look.with_font(font),
            None => look,
        };
        let version = (look.version, st.rev, st.keys_rev, numbers, most, first, marks.clone(), st.wraps(), st.selection());
        rune::memo(key.clone(), version, move || {
            let view = View { look, fold_row: FoldRow::Summary, numbers, first, headers: false, totals: false, marks, bind: Box::new(Inert) };
            view.inline(st, key, most)
        })
    }
}

impl View {
    fn inline(&self, st: &DiffState, key: String, most: usize) -> El {
        let wrap = st.wraps();
        if !wrap && let Some(r) = rune::memo::layout(&format!("{key}/scroll")) {
            st.set_viewport(r);
        }
        let none: &dyn Any = &();
        let rows = st
            .rows
            .iter()
            .enumerate()
            .filter(|&(_, &r)| drawn(st, r))
            .map(|(i, r)| {
                let el = match *r {
                    Row::File(f) => self.path_row(st, f as usize),
                    Row::Note(f) => self.note(st, f as usize),
                    Row::Gap { f, gap } => self.gap(st, f as usize, gap as usize),
                    _ => self.line(none, st, i),
                };
                el.key(st.keys[i].clone())
            })
            .take(most);
        let mut list = col().key("rows").w_full().selection_group(true).role("list").label("Diff").children(rows);
        if !wrap {
            list = list.min_w(self.content_width(st).max(st.view_width()));
        }
        let scroll = col().key("scroll").w_full();
        let scroll = if wrap { scroll } else { scroll.scroll_x().scroll_edge_fade(EDGE_FADE) };
        col().key(key).dir(rune::Direction::Ltr).w_full().child(scroll.child(list))
    }

    fn path_row(&self, st: &DiffState, f: usize) -> El {
        let l = &self.look;
        let path = st.files()[f].path().to_owned();
        let mut inner = row().key("inner").h(PATH_H).items_center().px(12.0).child(text(path.clone()).font(l.small).color(l.muted).truncate_head());
        if let Some(w) = self.view_w(st, false) {
            inner = inner.max_w(w).sticky_left(0.0);
        }
        let rule = col().key("rule").w_full().h(1.0).fill(if f > 0 { l.border } else { l.bg }).a11y_hidden();
        col().w_full().role("heading").label(path).child(rule).child(inner)
    }
}

/// Whether an inline diff draws row `r`: a file's path only among several, and the lines
/// before the first hunk only when the file holds them.
fn drawn(st: &DiffState, r: Row) -> bool {
    match r {
        Row::File(_) => st.files().len() > 1,
        Row::Gap { f, gap: 0 } => st.files()[f as usize].expandable(),
        _ => true,
    }
}

struct Inert;

impl DiffBind for Inert {
    fn expands(&self) -> bool { false }
    fn has_comment(&self) -> bool { false }
    fn has_annotation(&self) -> bool { false }
    fn header_action(&self, _: &dyn Any, _: usize) -> Option<El> { None }
    fn annotation(&self, _: &dyn Any, _: &Annotation) -> Option<El> { None }
    fn bind_list(&self, list: El, _: usize, _: Rc<View>) -> El { list }
    fn bind_number_drag(&self, el: El, _: GutterDrag) -> El { el }
    fn bind_plus(&self, el: El, _: usize, _: LineRef) -> El { el }
    fn bind_code_hover(&self, el: El, _: Side, _: String) -> El { el }
    fn bind_expand(&self, el: El, _: usize, _: usize, _: Expand, _: bool) -> El { el }
    fn bind_expand_both(&self, el: El, _: usize, _: usize, _: bool, _: bool) -> El { el }
    fn bind_toggle_file(&self, el: El, _: usize, _: bool) -> El { el }
}
