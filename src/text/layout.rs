use crate::text::{Alignment, FontProvider, Metric};
use std::collections::{HashMap, VecDeque};

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

fn can_break(c: &char) -> bool {
    WORD_BREAKERS.contains(c) || LINE_BREAKERS.contains(c)
}

fn must_break(c: &char) -> bool {
    LINE_BREAKERS.contains(c)
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

// FIXME: There seems to be a problem when rendering lines that end in <micro>s
pub(crate) fn get_line_length_and_width(
    font: &Box<dyn FontProvider>,
    metrics: &HashMap<char, Metric>,
    font_size: f32,
    max_width: Option<i32>,
    s: &str,
) -> (usize, i32) {
    let mut widths = VecDeque::new();
    let mut total_width = 0; // See the end of the function: this is re-calculated there
    let mut previous_character = None;
    let mut len = 0;
    let mut can_break_len = None;

    // Find characters that fit in the given width
    for c in s.chars() {
        len += 1;
        let mut width = 0;
        if let Some(previous_character) = previous_character {
            if let Some(a) = get_char_advance(font, metrics, font_size, c, previous_character) {
                width -= get_char_width(metrics, previous_character);
                width += a;
            }
        }
        width += get_char_width(metrics, c);
        widths.push_back(width);
        total_width += width;
        previous_character = Some(c);

        if can_break(&c) {
            can_break_len = Some(len);
        }

        if must_break(&c) {
            widths.pop_back(); // Pop off the breaking character
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
                    widths.pop_back(); // Pop off the overflown character
                    len -= 1;
                }
                break;
            }
        }
    }

    let total_width = widths.into_iter().fold(0, |acc, x| acc + x);

    (len, total_width)
}

pub(crate) fn get_char_advance(
    font: &Box<dyn FontProvider>,
    metrics: &HashMap<char, Metric>,
    font_size: f32,
    current_char: char,
    previous_char: char,
) -> Option<i32> {
    font.get_advance(
        metrics.get(&previous_char)?.glyph_id,
        metrics.get(&current_char)?.glyph_id,
        font_size,
    )
}

pub(crate) fn get_char_width(metrics: &HashMap<char, Metric>, c: char) -> i32 {
    metrics.get(&c).unwrap().size.width
}
