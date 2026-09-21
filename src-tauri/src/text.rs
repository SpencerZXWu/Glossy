//! Cleanup of the text a selection comes back as.
//!
//! Ctrl+C hands over whatever the program put on the clipboard, which is the
//! text as the *renderer* laid it out rather than what the author wrote: soft
//! hyphens and zero width spaces from a browser, escape sequences from a
//! terminal, sentence after sentence broken onto the next line by the window
//! width, runs of non breaking spaces in place of the spaces a search engine
//! inserts, and stray control characters. Translating that literally wastes the
//! provider's quota and can even change the meaning, so the selection is
//! reduced to sentences before anything is asked for.
//!
//! Nothing here is destructive on purpose: only characters that no reader would
//! ever see are dropped, and a line is only joined onto the previous one when
//! the break itself was a layout artifact.

use std::iter::Peekable;
use std::str::Chars;

/// Characters that stand for a space without being one: the non breaking
/// spaces a browser writes, and the various fixed width spaces Unicode adds.
fn is_space_like(c: char) -> bool {
    matches!(
        c,
        '\u{00A0}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// Characters with no visible shape that a renderer leaves behind.
fn is_invisible(c: char) -> bool {
    matches!(
        c,
        // Soft hyphen, zero width space, zero width non-joiner and joiner,
        // word joiner, byte order mark.
        '\u{00AD}' | '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FEFF}'
            // Directional marks and the isolates that surround them.
            | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// Strips the characters a copy added rather than the author.
fn clean(raw: &str) -> String {
    let mut text = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // A terminal copy hands over the colour and cursor codes as well.
            '\u{1B}' => skip_escape(&mut chars),
            // Every line ending becomes one `\n`, so the joining below has a
            // single shape to work with.
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                text.push('\n');
            }
            '\n' => text.push('\n'),
            // A tab separates cells or columns, which a translation reads as a
            // word break anyway.
            '\t' => text.push(' '),
            c if is_space_like(c) => text.push(' '),
            c if is_invisible(c) => {}
            // Control characters, including the `\u{7F}` and the cursor moves
            // some programs place around a selection.
            c if c < ' ' || c == '\u{7F}' => {}
            c => text.push(c),
        }
    }
    text
}

/// Consumes the body of an escape sequence. `ESC [` runs to a byte in the
/// `@`–`~` range, `ESC ]` to a bell or a string terminator, and anything else is
/// a two character sequence.
fn skip_escape(chars: &mut Peekable<Chars>) {
    match chars.peek() {
        Some('[') => {
            chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
        }
        Some(']') | Some('P') | Some('^') | Some('_') => {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '\u{7}' {
                    break;
                }
                if c == '\u{1B}' && chars.peek() == Some(&'\\') {
                    chars.next();
                    break;
                }
            }
        }
        Some(_) => {
            chars.next();
        }
        None => {}
    }
}

/// How a line is put onto the text collected so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Join {
    /// Keep the break: a new sentence, heading or list item starts here.
    Keep,
    /// Join with a space: the line was wrapped in the middle of a sentence.
    Space,
    /// Join without the hyphen: the word was hyphenated across the break.
    Dehyphenate,
    /// Join without a space: the language has no word separator.
    Direct,
}

/// Decides what to do with `line` given the text collected so far.
fn join_kind(current: &str, line: &str) -> Join {
    let Some(previous) = current.chars().next_back() else {
        return Join::Direct;
    };
    let Some(next) = line.chars().next() else {
        return Join::Direct;
    };

    // A hyphen at the end of a line with a lower case letter after it is a word
    // broken across the lines; anything else is a real hyphen ("well-").
    if (previous == '-' || previous == '\u{2010}') && next.is_lowercase() {
        return Join::Dehyphenate;
    }
    if is_cjk(previous) || is_cjk(next) {
        return Join::Direct;
    }
    // The end of a sentence, a heading or a list marker is a real break.
    if ends_an_utterance(previous) || starts_a_block(line) {
        return Join::Keep;
    }
    // A capital letter usually starts a new sentence that lost its full stop to
    // a language that does not use one, so the break is kept as well.
    if next.is_uppercase() {
        return Join::Keep;
    }
    Join::Space
}

/// True for the characters of the languages that do not separate words.
fn is_cjk(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x30FF     // kana
            | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF // han
            | 0xAC00..=0xD7AF // hangul
            | 0xFF01..=0xFF60 // full width forms
            | 0x3000..=0x303F // punctuation and marks
    )
}

/// True when the character can end a sentence or an utterance.
fn ends_an_utterance(c: char) -> bool {
    matches!(
        c,
        '.' | '!'
            | '?'
            | ';'
            | ':'
            | '\u{3002}'
            | '\u{FF01}'
            | '\u{FF1F}'
            | '\u{FF1B}'
            | '\u{FF1A}'
            | '\u{2026}'
            | '"'
            | '\u{201D}'
            | '\u{2019}'
            | ')'
            | '\u{FF09}'
    )
}

