//! The sentence a selected word stands in.
//!
//! A dictionary entry answers what a word means, but not what it means *here*.
//! The sentence the user selected the word from is read out of the program in
//! front with UI Automation, which browsers, Word and most readers support; a
//! program that does not offer it (a terminal, a canvas application) simply
//! gets no context line. The paragraph UI Automation returns is then narrowed
//! to one sentence locally, which is the part that is worth testing.

use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomationTextPattern, IUIAutomationTextRange,
    IUIAutomationTextRangeArray, TextUnit_Line, TextUnit_Paragraph, UIA_TextPatternId,
};

/// The longest context the card shows; an enclosing unit that turns out to be
/// the whole document is replaced by its line instead.
const MAX_CONTEXT: usize = 400;

/// The sentence around `selected`, when the program in front can be read.
pub fn sentence(selected: &str) -> Option<String> {
    let paragraph = focused_text(TextUnit_Paragraph)?;
    // A provider that does not know paragraph units answers with everything it
    // has, which the line around the selection narrows down again.
    if paragraph.chars().count() > MAX_CONTEXT {
        let line = focused_text(TextUnit_Line).unwrap_or(paragraph);
        return sentence_in(&line, selected);
    }
    sentence_in(&paragraph, selected)
}

/// Reads the enclosing unit of the current selection.
fn focused_text(unit: windows::Win32::UI::Accessibility::TextUnit) -> Option<String> {
    // UI Automation needs COM, and the caller is a worker thread that has not
    // initialised it yet.
    let started = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let text = unsafe { read_enclosing(unit) };
    if started {
        unsafe { CoUninitialize() };
    }
    text
}

unsafe fn read_enclosing(unit: windows::Win32::UI::Accessibility::TextUnit) -> Option<String> {
    let automation: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_ALL).ok()?;
    let element = automation.GetFocusedElement().ok()?;
    let pattern: IUIAutomationTextPattern = element.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
    let ranges: IUIAutomationTextRangeArray = pattern.GetSelection().ok()?;
    let range = ranges.GetElement(0).ok()?;
    let text = expand(&range, unit)?;
    (!text.trim().is_empty()).then_some(text)
}

/// Expands a copy of `range` to the unit around it and returns its text.
unsafe fn expand(
    range: &IUIAutomationTextRange,
    unit: windows::Win32::UI::Accessibility::TextUnit,
) -> Option<String> {
    // The range is expanded in place, so the caller keeps the original.
    let range = range.clone();
    range.ExpandToEnclosingUnit(unit).ok()?;
    let text: BSTR = range.GetText(-1).ok()?;
    Some(text.to_string())
}

/// True for the characters that end a sentence.
fn is_boundary(c: char) -> bool {
    matches!(
        c,
        '.' | '!' | '?' | '\n'
            | '\u{3002}'  // 。
            | '\u{FF01}'  // ！
            | '\u{FF1F}'  // ？
            | '\u{FF1B}'  // ；
            | '\u{2026}' // …
    )
}

/// True for the characters that may follow a full stop before the next
/// sentence begins.
fn is_closing(c: char) -> bool {
    matches!(
        c,
        '"' | '\'' | '\u{201D}' | '\u{2019}' | ')' | '\u{FF09}' | '\u{300D}' | '\u{300F}'
    )
}

/// The sentence of `text` that contains `word`.
///
/// `None` means the sentence could not be found, or that there is nothing
/// around the word to show: a card that repeats the headword as context is
/// worse than no context line at all.
pub fn sentence_in(text: &str, word: &str) -> Option<String> {
    let text = crate::text::normalize(text);
    let word = word.trim();
    if word.is_empty() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    let lowered: Vec<char> = text.to_lowercase().chars().collect();
    let needle: Vec<char> = word.to_lowercase().chars().collect();
    // Lowercasing can change how many characters a string has, and the offsets
    // would then point at the wrong place.
    let (haystack, needle) = if lowered.len() == chars.len() {
        (lowered, needle)
    } else {
        (chars.clone(), word.chars().collect())
    };
    if needle.len() > haystack.len() {
        return None;
    }

    let start = haystack
        .windows(needle.len())
        .position(|window| window == needle.as_slice())?;
    let end = start + needle.len();

    let left = chars[..start]
        .iter()
        .rposition(|c| is_boundary(*c))
        .map_or(0, |index| index + 1);
    let mut right = chars[end..]
        .iter()
        .position(|c| is_boundary(*c))
        .map_or(chars.len(), |index| end + index + 1);
    while right < chars.len() && is_closing(chars[right]) {
        right += 1;
    }

    let sentence: String = chars[left..right].iter().collect();
    let sentence = sentence.trim().to_string();
    if sentence.chars().count() <= word.chars().count() {
        return None;
    }
    Some(truncate(&sentence, MAX_CONTEXT))
}

