//! The name tags on a shared field's presence carets ([`Presence::label`](crate::Presence::label)).
use super::*;

impl RuntimeCore {
    /// Paints the tag of each labelled presence caret of field `i`: the name in white on a
    /// rounded chip of the caret's colour, standing on the caret's cap, its left edge on the bar.
    pub(in crate::runtime) fn push_presence_tags(&mut self, i: usize, r: [f32; 4], (scale, s): (f32, f32), (o, srgb): (f32, bool), clip: [f32; 4], fade: [f32; 4]) {
        let n = &self.tree.nodes[i];
        let (Some(spec), Some(input)) = (n.text.as_ref(), n.input.as_ref()) else { return };
        if input.presence.iter().all(|p| p.label.is_none()) {
            return;
        }
        let labels: Vec<Option<String>> = input.presence.iter().map(|p| p.label.clone()).collect();
        let style = crate::TextStyle { size: 11.0, weight: 600.0, line_height: 14.0, tracking: 0.0, ..spec.style };
        let carets = presence_carets(n, r, scale, s);
        let (pad_x, pad_y, cap) = ((5.0 * s).round() * scale, (1.0 * s).round() * scale, (5.0 * s * scale).round().max(3.0));
        for ((bar, tint), label) in carets.into_iter().zip(labels) {
            let Some(label) = label else { continue };
            let layout = self.text.layout_rich_with(&label, &[], style, s, None, crate::text::RichLayoutOptions::new());
            let (w, h) = (layout.full_width() * scale + 2.0 * pad_x, layout.height() * scale + 2.0 * pad_y);
            let chip = [bar[0].round(), (bar[1] - cap * 0.6 - h).round(), w.ceil(), h.ceil()];
            self.instances.push(rect_inst(chip, premul(tint, o, srgb), (3.0 * s * scale).round(), clip, fade));
            let origin = [chip[0] + pad_x, chip[1] + pad_y];
            self.paint.glyph_scratch.clear();
            self.text.set_glyph_inks(Color::WHITE, &[]);
            self.text.glyphs_for_layout_scaled(&layout, origin, 1.0, false, &mut self.paint.glyph_scratch);
            let ink = premul(Color::WHITE, o, srgb);
            for q in &self.paint.glyph_scratch {
                self.instances.extend(glyph_or_image(q, &self.text, &mut self.images, ink, clip, fade).map(|g| zoom_glyph(g, origin, origin, scale)));
            }
        }
    }
}
