//! Decides whether a selection should be presented as a dictionary style entry
//! (word / short phrase) or as a plain sentence translation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A single word or a short phrase: show phonetics, part of speech, definitions.
    Word,
    /// A sentence or paragraph: show a smooth translation.
    Sentence,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Word => "word",
            Kind::Sentence => "sentence",
        }
    }
}

/// Characters that unambiguously end a clause or sentence.
fn is_sentence_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | ';' | '\n' | '。' | '！' | '？' | '；' | '…')
}

pub fn has_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32,
            0x3040..=0x30FF |   // kana
            0x3400..=0x4DBF |   // CJK ext A
            0x4E00..=0x9FFF |   // CJK unified
            0xAC00..=0xD7AF |   // hangul
            0xF900..=0xFAFF     // CJK compatibility
        )
    })
}

/// Japanese kana, Hiragana and Katakana.
pub fn contains_kana(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x3040..=0x30FF | 0xFF66..=0xFF9D))
}

/// Korean Hangul syllables and jamo.
pub fn contains_hangul(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF))
}

pub fn classify(raw: &str) -> Kind {
    let text = raw.trim();
    if text.is_empty() {
        return Kind::Sentence;
    }

    if has_cjk(text) {
        return classify_cjk(text);
    }

    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Kind::Sentence;
    }

    if tokens.len() == 1 {
        let token = tokens[0];
        // A trailing period ("etc.") or an abbreviation stays a word lookup.
        return if token.chars().count() <= 48 {
            Kind::Word
        } else {
            Kind::Sentence
        };
    }

    // Multi word selections: punctuation or several words mean real prose.
    let terminates_early = text
        .chars()
        .take(text.chars().count().saturating_sub(1))
        .any(is_sentence_terminator);
    if terminates_early {
        return Kind::Sentence;
    }

    if tokens.len() <= 4 && text.chars().count() <= 32 {
        Kind::Word
    } else {
        Kind::Sentence
    }
}

fn classify_cjk(text: &str) -> Kind {
    if text.contains('\n') {
        return Kind::Sentence;
    }

    // CJK text has no spaces, so a short punctuation free run is treated as a phrase.
    let significant: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    let has_terminator = significant
        .iter()
        .take(significant.len().saturating_sub(1))
        .any(|c| is_sentence_terminator(*c));

    if has_terminator || significant.len() > 12 {
        Kind::Sentence
    } else {
        Kind::Word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word_is_a_word() {
        assert_eq!(classify("running"), Kind::Word);
        assert_eq!(classify("  hello  "), Kind::Word);
        assert_eq!(classify("running."), Kind::Word);
    }

    #[test]
    fn short_phrase_is_a_word() {
        assert_eq!(classify("good morning"), Kind::Word);
        assert_eq!(classify("carpe diem"), Kind::Word);
        assert_eq!(classify("as soon as possible"), Kind::Word);
    }

    #[test]
    fn prose_is_a_sentence() {
        assert_eq!(
            classify("The quick brown fox jumps over the lazy dog."),
            Kind::Sentence
        );
        assert_eq!(classify("Hello, how are you doing today?"), Kind::Sentence);
        assert_eq!(classify("line one\nline two"), Kind::Sentence);
        assert_eq!(
            classify("this is a fairly long run of words that keeps going"),
            Kind::Sentence
        );
    }

    #[test]
    fn cjk_words_and_sentences() {
        assert_eq!(classify("你好"), Kind::Word);
        assert_eq!(classify("一丝不苟"), Kind::Word);
        assert_eq!(classify("今天天气很好，我们去公园散步吧。"), Kind::Sentence);
        assert_eq!(classify("こんにちは"), Kind::Word);
    }

    #[test]
    fn empty_selection_is_treated_as_sentence() {
        assert_eq!(classify("   "), Kind::Sentence);
        assert_eq!(classify(""), Kind::Sentence);
    }
}