/// Splits a paragraph into its sentences, in order.
///
/// The text arrives normalized, so a blank line is the only remaining newline
/// and always ends a sentence.
pub fn split_sentences(text: &str) -> Vec<String> {
    let text = crate::text::normalize(text);
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while index < chars.len() {
        if !is_boundary(chars[index]) {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        // A closing quote or bracket follows the full stop rather than starting
        // the next sentence.
        while end < chars.len() && is_closing(chars[end]) {
            end += 1;
        }
        if ends_a_sentence(&chars, index, end) {
            take_sentence(&mut sentences, &chars[start..end]);
            start = end;
        }
        index = end;
    }
    take_sentence(&mut sentences, &chars[start..]);
    sentences
}

/// Whether the boundary at `at`, already extended over any closing characters to
/// `next`, really ends a sentence.
fn ends_a_sentence(chars: &[char], at: usize, next: usize) -> bool {
    if chars[at] != '.' {
        return true;
    }
    // "3.5", "e.g." and "no. 4" are one sentence: a full stop inside a number or
    // a word is followed by something that does not open one.
    match chars[next..].iter().find(|c| !c.is_whitespace()) {
        None => true,
        Some(c) => !(c.is_lowercase() || c.is_numeric()),
    }
}

fn take_sentence(sentences: &mut Vec<String>, chars: &[char]) {
    let sentence: String = chars.iter().collect();
    let sentence = sentence.trim();
    if !sentence.is_empty() {
        sentences.push(sentence.to_string());
    }
}

/// Pairs each sentence of `original` with the matching one of `translation`.
///
/// The two languages do not punctuate alike, so the two counts often disagree.
/// Rather than guess at a wrong alignment, the whole translation is then shown
/// against the whole original as a single pair.
pub fn pairs(original: &str, translation: &str) -> Vec<(String, String)> {
    let left = split_sentences(original);
    let right = split_sentences(translation);
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    if left.len() != right.len() {
        return vec![(left.join(" "), right.join(" "))];
    }
    left.into_iter().zip(right).collect()
}

/// Shortens a sentence that is too long to show, keeping its beginning.
fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut short: String = text.chars().take(limit - 1).collect();
    short.push('…');
    short
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sentence_around_the_word_is_taken() {
        let text = "The quick brown fox jumps over the lazy dog. It was late.";
        assert_eq!(
            sentence_in(text, "fox").unwrap(),
            "The quick brown fox jumps over the lazy dog."
        );
        assert_eq!(sentence_in(text, "late").unwrap(), "It was late.");
    }

    #[test]
    fn the_word_is_found_whatever_its_case() {
        assert_eq!(
            sentence_in("The Quick Fox ran. Nothing else.", "FOX").unwrap(),
            "The Quick Fox ran."
        );
    }

    #[test]
    fn a_paragraph_break_ends_the_sentence() {
        assert_eq!(
            sentence_in("First line here.\n\nfox is here too", "fox").unwrap(),
            "fox is here too"
        );
    }

    #[test]
    fn a_wrapped_line_is_part_of_the_same_sentence() {
        assert_eq!(
            sentence_in("First line here\nfox is here too", "fox").unwrap(),
            "First line here fox is here too"
        );
    }

    #[test]
    fn languages_without_spaces_are_split_at_their_own_full_stop() {
        let text = "这是第一句话。第二句话里有测试词。";
        assert_eq!(sentence_in(text, "测试词").unwrap(), "第二句话里有测试词。");
    }

    #[test]
    fn a_trailing_quote_belongs_to_the_sentence() {
        assert_eq!(
            sentence_in("She said \"the fox jumped.\" Then she left.", "fox").unwrap(),
            "She said \"the fox jumped.\""
        );
    }

    #[test]
    fn nothing_is_shown_when_there_is_no_sentence() {
        // The word is not in the text at all.
        assert_eq!(sentence_in("The quick brown fox.", "wolf"), None);
        // The word is the whole text, so the context would repeat it.
        assert_eq!(sentence_in("fox", "fox"), None);
        assert_eq!(sentence_in("", "fox"), None);
        assert_eq!(sentence_in("fox jumped", ""), None);
    }

    #[test]
    fn a_very_long_sentence_is_shortened() {
        let long = format!("{} fox.", "word ".repeat(200));
        let sentence = sentence_in(&long, "fox").unwrap();
        assert_eq!(sentence.chars().count(), MAX_CONTEXT);
        assert!(sentence.ends_with('…'));
    }

    #[test]
    fn the_splitter_breaks_a_paragraph_into_its_sentences() {
        let text = "The fox jumped. Was it late? Yes it was!";
        assert_eq!(
            split_sentences(text),
            ["The fox jumped.", "Was it late?", "Yes it was!"]
        );
    }

    #[test]
    fn the_splitter_keeps_a_closing_quote_or_bracket() {
        assert_eq!(
            split_sentences("She said \"the fox jumped.\" Then she left."),
            ["She said \"the fox jumped.\"", "Then she left."]
        );
        assert_eq!(
            split_sentences("That was that (or so he thought.) The end."),
            ["That was that (or so he thought.)", "The end."]
        );
    }

    #[test]
    fn the_splitter_ignores_a_full_stop_inside_a_number_or_an_abbreviation() {
        assert_eq!(
            split_sentences("It is 3.5 m long. Really."),
            ["It is 3.5 m long.", "Really."]
        );
        assert_eq!(
            split_sentences("See no. 4 of e.g. that file."),
            ["See no. 4 of e.g. that file."]
        );
    }

    #[test]
    fn the_splitter_knows_the_full_stops_of_languages_without_spaces() {
        assert_eq!(
            split_sentences("这是第一句话。第二句话！第三句话？"),
            ["这是第一句话。", "第二句话！", "第三句话？"]
        );
    }

    #[test]
    fn the_splitter_splits_at_a_paragraph_break_but_not_at_a_wrapped_line() {
        assert_eq!(
            split_sentences("One sentence\nsplit over two lines. And another."),
            ["One sentence split over two lines.", "And another."]
        );
        assert_eq!(
            split_sentences("First paragraph.\n\nSecond paragraph."),
            ["First paragraph.", "Second paragraph."]
        );
    }

    #[test]
    fn the_splitter_keeps_text_without_a_terminator_as_one_sentence() {
        assert_eq!(split_sentences("the fox jumped"), ["the fox jumped"]);
        assert_eq!(split_sentences(""), Vec::<String>::new());
        assert_eq!(split_sentences("   \n  "), Vec::<String>::new());
    }

    #[test]
    fn a_translation_with_the_same_sentence_count_is_paired_off() {
        let pairs = pairs("The fox jumped. It was late.", "狐狸跳了。天很晚。");
        assert_eq!(
            pairs,
            [
                ("The fox jumped.".to_string(), "狐狸跳了。".to_string()),
                ("It was late.".to_string(), "天很晚。".to_string())
            ]
        );
    }

    #[test]
    fn a_translation_that_does_not_divide_the_same_way_stays_whole() {
        // No pairing at all when one side has nothing to split.
        assert_eq!(pairs("", "狐狸跳了。"), Vec::new());
        // Counts disagree, so the whole translation answers the whole original.
        let pairs = pairs(
            "The fox jumped. It was late.",
            "狐狸跳了，当时天已经很晚了。",
        );
        assert_eq!(
            pairs,
            [(
                "The fox jumped. It was late.".to_string(),
                "狐狸跳了，当时天已经很晚了。".to_string()
            )]
        );
    }
}
