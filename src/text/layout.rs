use crate::text::{Alignment, FontProvider, Metric};
use std::collections::HashMap;

// TODO: Test all the layout features, that they work as they
// should. Perhaps list them as well, somewhere in docs.

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

pub(crate) fn move_forward_chars(s: &str, n: usize) -> (&str, char) {
    let mut chars = s.chars();
    let mut last_char = ' ';
    for _ in 0..n {
        if let Some(c) = chars.next() {
            last_char = c;
        }
    }
    (chars.as_str(), last_char)
}

pub(crate) fn get_line_length_and_width(
    font: &Box<dyn FontProvider>,
    metrics: &HashMap<char, Metric>,
    font_size: f32,
    max_width: Option<i32>,
    mut s: &str,
) -> (usize, i32) {
    let mut len = 0;
    let mut total_width = 0;

    // Add words until the line can't fit more words
    'outer: while let Some((word_len, word_width)) = get_word_length(
        font,
        metrics,
        font_size,
        max_width.map(|w| w - total_width),
        &s,
        len == 0,
    ) {
        // Handle a word
        len += word_len;
        total_width += word_width;
        let (s_, c) = move_forward_chars(s, word_len);
        s = s_;

        // Break if line is too long
        if s.is_empty() || max_width.iter().any(|w| total_width > *w) {
            break;
        }

        // Handle whitespace after word
        let len_before_whitespace = len;
        let mut previous_char = c;
        for c in s.chars() {
            len += 1;
            if c == '\n' {
                break 'outer;
            } else if c.is_whitespace() {
                // Update line width
                if let Some(advance) = get_char_advance(font, metrics, font_size, c, previous_char)
                {
                    total_width += advance;
                }
            } else {
                len -= 1;
                break;
            }
            previous_char = c;
        }
        s = move_forward_chars(s, len - len_before_whitespace).0;

        // Break if line is too long
        if s.is_empty() || max_width.iter().any(|w| total_width > *w) {
            break;
        }
    }

    (len, total_width)
}

// Returns None if the word doesn't fit on this line (and might fit on
// the next), Some(word length, word width) if it does (or requires
// more than a whole line's worth of space, being the first word on
// the line).
pub(crate) fn get_word_length(
    font: &Box<dyn FontProvider>,
    metrics: &HashMap<char, Metric>,
    font_size: f32,
    max_width: Option<i32>,
    s: &str,
    starts_line: bool,
) -> Option<(usize, i32)> {
    let mut len = 0;
    let mut total_width = 0;
    let mut previous_char = None;
    for c in s.chars() {
        len += 1;
        if c.is_whitespace() {
            // Whitespace = end of word, cut off the whitespace and
            // return
            len -= 1;
            break;
        }

        // Check if line was overflowed
        if let Some(previous_char) = previous_char {
            if let Some(advance) = get_char_advance(font, metrics, font_size, c, previous_char) {
                let previous_width = get_char_width(metrics, previous_char);
                total_width += advance - previous_width;
            }
        }
        let width = get_char_width(metrics, c);
        total_width += width;
        if max_width.iter().any(|w| total_width > *w) {
            len -= 1;
            if starts_line {
                // This word is the first of the line, so we can't get
                // more space by starting the word on the next
                // line. So, just cut the word here and continue the
                // next line from where we now are.

                // TODO: Consider: text layout as a feature
                // Should we hyphenate, just cut off and go to the
                // next line, maybe even hyphenate before trying to go
                // to the next line (for justify-esque text)?
                break;
            } else {
                // This word is not the first on this line, so return
                // None to signal that the line ends here, to try and
                // fit this word on the next line.
                return None;
            }
        }

        previous_char = Some(c);
    }
    Some((len, total_width))
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