/// True when the line opens a block of its own: a bullet, a number, a quote or
/// a heading, none of which belong on the line above.
fn starts_a_block(line: &str) -> bool {
    let trimmed = line.trim_start();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some('•' | '◦' | '‣' | '·' | '-' | '*' | '+' | '>' | '#' | '|' | '／') => true,
        Some(c) if c.is_ascii_digit() => {
            // `1.`, `2)` and `3、` number the items of a list.
            let rest: String = chars.take(3).collect();
            rest.starts_with('.') || rest.starts_with(')') || rest.starts_with('\u{3001}')
        }
        _ => false,
    }
}

/// Collapses every run of spaces into one.
fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if c == ' ' {
            if space {
                continue;
            }
            space = true;
        } else {
            space = false;
        }
        out.push(c);
    }
    out
}

/// Reduces a raw selection to the text that is worth sending.
///
/// The result keeps the sentence and paragraph breaks the author made (or the
/// ones the layout implies) and nothing else.
pub fn normalize(raw: &str) -> String {
    let cleaned = clean(raw);
    let mut blocks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut blank_line = false;

    for raw_line in cleaned.split('\n') {
        let line = raw_line.trim();
        if line.is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            // One empty line between paragraphs is what a translation expects;
            // a taller gap carries no meaning.
            blank_line = !blocks.is_empty();
            continue;
        }
        if blank_line && !blocks.is_empty() {
            blocks.push(String::new());
            blank_line = false;
        }
        if current.is_empty() {
            current.push_str(line);
            continue;
        }
        match join_kind(&current, line) {
            Join::Keep => {
                blocks.push(std::mem::take(&mut current));
                current.push_str(line);
            }
            Join::Space => {
                current.push(' ');
                current.push_str(line);
            }
            Join::Dehyphenate => {
                current.pop();
                current.push_str(line);
            }
            Join::Direct => current.push_str(line),
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }

    collapse_spaces(&blocks.join("\n")).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fixture is a copy of what a real program put on the clipboard,
    /// next to what the translator should be asked for.
    fn fixture(name: &str) -> (String, String) {
        let raw = std::fs::read_to_string(format!("tests/fixtures/text/{name}.txt"))
            .expect("input fixture");
        let expected = std::fs::read_to_string(format!("tests/fixtures/text/{name}.expected.txt"))
            .expect("expected fixture");
        (raw, expected)
    }

    #[test]
    fn real_world_copies_are_reduced_to_their_text() {
        for name in ["browser", "office", "terminal", "pdf", "cjk"] {
            let (raw, expected) = fixture(name);
            assert_eq!(normalize(&raw), expected.trim_end(), "fixture {name}");
        }
    }

    #[test]
    fn invisible_characters_are_dropped() {
        assert_eq!(normalize("run\u{200B}ning"), "running");
        assert_eq!(normalize("soft\u{00AD}hyphen"), "softhyphen");
        assert_eq!(normalize("\u{FEFF}hello"), "hello");
        assert_eq!(normalize("a\u{202E}b"), "ab");
    }

    #[test]
    fn spaces_that_are_not_spaces_become_spaces() {
        assert_eq!(normalize("hello\u{00A0}world"), "hello world");
        assert_eq!(normalize("a\u{2009}b"), "a b");
        assert_eq!(normalize("a   b"), "a b");
    }

    #[test]
    fn a_wrapped_sentence_is_joined() {
        assert_eq!(
            normalize("the quick brown\nfox jumps"),
            "the quick brown fox jumps"
        );
        assert_eq!(normalize("a hyphen-\nated word"), "a hyphenated word");
        // A capital letter starts something new.
        assert_eq!(
            normalize("one sentence.\nAnother one"),
            "one sentence.\nAnother one"
        );
        // So does the full stop itself.
        assert_eq!(
            normalize("short.\nlower case rest"),
            "short.\nlower case rest"
        );
    }

    #[test]
    fn languages_without_word_separators_are_joined_without_a_space() {
        assert_eq!(normalize("这是一句\n中文"), "这是一句中文");
        assert_eq!(normalize("これは\nテスト"), "これはテスト");
    }

    #[test]
    fn lists_and_numbered_items_keep_their_lines() {
        assert_eq!(normalize("• first\n• second"), "• first\n• second");
        assert_eq!(normalize("1. first\n2. second"), "1. first\n2. second");
    }

    #[test]
    fn paragraphs_are_kept_apart_by_one_empty_line() {
        assert_eq!(
            normalize("first paragraph\n\n\n\nsecond paragraph"),
            "first paragraph\n\nsecond paragraph"
        );
    }

    #[test]
    fn terminal_colours_and_cursor_codes_are_removed() {
        assert_eq!(normalize("\u{1B}[31mred\u{1B}[0m text"), "red text");
        assert_eq!(normalize("bell\u{7} here"), "bell here");
        assert_eq!(normalize("title\u{1B}]0;x\u{7}body"), "titlebody");
    }

    #[test]
    fn an_empty_selection_stays_empty() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("   \n\t\n "), "");
        assert_eq!(normalize("\u{200B}"), "");
    }

    #[test]
    fn a_plain_word_is_untouched() {
        assert_eq!(normalize("  running  "), "running");
        assert_eq!(normalize("mother-in-law"), "mother-in-law");
    }
}
