//! Best-effort detection of the language a selection is written in.
//!
//! Only the "translate only these source languages" setting needs this, and a
//! wrong guess would swallow a selection the user meant to read. `detect`
//! therefore answers `None` as soon as the evidence is thin, and every caller
//! treats "no answer" as "translate it anyway".

use std::collections::HashMap;

/// The most frequent short words of the languages written with the Latin
/// alphabet, used to score a text. Diacritics are kept as spelled.
const LATIN_WORDS: &[(&str, &[&str])] = &[
    (
        "en",
        &[
            "the", "of", "and", "to", "in", "is", "that", "it", "for", "was", "with", "you",
            "this", "are", "not", "have", "but", "they", "from", "will", "there", "what", "when",
            "hello", "please", "thanks", "would", "about",
        ],
    ),
    (
        "de",
        &[
            "der", "die", "das", "und", "ist", "nicht", "ein", "eine", "mit", "sich", "auf", "für",
            "von", "zu", "den", "dem", "des", "auch", "wird", "oder", "wie", "als", "bei", "ich",
            "wir", "kein", "sehr", "danke", "hallo",
        ],
    ),
    (
        "fr",
        &[
            "le", "la", "les", "des", "et", "est", "une", "un", "pour", "dans", "que", "qui",
            "avec", "pas", "sur", "sont", "plus", "au", "ce", "il", "elle", "vous", "nous", "aux",
            "du", "merci", "bonjour", "être",
        ],
    ),
    (
        "es",
        &[
            "el", "la", "los", "las", "de", "que", "y", "en", "es", "un", "una", "para", "con",
            "no", "por", "se", "su", "al", "como", "más", "pero", "está", "este", "cuando", "muy",
            "hay", "gracias", "hola",
        ],
    ),
    (
        "it",
        &[
            "il", "lo", "gli", "le", "di", "che", "e", "in", "un", "una", "per", "con", "non",
            "si", "del", "come", "più", "questo", "sono", "anche", "quando", "molto", "della",
            "grazie", "ciao",
        ],
    ),
    (
        "pt",
        &[
            "o", "a", "os", "as", "de", "que", "e", "em", "um", "uma", "para", "com", "não", "por",
            "se", "seu", "ao", "como", "mais", "mas", "está", "este", "quando", "muito", "dos",
            "obrigado", "olá",
        ],
    ),
    (
        "nl",
        &[
            "de", "het", "een", "en", "van", "is", "dat", "op", "te", "in", "niet", "met", "voor",
            "zijn", "er", "aan", "ook", "als", "maar", "door", "om", "bij", "wordt", "dit",
            "bedankt",
        ],
    ),
    (
        "pl",
        &[
            "nie",
            "jest",
            "się",
            "i",
            "w",
            "na",
            "z",
            "do",
            "że",
            "to",
            "jak",
            "ale",
            "po",
            "tak",
            "dla",
            "od",
            "tym",
            "ich",
            "być",
            "oraz",
            "czy",
            "pod",
            "przez",
            "dziękuję",
        ],
    ),
    (
        "tr",
        &[
            "bir",
            "ve",
            "bu",
            "için",
            "ile",
            "de",
            "da",
            "değil",
            "olarak",
            "çok",
            "daha",
            "ne",
            "var",
            "gibi",
            "ama",
            "en",
            "her",
            "ki",
            "olan",
            "sonra",
            "teşekkür",
        ],
    ),
    (
        "vi",
        &[
            "của", "và", "là", "không", "có", "trong", "một", "được", "cho", "với", "này", "những",
            "khi", "như", "để", "người", "các", "đã", "trên", "thì", "cảm",
        ],
    ),
    (
        "id",
        &[
            "yang", "dan", "di", "itu", "dengan", "untuk", "tidak", "ini", "dari", "dalam", "akan",
            "pada", "adalah", "ke", "oleh", "juga", "saya", "kita", "karena", "bisa", "terima",
        ],
    ),
    (
        "ms",
        &[
            "yang", "dan", "di", "itu", "dengan", "untuk", "tidak", "ini", "dari", "dalam", "akan",
            "pada", "ialah", "ke", "oleh", "juga", "saya", "kita", "kerana", "boleh", "terima",
        ],
    ),
    (
        "cs",
        &[
            "a", "se", "na", "je", "že", "to", "v", "s", "z", "do", "jako", "ale", "pro", "jsou",
            "byl", "které", "nebo", "při", "tak", "jeho", "děkuji",
        ],
    ),
    (
        "sk",
        &[
            "a", "sa", "na", "je", "že", "to", "v", "s", "z", "do", "ako", "ale", "pre", "sú",
            "bol", "ktoré", "alebo", "pri", "tak", "jeho", "ďakujem",
        ],
    ),
    (
        "sv",
        &[
            "och", "att", "är", "för", "med", "det", "som", "inte", "på", "en", "ett", "den",
            "har", "till", "av", "om", "men", "eller", "så", "från", "jag", "tack",
        ],
    ),
    (
        "da",
        &[
            "og", "at", "er", "for", "med", "det", "som", "ikke", "på", "en", "et", "den", "har",
            "til", "af", "om", "men", "eller", "så", "fra", "jeg", "tak", "hvad", "meget", "være",
            "bliver", "siger",
        ],
    ),
    (
        "no",
        &[
            "og", "å", "er", "for", "med", "det", "som", "ikke", "på", "en", "et", "den", "har",
            "til", "av", "om", "men", "eller", "så", "fra", "jeg", "takk", "hva", "mye", "være",
            "blir", "sier",
        ],
    ),
    (
        "fi",
        &[
            "ja", "on", "ei", "se", "että", "oli", "mutta", "kun", "myös", "niin", "tai", "jos",
            "kuin", "vain", "ovat", "olla", "hän", "sen", "kiitos",
        ],
    ),
    (
        "hu",
        &[
            "a",
            "az",
            "és",
            "hogy",
            "nem",
            "is",
            "egy",
            "de",
            "van",
            "ez",
            "meg",
            "csak",
            "mint",
            "már",
            "még",
            "vagy",
            "volt",
            "kell",
            "köszönöm",
        ],
    ),
    (
        "ro",
        &[
            "și",
            "în",
            "de",
            "la",
            "cu",
            "pe",
            "este",
            "nu",
            "care",
            "pentru",
            "din",
            "se",
            "o",
            "un",
            "mai",
            "sau",
            "dar",
            "sunt",
            "acest",
            "mulțumesc",
        ],
    ),
];

