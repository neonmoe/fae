use crate::text::types::*;
use crate::text::{Alignment, FontProvider};
use crate::types::*;
use std::collections::VecDeque;

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

// TODO(optimization): This takes around 30% of fae's runtime, needs to be optimized
pub(crate) fn get_line_length_and_width<F>(
    font: &mut dyn FontProvider,
    mut cursor: Cursor,
    get_metric: &F,
    font_size: i32,
    max_width: Option<i32>,
    s: &str,
) -> (usize, usize, i32)
where
    F: Fn(&mut dyn FontProvider, Cursor, char) -> Option<(GlyphId, RectPx)>,
{
    let mut widths = VecDeque::new();
    let mut total_width = 0; // See the end of the function: this is re-calculated there
    let mut previous_character = None;
    let mut len = 0;
    let mut can_break_len = None;
    // Linebreakers shouldn't be rendered
    let mut broken_by_line_breaker = false;

    // Find characters that fit in the given width
    for c in s.chars() {
        len += 1;
        let mut width = 0;
        if let Some(previous_character) = previous_character {
            if let Some(a) = get_char_advance(font, cursor, font_size, c, previous_character) {
                width -= get_char_width(font, cursor, get_metric, previous_character);
                cursor = cursor + a;
                width += a.advance_x;
            }
        }
        width += get_char_width(font, cursor, get_metric, c);
        widths.push_back(width);
        total_width += width;
        previous_character = Some(c);

        if can_break(c) {
            can_break_len = Some(len);
        }

        if must_break(c) {
            widths.pop_back(); // Pop off the breaking character
            broken_by_line_breaker = true;
            break;
        } else if let Some(max_width) = max_width {
            if total_width > max_width {
                if let Some(can_break_len) = can_break_len {
                    for _ in can_break_len..len {
                        widths.pop_back(); // Pop off the overflown characters
                    }
                    len = can_break_len;

                    widths.pop_back(); // Pop off the breaking character (from the width)
                } else {
                    if len > 1 {
                        widths.pop_back(); // Pop off the overflown character
                        len -= 1;
                    }
                }
                break;
            }
        }
    }

    let total_width = widths.into_iter().sum();
    let printable_len = if broken_by_line_breaker { len - 1 } else { len };

    (len, printable_len, total_width)
}

// TODO(optimization): This takes around 30% of fae's runtime (half of that inside get_line_length_and_width), needs to be optimized
pub(crate) fn get_char_advance(
    font: &mut dyn FontProvider,
    cursor: Cursor,
    font_size: i32,
    current_char: char,
    previous_char: char,
) -> Option<Advance> {
    let previous_id = font.get_glyph_id(previous_char)?;
    let current_id = font.get_glyph_id(current_char)?;
    Some(font.get_advance(previous_id, current_id, cursor, font_size))
}

pub(crate) fn get_char_width<F>(
    font: &mut dyn FontProvider,
    cursor: Cursor,
    get_metric: &F,
    c: char,
) -> i32
where
    F: Fn(&mut dyn FontProvider, Cursor, char) -> Option<(GlyphId, RectPx)>,
{
    get_metric(font, cursor, c).map(|m| m.1.width).unwrap_or(0)
}
