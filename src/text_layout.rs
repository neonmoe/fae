use crate::text::{Alignment, Metric};
use std::collections::HashMap;

pub(crate) fn get_line_start_x(
    base_x: f32,
    line_width: f32,
    max_line_width: f32,
    alignment: Alignment,
) -> f32 {
    match alignment {
        Alignment::Left => base_x,
        Alignment::Center => base_x + (max_line_width - line_width) / 2.0,
        Alignment::Right => base_x + (max_line_width - line_width),
    }
}

pub(crate) fn move_forward_chars(s: &str, n: usize) -> &str {
    let mut chars = s.chars();
    for _ in 0..n {
        chars.next();
    }
    chars.as_str()
}

pub(crate) fn get_line_length_and_width(
    metrics: &HashMap<char, Metric>,
    max_width: Option<f32>,
    mut s: &str,
) -> (usize, f32) {
    let mut len = 0;
    let mut total_width = 0.0;

    // Add words until the line can't fit more words
    'outer: while let Some((word_len, word_width)) =
        get_word_length(metrics, max_width.map(|w| w - total_width), &s, len == 0)
    {
        // Handle a word
        len += word_len;
        total_width += word_width;
        s = move_forward_chars(s, word_len);

        // Break if line is too long
        if s.is_empty() || max_width.iter().any(|w| total_width > *w) {
            break;
        }

        // Handle whitespace after word
        let len_before_whitespace = len;
        let mut previous_char = None;
        for c in s.chars() {
            len += 1;
            if c == '\n' {
                break 'outer;
            } else if c.is_whitespace() {
                // Update line width
                total_width += get_char_width(metrics, c, previous_char);
            } else {
                len -= 1;
                break;
            }
            previous_char = Some(c);
        }
        s = move_forward_chars(s, len - len_before_whitespace);

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
    metrics: &HashMap<char, Metric>,
    max_width: Option<f32>,
    s: &str,
    starts_line: bool,
) -> Option<(usize, f32)> {
    let mut len = 0;
    let mut total_width = 0.0;
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
        let width = get_char_width(metrics, c, previous_char);
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

pub(crate) fn get_char_width(
    metrics: &HashMap<char, Metric>,
    current_char: char,
    _previous_char: Option<char>,
) -> f32 {
    if let Some(current_metric) = metrics.get(&current_char) {
        current_metric.size.w + 2.0 // + kerning between this char and the previous char
    } else {
        panic!(
            "'{}' has no metric but was in a string to be drawn",
            current_char
        )
    }
}