/// Letters that only Ukrainian uses, and the ones only Russian uses.
const UKRAINIAN_LETTERS: &str = "іїєґІЇЄҐ";
const RUSSIAN_LETTERS: &str = "ыэёъЫЭЁЪ";

/// Guesses the language of `text`, or `None` when it cannot be told apart.
///
/// The answer is the base code of a language (`zh` rather than `zh-CN`), which
/// is what a whitelist wants to match against.
pub fn detect(text: &str) -> Option<String> {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return None;
    }

    // Kana come first: a Japanese sentence mixes them with Chinese characters.
    if let Some(code) = dominant_script(&letters, is_kana, "ja") {
        return Some(code);
    }
    for (is_script, code) in [
        (is_hangul as fn(char) -> bool, "ko"),
        (is_arabic, "ar"),
        (is_devanagari, "hi"),
        (is_thai, "th"),
        (is_hebrew, "he"),
        (is_greek, "el"),
    ] {
        if let Some(code) = dominant_script(&letters, is_script, code) {
            return Some(code);
        }
    }
    // Cyrillic is one alphabet shared by several languages.
    if is_dominant(&letters, is_cyrillic) {
        return cyrillic_language(&letters);
    }
    // Chinese last of the scripts: it shares characters with Japanese.
    if let Some(code) = dominant_script(&letters, is_han, "zh") {
        return Some(code);
    }

    latin_language(&words(text))
}

/// `Some(code)` when at least half of the letters belong to one script.
fn dominant_script(letters: &[char], is_script: fn(char) -> bool, code: &str) -> Option<String> {
    is_dominant(letters, is_script).then(|| code.to_string())
}

fn is_dominant(letters: &[char], is_script: fn(char) -> bool) -> bool {
    let count = letters.iter().filter(|c| is_script(**c)).count();
    count > 0 && count * 2 >= letters.len()
}

fn cyrillic_language(letters: &[char]) -> Option<String> {
    if letters.iter().any(|c| UKRAINIAN_LETTERS.contains(*c)) {
        return Some("uk".to_string());
    }
    if letters.iter().any(|c| RUSSIAN_LETTERS.contains(*c)) {
        return Some("ru".to_string());
    }
    None
}

/// Language with the highest function word score, when one of them stands out.
fn latin_language(words: &[String]) -> Option<String> {
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for word in words {
        *counts.entry(word.as_str()).or_insert(0) += 1;
    }

    let mut scores: Vec<(&str, i32)> = LATIN_WORDS
        .iter()
        .map(|(code, list)| {
            let score = list
                .iter()
                .map(|word| counts.get(word).copied().unwrap_or(0))
                .sum();
            (*code, score)
        })
        .collect();
    // Most matches first; a tie between two languages stays unresolved.
    scores.sort_by_key(|(_, score)| -score);

    let (code, best) = *scores.first()?;
    if best == 0 || scores.get(1).is_some_and(|(_, second)| *second == best) {
        return None;
    }
    Some(code.to_string())
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// True when both codes name the same language, ignoring a region suffix.
///
/// `zh-TW` and `zh` describe one language for the purpose of a whitelist, and
/// detection cannot tell the two Chinese scripts apart from a selection alone.
pub fn same_base(a: &str, b: &str) -> bool {
    fn base(code: &str) -> String {
        code.split(['-', '_'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
    }
    let (a, b) = (base(a), base(b));
    !a.is_empty() && a == b
}

fn is_han(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2FA1F)
}

fn is_kana(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D)
}

fn is_hangul(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF)
}

