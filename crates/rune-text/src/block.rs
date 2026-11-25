//! Paragraph-level block attributes for `\n`-delimited lines.

/// Maximum nesting level (`0..=MAX_LIST_LEVEL`) for [`Block::Bullet`], [`Block::Numbered`] and
/// [`Block::Check`].
pub const MAX_LIST_LEVEL: u8 = 3;

/// Paragraph-level block attribute for a `\n`-delimited line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Block {
    #[default]
    Paragraph,
    /// Bullet list item (`•`, `◦`, `▪`) at nesting level `0..=MAX_LIST_LEVEL`.
    Bullet(u8),
    /// Numbered list item (`1.`, `2.`, …) at nesting level `0..=MAX_LIST_LEVEL`.
    Numbered(u8),
    /// Indented quote paragraph with a vertical left bar.
    Quote,
    /// Monospace code block line with a background fill; inline marks do not apply.
    Code,
    /// Heading line at level `1..=MAX_HEADING_LEVEL` (title, heading, subheading), drawn
    /// larger and bolder than the body.
    Heading(u8),
    /// Checklist item with a checkbox before it at nesting level `0..=MAX_LIST_LEVEL`; `true`
    /// when checked, which dims and strikes its text.
    Check(bool, u8),
}

/// Deepest [`Block::Heading`] level.
pub const MAX_HEADING_LEVEL: u8 = 3;

impl Block {
    pub fn is_list(self) -> bool {
        matches!(self, Block::Bullet(_) | Block::Numbered(_))
    }

    pub fn list_level(self) -> Option<u8> {
        match self {
            Block::Bullet(l) | Block::Numbered(l) | Block::Check(_, l) => Some(l),
            _ => None,
        }
    }

    pub fn with_level(self, level: u8) -> Self {
        let l = level.min(MAX_LIST_LEVEL);
        match self {
            Block::Bullet(_) => Block::Bullet(l),
            Block::Numbered(_) => Block::Numbered(l),
            Block::Check(done, _) => Block::Check(done, l),
            other => other,
        }
    }

    /// Whether `self` and `other` share the same block kind (ignoring list nesting level).
    pub fn same_kind(self, other: Block) -> bool {
        matches!(
            (self, other),
            (Block::Paragraph, Block::Paragraph)
                | (Block::Bullet(_), Block::Bullet(_))
                | (Block::Numbered(_), Block::Numbered(_))
                | (Block::Quote, Block::Quote)
                | (Block::Code, Block::Code)
                | (Block::Heading(_), Block::Heading(_))
                | (Block::Check(..), Block::Check(..))
        )
    }

    /// This block's kind as one bit, for `TextEdit::allow_blocks`.
    #[doc(hidden)]
    pub fn kind_bit(self) -> u8 {
        1 << match self {
            Block::Paragraph => 0,
            Block::Bullet(_) => 1,
            Block::Numbered(_) => 2,
            Block::Quote => 3,
            Block::Code => 4,
            Block::Heading(_) => 5,
            Block::Check(..) => 6,
        }
    }

    /// The size, weight, line height and tracking a [`Block::Heading`] line draws at: the
    /// look's when the field has one, else 1.75, 1.35 or 1.12 times the body at weights 700,
    /// 650 and 600 on the body's leading. `None` for other blocks.
    pub fn heading_look(self, m: BlockMetrics) -> Option<HeadingLook> {
        let Block::Heading(level) = self else { return None };
        let level = level.clamp(1, MAX_HEADING_LEVEL);
        if let Some(look) = m.look {
            return Some(look.headings[usize::from(level - 1)]);
        }
        let (scale, weight) = match level {
            1 => (1.75, 700.0),
            2 => (1.35, 650.0),
            _ => (1.12, 600.0),
        };
        let size = (m.size * scale).round();
        Some(HeadingLook { size, weight, line_height: 0.0, tracking: 0.0, before: (m.size * [0.9, 0.75, 0.5][usize::from(level - 1)]).round(), after: 0.0 })
    }

    /// Points between a line of `prev` and the next line, of `self`, when a new paragraph
    /// starts there: the larger of what `prev` leaves after it and `self` wants before it, as
    /// CSS margins collapse. Items of one list sit `list_gap` apart and the lines of one quote
    /// or code block touch. Without a look, only headings stand apart.
    pub fn gap_after(self, prev: Block, m: BlockMetrics) -> f32 {
        let Some(look) = m.look else {
            return self.heading_look(m).map_or(0.0, |h| h.before);
        };
        let listed = |b: Block| matches!(b, Block::Bullet(_) | Block::Numbered(_) | Block::Check(..));
        if listed(prev) && listed(self) {
            return look.list_gap;
        }
        if (prev == Block::Quote && self == Block::Quote) || (prev == Block::Code && self == Block::Code) {
            return 0.0;
        }
        let after = prev.heading_look(m).map_or(look.paragraph_gap, |h| h.after);
        let before = self.heading_look(m).map_or(look.paragraph_gap, |h| h.before);
        after.max(before)
    }

