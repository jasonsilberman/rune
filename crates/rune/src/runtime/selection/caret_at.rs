//! `Cx::caret_layout`: where a caret at a UTF-16 offset of a text element sits after layout,
//! for a view that places something beside a caret it doesn't own (a collaborator's name
//! flag). It is a layout read like `Cx::layout`, under a key no element can have, so it
//! settles, replays from `memo` and re-runs the view the same way.
use super::*;

/// The prefix of a caret read's key, which no element key starts with.
const CARET: &str = "\u{1}caret:";

/// The layout-read key a caret read of `key` at UTF-16 `offset` is noted under.
pub(crate) fn caret_key(key: &str, offset: usize) -> String {
    format!("{CARET}{offset}:{key}")
}

/// The UTF-16 offset and element key a caret read's key stands for.
pub(crate) fn caret_read(key: &str) -> Option<(usize, &str)> {
    let (offset, key) = key.strip_prefix(CARET)?.split_once(':')?;
    Some((offset.parse().ok()?, key))
}

/// The byte of `text` that UTF-16 `offset` falls on, or its end past the last unit.
fn byte_at(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        if units >= offset {
            return byte;
        }
        units += ch.len_utf16();
    }
    text.len()
}

impl RuntimeCore {
    /// Node `i`'s caret line at UTF-16 `offset` of its text (a field's edit text, not what it
    /// displays), `[x, y, w, h]` from its laid-out rect's top-left, or `None` for a node that
    /// lays out no text.
    pub(in crate::runtime) fn caret_in(nodes: &[Node], i: usize, offset: usize, scale: f32) -> Option<[f32; 4]> {
        let n = &nodes[i];
        let (spec, laid) = (n.text.as_ref()?, n.text_layout.as_ref()?);
        let index = match n.input.as_ref() {
            Some(inp) => inp.edit.display_index(byte_at(&inp.edit.text, offset)),
            None => byte_at(&spec.content, offset),
        }
        .min(spec.content.len());
        let layout = laid.view();
        let width = n.target[2] - n.target[0];
        let tw = text_width(layout.full_width()) / scale;
        let x = text_x(spec, n.inset[0], width - n.inset[2], tw, layout.flow());
        let bb = layout.caret(index, parley::layout::Affinity::Downstream, 1.5 * scale);
        let h = if bb.y1 > bb.y0 { (bb.y1 - bb.y0) as f32 / scale } else { spec.style.line_px() };
        Some([x + bb.x0 as f32 / scale, n.inset[1] + bb.y0 as f32 / scale, 1.5, h])
    }
}

impl cx::CxCore {
    #[track_caller]
    pub(crate) fn caret_layout(&self, key: &str, offset: usize) -> Option<[f32; 4]> {
        self.layout(&caret_key(key, offset))
    }
}

impl<S> Cx<S> {
    /// The caret line `[x, y, w, h]` a caret at UTF-16 `offset` of the text keyed `key` would
    /// draw, whether or not anything is focused there: for a [`text`](crate::text) its content,
    /// for a [`text_input`](crate::text_input) its edit text (bytes a rich field hides, such as
    /// a list marker's, map to where the field shows them). Window points as [`Self::layout`]
    /// gives them, unscrolled, so a flag placed beside a collaborator's caret inside the same
    /// scroll viewport subtracts the viewport content's own `layout` rect. `None` while the
    /// element isn't laid out or lays out no text. An offset past the end reads the end.
    ///
    /// A view that reads it re-runs when that caret moves by more than half a point, as
    /// [`Self::layout`] does.
    #[track_caller]
    #[inline(always)]
    pub fn caret_layout(&self, key: &str, offset: usize) -> Option<[f32; 4]> {
        self.core.caret_layout(key, offset)
    }
}

#[cfg(test)]
#[path = "caret_at_tests.rs"]
mod tests;