fn is_cyrillic(c: char) -> bool {
    matches!(c as u32, 0x0400..=0x052F)
}

fn is_greek(c: char) -> bool {
    matches!(c as u32, 0x0370..=0x03FF | 0x1F00..=0x1FFF)
}

fn is_arabic(c: char) -> bool {
    matches!(c as u32,
        0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
}

fn is_hebrew(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x05FF | 0xFB1D..=0xFB4F)
}

fn is_thai(c: char) -> bool {
    matches!(c as u32, 0x0E00..=0x0E7F)
}

fn is_devanagari(c: char) -> bool {
    matches!(c as u32, 0x0900..=0x097F)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detected(text: &str) -> Option<String> {
        detect(text)
    }

    #[test]
    fn reads_the_languages_that_use_their_own_alphabet() {
        assert_eq!(
            detected("選択したテキストを翻訳します").as_deref(),
            Some("ja")
        );
        assert_eq!(
            detected("선택한 텍스트를 번역합니다").as_deref(),
            Some("ko")
        );
        assert_eq!(detected("翻译选中的文字").as_deref(), Some("zh"));
        assert_eq!(
            detected("Перевести выделенный текст").as_deref(),
            Some("ru")
        );
        assert_eq!(
            detected("Перекласти виділений текст").as_deref(),
            Some("uk")
        );
        assert_eq!(detected("Μετάφραση κειμένου").as_deref(), Some("el"));
        assert_eq!(detected("ترجمة النص المحدد").as_deref(), Some("ar"));
        assert_eq!(detected("תרגום טקסט נבחר").as_deref(), Some("he"));
        assert_eq!(detected("แปลข้อความที่เลือก").as_deref(), Some("th"));
        assert_eq!(detected("चयनित पाठ का अनुवाद").as_deref(), Some("hi"));
    }

    #[test]
    fn the_kana_of_a_sentence_win_over_its_chinese_characters() {
        // One Chinese character among the kana must not turn the text Chinese.
        assert_eq!(detected("これは本です").as_deref(), Some("ja"));
        assert_eq!(detected("本").as_deref(), Some("zh"));
    }

    #[test]
    fn reads_the_languages_written_with_the_latin_alphabet() {
        let sentences = [
            ("en", "The quick brown fox is not in the garden"),
            ("de", "Der Hund ist nicht im Garten und die Katze"),
            ("fr", "Le chat est dans la maison et il dort"),
            ("es", "El gato está en la casa y no quiere salir"),
            ("it", "Il gatto è in casa e non vuole uscire"),
            ("pt", "O gato está em casa e não quer sair"),
            ("nl", "De kat is niet in het huis en dat is jammer"),
            ("pl", "Nie ma go w domu i to jest dla mnie"),
            ("tr", "Bu bir kitap ve çok güzel ama"),
            ("vi", "Đây là một cuốn sách và không có gì"),
            ("id", "Ini adalah sebuah buku dan tidak untuk dijual"),
            ("cs", "To je kniha a to je pro tebe, děkuji"),
            ("sv", "Det är en bok och den är inte min"),
            ("da", "Det er en bog af meget værdi, siger hun ikke"),
            ("no", "Det er en bok av mye verdi og den blir ikke min"),
            ("fi", "Se on kirja ja se ei ole minun"),
            ("hu", "Ez egy könyv és nem az enyém"),
            ("ro", "Este o carte și nu este a mea"),
        ];

        // Malay is deliberately absent: it shares almost every one of its short
        // words with Indonesian, so the two are not worth telling apart.

        for (expected, sentence) in sentences {
            assert_eq!(
                detected(sentence).as_deref(),
                Some(expected),
                "for {sentence}"
            );
        }
    }

    #[test]
    fn keeps_quiet_when_the_evidence_is_thin() {
        for text in ["ok", "yes", "1234", "?", "e", "la", ""] {
            assert_eq!(detected(text), None, "for {text:?}");
        }
    }

    #[test]
    fn a_word_shared_by_several_languages_stays_unresolved() {
        // The same two-letter article belongs to French, Spanish and Italian.
        assert_eq!(detected("la casa"), None);
    }

    #[test]
    fn a_mixed_script_selection_is_not_called_chinese() {
        // Half Latin, half Chinese: no script holds a majority of the letters.
        assert_ne!(detected("hello 世界").as_deref(), Some("zh"));
    }

    #[test]
    fn a_region_variant_matches_its_base_language() {
        assert!(same_base("zh-TW", "zh"));
        assert!(same_base("pt-BR", "pt"));
        assert!(same_base("en", "EN-US"));
        assert!(same_base("zh_CN", "zh-tw"));
        assert!(!same_base("zh", "ja"));
        assert!(!same_base("", ""));
        assert!(!same_base("en", ""));
    }
}
