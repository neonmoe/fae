use crate::text::types::*;
use crate::text::{Alignment, FontProvider};

// Sources for the following two arrays: https://en.wikipedia.org/wiki/Whitespace_character#Unicode
// Characters that must break lines when encountered:
static LINE_BREAKERS: [char; 7] = [
    '\u{A}', '\u{B}', '\u{C}', '\u{D}', '\u{85}', '\u{2028}', '\u{2029}',
];
// Characters that can be used for breaking a line cleanly
static WORD_BREAKERS: [char; 19] = [
    '\u{9}', '\u{20}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
    '\u{2005}', '\u{2006}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{205F}', '\u{3000}', '\u{180E}',
    '\u{200B}', '\u{200C}', '\u{200D}',
];

fn can_break(c: char) -> bool {
    WORD_BREAKERS.contains(&c) || LINE_BREAKERS.contains(&c)
}

fn must_break(c: char) -> bool {
    LINE_BREAKERS.contains(&c)
}

pub(crate) fn get_line_start_x(
    base_x: i32,
    line_width: i32,
    max_line_width: i32,
    alignment: Alignment,
) -> i32 {
    match alignment {
        Alignment::Left => base_x,
        Alignment::Center => base_x + (max_line_width - line_width) / 2,
        Alignment::Right => base_x + (max_line_width - line_width),
    }
}

pub(crate) fn get_line_length_and_width(
    font: &mut dyn FontProvider,
    mut cursor: Cursor,
    font_size: i32,
    max_width: Option<i32>,
    glyphs: &[(char, GlyphId)],
) -> (usize, usize, i32) {
    let mut total_width = 0;
    let mut width_since_can_break = 0;
    let mut previous_id = None;
    let mut len = 0;
    let mut can_break_len = None;
    // Linebreakers shouldn't be rendered
    let mut broken_by_line_breaker = false;

    for (c, glyph_id) in glyphs {
        len += 1;
        let mut width = 0;
        if let Some(previous_id) = previous_id {
            let a = font.get_advance(previous_id, *glyph_id, font_size);
            cursor = cursor + a;
            width += a.advance_x - font.get_metric(previous_id, font_size).width;
        }
        width += font.get_metric(*glyph_id, font_size).width;
        total_width += width;
        previous_id = Some(*glyph_id);

        if can_break(*c) {
            can_break_len = Some(len);
            width_since_can_break = 0;
        }
        if can_break_len.is_some() {
            width_since_can_break += width;
        }

        if must_break(*c) {
            total_width -= width; // Pop off the breaking character
            broken_by_line_breaker = true;
            break;
        } else if let Some(max_width) = max_width {
            if total_width > max_width {
                if let Some(can_break_len) = can_break_len {
                    total_width -= width_since_can_break;
                    len = can_break_len;
                } else if len > 1 {
                    total_width -= width; // Pop off the overflown character
                    len -= 1;
                }
                break;
            }
        }
    }

    let printable_len = if broken_by_line_breaker { len - 1 } else { len };
    (len, printable_len, total_width)
}

pub(crate) fn get_line_length(glyphs: &[(char, GlyphId)]) -> (usize, usize) {
    if let Some(len) = glyphs.iter().position(|(c, _)| must_break(*c)) {
        let stride = if len < glyphs.len() { len + 1 } else { len };
        (stride, len)
    } else {
        (glyphs.len(), glyphs.len())
    }
}
