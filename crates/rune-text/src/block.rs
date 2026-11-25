//! Paragraph-level block attributes for `\n`-delimited lines.

/// Maximum nesting level (`0..=MAX_LIST_LEVEL`) for [`Block::Bullet`] and [`Block::Numbered`].
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
    /// Checklist item with a checkbox before it; `true` when checked, which dims and strikes
    /// its text.
    Check(bool),
}

/// Deepest [`Block::Heading`] level.
pub const MAX_HEADING_LEVEL: u8 = 3;

impl Block {
    pub fn is_list(self) -> bool {
        matches!(self, Block::Bullet(_) | Block::Numbered(_))
    }

    pub fn list_level(self) -> Option<u8> {
        match self {
            Block::Bullet(l) | Block::Numbered(l) => Some(l),
            _ => None,
        }
    }

    pub fn with_level(self, level: u8) -> Self {
        let l = level.min(MAX_LIST_LEVEL);
        match self {
            Block::Bullet(_) => Block::Bullet(l),
            Block::Numbered(_) => Block::Numbered(l),
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
                | (Block::Check(_), Block::Check(_))
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
            Block::Check(_) => 6,
        }
    }

    /// The font size and weight a [`Block::Heading`] line draws at for a body of `size`
    /// points: level 1 a title, 2 a heading, 3 a subheading. `None` for other blocks.
    pub fn heading_style(self, size: f32) -> Option<(f32, f32)> {
        match self {
            Block::Heading(level) => Some(match level.clamp(1, MAX_HEADING_LEVEL) {
                1 => ((size * 1.75).round(), 700.0),
                2 => ((size * 1.35).round(), 650.0),
                _ => ((size * 1.12).round(), 600.0),
            }),
            _ => None,
        }
    }

    /// Room in points above a line of this block that follows another, for a body of `size`
    /// points: a heading stands apart from what comes before it.
    pub fn space_before_pt(self, size: f32) -> f32 {
        match self {
            Block::Heading(1) => (size * 0.9).round(),
            Block::Heading(2) => (size * 0.75).round(),
            Block::Heading(_) => (size * 0.5).round(),
            _ => 0.0,
        }
    }

    /// Left indentation in points (before scale) for this block type.
    #[doc(hidden)]
    pub fn indent_pt(self, font_size: f32) -> f32 {
        let unit = (font_size * 1.45).round().max(18.0);
        match self {
            Block::Paragraph => 0.0,
            Block::Bullet(l) | Block::Numbered(l) => unit * (l.min(MAX_LIST_LEVEL) as f32 + 1.0),
            Block::Quote => (font_size * 1.1).round().max(14.0),
            Block::Code => (font_size * 0.75).round().max(10.0),
            Block::Heading(_) => 0.0,
            Block::Check(_) => (font_size * 1.9).round().max(26.0),
        }
    }

    /// Left indent and right padding in device px reserved during line breaking for this block.
    pub(crate) fn pad_px(self, font_size: f32, scale: f32) -> [f32; 2] {
        let indent = (self.indent_pt(font_size) * scale).round();
        let right = if self == Block::Code { indent } else { 0.0 };
        [indent, right]
    }
}