    /// Left indentation in points (before scale) for this block type.
    #[doc(hidden)]
    pub fn indent_pt(self, m: BlockMetrics) -> f32 {
        if let Some(look) = m.look {
            return match self {
                Block::Paragraph | Block::Heading(_) => 0.0,
                Block::Bullet(l) | Block::Numbered(l) | Block::Check(_, l) => look.list_indent + look.level_indent * f32::from(l.min(MAX_LIST_LEVEL)),
                Block::Quote => look.quote_indent,
                Block::Code => (m.size * 0.75).round().max(10.0),
            };
        }
        let font_size = m.size;
        let unit = (font_size * 1.45).round().max(18.0);
        match self {
            Block::Paragraph => 0.0,
            Block::Bullet(l) | Block::Numbered(l) => unit * (l.min(MAX_LIST_LEVEL) as f32 + 1.0),
            Block::Quote => (font_size * 1.1).round().max(14.0),
            Block::Code => (font_size * 0.75).round().max(10.0),
            Block::Heading(_) => 0.0,
            Block::Check(_, l) => (font_size * 1.9).round().max(26.0) + unit * l.min(MAX_LIST_LEVEL) as f32,
        }
    }

    /// Points between a list marker's trailing edge and its item's text.
    #[doc(hidden)]
    pub fn marker_gap(m: BlockMetrics) -> f32 {
        m.look.map_or(7.0, |look| look.marker_gap)
    }

    /// The style a paragraph of this block lays out in: a heading's own size, weight, leading and
    /// tracking when the field sets a [`BlockLook`], else the field's `style`.
    pub(crate) fn paragraph_style(self, style: rune_core::tok::TextStyle, m: BlockMetrics) -> rune_core::tok::TextStyle {
        match (m.look, self.heading_look(m)) {
            (Some(_), Some(h)) => rune_core::tok::TextStyle {
                size: h.size,
                weight: h.weight,
                line_height: if h.line_height > 0.0 { h.line_height } else { h.size * style.line_px() / style.size.max(1.0) },
                tracking: h.tracking,
                ..style
            },
            _ => style,
        }
    }

    /// Left indent and right padding in device px reserved during line breaking for this block.
    pub(crate) fn pad_px(self, m: BlockMetrics, scale: f32) -> [f32; 2] {
        let indent = (self.indent_pt(m) * scale).round();
        let right = if self == Block::Code { indent } else { 0.0 };
        [indent, right]
    }
}

/// How a [`Block::Heading`] level draws in a field with a [`BlockLook`], in points. A
/// `line_height` of 0 keeps the body's leading ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadingLook {
    pub size: f32,
    pub weight: f32,
    pub line_height: f32,
    pub tracking: f32,
    pub before: f32,
    pub after: f32,
}

/// A document's block metrics, in points, for a rich field that sets them (`El::block_look`):
/// each heading level's type and the room around it, the gap between paragraphs and between
/// items of a list, where list text starts and how far each level indents, the marker's gap
/// to its text, the checkbox, and the quote's indent and bar. A field without one keeps the
/// formulas a chat field uses, from its body size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockLook {
    pub headings: [HeadingLook; MAX_HEADING_LEVEL as usize],
    pub paragraph_gap: f32,
    pub list_gap: f32,
    pub list_indent: f32,
    pub level_indent: f32,
    pub marker_gap: f32,
    pub check_size: f32,
    pub check_ring: Option<rune_core::Color>,
    pub check_fill: Option<rune_core::Color>,
    pub check_tick: Option<rune_core::Color>,
    pub quote_indent: f32,
    pub quote_bar: f32,
}

impl BlockLook {
    /// A hash of every metric and colour, for keying layouts laid out with it.
    pub fn key(&self) -> u64 {
        let mut words: Vec<u32> = Vec::with_capacity(64);
        for h in &self.headings {
            words.extend([h.size, h.weight, h.line_height, h.tracking, h.before, h.after].map(f32::to_bits));
        }
        words.extend(
            [self.paragraph_gap, self.list_gap, self.list_indent, self.level_indent, self.marker_gap, self.check_size, self.quote_indent, self.quote_bar]
                .map(f32::to_bits),
        );
        for c in [self.check_ring, self.check_fill, self.check_tick] {
            words.extend(c.map_or([u32::MAX; 4], |c| [c.r, c.g, c.b, c.a].map(f32::to_bits)));
        }
        words.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &w| (h ^ u64::from(w)).wrapping_mul(0x100_0000_01b3))
    }
}

/// What a field's block decorations and indents are measured from: its body size and its
/// [`BlockLook`], if it sets one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockMetrics {
    pub size: f32,
    pub look: Option<BlockLook>,
}

impl BlockMetrics {
    pub fn new(size: f32, look: Option<BlockLook>) -> Self {
        BlockMetrics { size, look }
    }
}
