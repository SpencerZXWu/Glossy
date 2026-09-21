//! The forms of an English word: `run`, `runs`, `running`, `ran`.
//!
//! The dictionary that is asked for a word card only ever describes the form
//! that was selected, so a reader who looked up "running" learns nothing about
//! "run". The forms are built from the ordinary spelling rules plus a small
//! table of the irregular words a learner is most likely to meet. Only English
//! is handled: the other languages Glossy translates have far richer
//! inflection and no dictionary that could confirm a guessed form.

/// One inflected form, tagged so the interface can label it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Form {
    /// Stable tag, translated by the interface: `plural`, `thirdPerson`, …
    pub tag: &'static str,
    pub text: String,
}

/// Every tag `forms` can produce. The interface has a label for each of them.
const TAGS: [&str; 8] = [
    "plural",
    "thirdPerson",
    "presentParticiple",
    "past",
    "pastParticiple",
    "comparative",
    "superlative",
    "other",
];

/// Reads a form back from a stored card.
///
/// The tag is one of a closed set rather than a free string, so a name this
/// build does not know - a card written by a version that had one more of them -
/// is kept as a generic form instead of costing the whole history entry.
impl<'de> serde::Deserialize<'de> for Form {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Stored {
            tag: String,
            text: String,
        }

        let stored = Stored::deserialize(deserializer)?;
        let tag = TAGS
            .into_iter()
            .find(|tag| tag.eq_ignore_ascii_case(stored.tag.trim()))
            .unwrap_or("other");
        Ok(Form {
            tag,
            text: stored.text,
        })
    }
}

/// Words that do not follow the rules, in their common dictionary form.
const IRREGULAR_NOUNS: &[(&str, &str)] = &[
    ("man", "men"),
    ("woman", "women"),
    ("child", "children"),
    ("foot", "feet"),
    ("tooth", "teeth"),
    ("goose", "geese"),
    ("mouse", "mice"),
    ("louse", "lice"),
    ("ox", "oxen"),
    ("person", "people"),
    ("criterion", "criteria"),
    ("phenomenon", "phenomena"),
    ("datum", "data"),
    ("medium", "media"),
    ("analysis", "analyses"),
    ("basis", "bases"),
    ("crisis", "crises"),
    ("thesis", "theses"),
    ("axis", "axes"),
    ("diagnosis", "diagnoses"),
    ("index", "indices"),
    ("appendix", "appendices"),
];

const IRREGULAR_VERBS: &[(&str, &str, &str)] = &[
    // base, past, past participle (the same unless listed separately)
    ("be", "was", "been"),
    ("have", "had", "had"),
    ("do", "did", "done"),
    ("go", "went", "gone"),
    ("say", "said", "said"),
    ("get", "got", "gotten"),
    ("make", "made", "made"),
    ("know", "knew", "known"),
    ("think", "thought", "thought"),
    ("take", "took", "taken"),
    ("see", "saw", "seen"),
    ("come", "came", "come"),
    ("give", "gave", "given"),
    ("find", "found", "found"),
    ("tell", "told", "told"),
    ("become", "became", "become"),
    ("leave", "left", "left"),
    ("feel", "felt", "felt"),
    ("bring", "brought", "brought"),
    ("begin", "began", "begun"),
    ("keep", "kept", "kept"),
    ("hold", "held", "held"),
    ("write", "wrote", "written"),
    ("stand", "stood", "stood"),
    ("hear", "heard", "heard"),
    ("let", "let", "let"),
    ("mean", "meant", "meant"),
    ("set", "set", "set"),
    ("meet", "met", "met"),
    ("run", "ran", "run"),
    ("pay", "paid", "paid"),
    ("sit", "sat", "sat"),
    ("speak", "spoke", "spoken"),
    ("lie", "lay", "lain"),
    ("lead", "led", "led"),
    ("read", "read", "read"),
    ("grow", "grew", "grown"),
    ("lose", "lost", "lost"),
    ("fall", "fell", "fallen"),
    ("send", "sent", "sent"),
    ("build", "built", "built"),
    ("understand", "understood", "understood"),
    ("draw", "drew", "drawn"),
    ("break", "broke", "broken"),
    ("spend", "spent", "spent"),
    ("cut", "cut", "cut"),
    ("rise", "rose", "risen"),
    ("drive", "drove", "driven"),
    ("buy", "bought", "bought"),
    ("wear", "wore", "worn"),
    ("choose", "chose", "chosen"),
    ("eat", "ate", "eaten"),
    ("teach", "taught", "taught"),
    ("catch", "caught", "caught"),
    ("forget", "forgot", "forgotten"),
    ("hide", "hid", "hidden"),
    ("sing", "sang", "sung"),
    ("sell", "sold", "sold"),
    ("fight", "fought", "fought"),
    ("throw", "threw", "thrown"),
    ("fly", "flew", "flown"),
    ("sleep", "slept", "slept"),
    ("win", "won", "won"),
    ("put", "put", "put"),
    ("quit", "quit", "quit"),
];

const IRREGULAR_ADJECTIVES: &[(&str, &str, &str)] = &[
    // base, comparative, superlative
    ("good", "better", "best"),
    ("well", "better", "best"),
    ("bad", "worse", "worst"),
    ("far", "further", "furthest"),
    ("little", "less", "least"),
    ("many", "more", "most"),
    ("much", "more", "most"),
];

/// Nouns ending in `f`/`fe` that turn it into `ves`.
const F_TO_VES: &[&str] = &[
    "leaf", "knife", "wife", "life", "shelf", "wolf", "half", "calf", "loaf", "thief", "scarf",
    "elf", "self", "sheaf",
];

/// Nouns ending in a consonant plus `o` that take `es`.
const O_TO_OES: &[&str] = &["potato", "tomato", "hero", "echo", "torpedo", "veto"];

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// True when the word ends with consonant + vowel + consonant, which is the
/// only pattern that doubles the last letter (`stop` -> `stopped`).
fn doubled_final(word: &str) -> bool {
    // `play` -> `played`: `w`, `x` and `y` are never doubled.
    if word.ends_with('w') || word.ends_with('x') || word.ends_with('y') {
        return false;
    }
    let mut tail = word.chars().rev();
    let (Some(last), Some(middle), Some(first)) = (tail.next(), tail.next(), tail.next()) else {
        return false;
    };
    if is_vowel(first) || is_vowel(last) || !is_vowel(middle) {
        return false;
    }
    // One syllable only: `begin` -> `beginning` and `visit` -> `visited` both
    // need the stress to be known, which a rule cannot tell, so neither is
    // doubled.
    syllables(word) == 1
}

/// The number of syllables a word has, counting vowel groups.
fn syllables(word: &str) -> usize {
    let mut count = 0;
    let mut previous_was_vowel = false;
    for c in word.chars() {
        let vowel = is_vowel(c);
        if vowel && !previous_was_vowel {
            count += 1;
        }
        previous_was_vowel = vowel;
    }
    // A silent final `e` does not add a syllable.
    if word.ends_with('e') && !word.ends_with("le") && count > 1 {
        count -= 1;
    }
    count
}

/// True when the word only uses the letters the rules were written for.
fn is_english(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'')
}

fn third_person(verb: &str) -> String {
    if verb.ends_with('y') && verb.len() > 1 {
        let stem = &verb[..verb.len() - 1];
        if stem.chars().next_back().is_some_and(|c| !is_vowel(c)) {
            return format!("{stem}ies");
        }
    }
    if verb.ends_with('s')
        || verb.ends_with('x')
        || verb.ends_with('z')
        || verb.ends_with("ch")
        || verb.ends_with("sh")
        || verb.ends_with('o')
    {
        return format!("{verb}es");
    }
    format!("{verb}s")
}

fn present_participle(verb: &str) -> String {
    if let Some(stem) = verb.strip_suffix("ie") {
        return format!("{stem}ying");
    }
    if verb.ends_with('e')
        && !verb.ends_with("ee")
        && !verb.ends_with("oe")
        && !verb.ends_with("ye")
    {
        return format!("{}ing", &verb[..verb.len() - 1]);
    }
    if doubled_final(verb) {
        return format!("{verb}{}ing", verb.chars().next_back().unwrap());
    }
    format!("{verb}ing")
}

fn past_tense(verb: &str) -> String {
    if verb.ends_with('e') {
        return format!("{verb}d");
    }
    if verb.ends_with('y') && verb.len() > 1 {
        let stem = &verb[..verb.len() - 1];
        if stem.chars().next_back().is_some_and(|c| !is_vowel(c)) {
            return format!("{stem}ied");
        }
    }
    if doubled_final(verb) {
        return format!("{verb}{}ed", verb.chars().next_back().unwrap());
    }
    format!("{verb}ed")
}

fn plural(noun: &str) -> String {
    if let Some((_, plural)) = IRREGULAR_NOUNS.iter().find(|(base, _)| *base == noun) {
        return (*plural).to_string();
    }
    if let Some(stem) = F_TO_VES.iter().find(|word| **word == noun) {
        // `leaf` -> `leaves`, `knife` -> `knives`: `f` and `fe` both go.
        let keep = if stem.ends_with("fe") { 2 } else { 1 };
        return format!("{}ves", &stem[..stem.len() - keep]);
    }
    if O_TO_OES.contains(&noun) {
        return format!("{noun}es");
    }
    if noun.ends_with('s')
        || noun.ends_with('x')
        || noun.ends_with('z')
        || noun.ends_with("ch")
        || noun.ends_with("sh")
    {
        return format!("{noun}es");
    }
    if noun.ends_with('y') && noun.len() > 1 {
        let stem = &noun[..noun.len() - 1];
        if stem.chars().next_back().is_some_and(|c| !is_vowel(c)) {
            return format!("{stem}ies");
        }
    }
    format!("{noun}s")
}

/// Adjectives longer than this are compared with `more`/`most` instead of a
/// suffix, so no form is shown for them.
const MAX_SUFFIX_ADJECTIVE: usize = 7;

fn comparative(adjective: &str) -> Option<(String, String)> {
    if let Some((_, comparative, superlative)) = IRREGULAR_ADJECTIVES
        .iter()
        .find(|(base, _, _)| *base == adjective)
    {
        return Some(((*comparative).to_string(), (*superlative).to_string()));
    }
    // `-ful`, `-ous`, `-ive`, `-able`, `-ing` and `-ed` never take a suffix.
    // A word that is already a comparison would only gain another ending.
    if IRREGULAR_ADJECTIVES
        .iter()
        .any(|(_, comparative, superlative)| *comparative == adjective || *superlative == adjective)
        || adjective.ends_with("er") && adjective.len() > 4
        || adjective.ends_with("est") && adjective.len() > 4
    {
        return None;
    }
    if adjective.len() > MAX_SUFFIX_ADJECTIVE
        || syllables(adjective) > 2
        || adjective.ends_with("ful")
        || adjective.ends_with("ous")
        || adjective.ends_with("ive")
        || adjective.ends_with("able")
        || adjective.ends_with("ing")
        || adjective.ends_with("ed")
    {
        return None;
    }
    if let Some(stem) = adjective.strip_suffix('e') {
        return Some((format!("{adjective}r"), format!("{stem}st")));
    }
    if let Some(stem) = adjective.strip_suffix('y') {
        if stem.chars().next_back().is_some_and(|c| !is_vowel(c)) {
            return Some((format!("{stem}ier"), format!("{stem}iest")));
        }
    }
    if doubled_final(adjective) {
        let last = adjective.chars().next_back().unwrap();
        return Some((
            format!("{adjective}{last}er"),
            format!("{adjective}{last}est"),
        ));
    }
    Some((format!("{adjective}er"), format!("{adjective}est")))
}

/// True when the part of speech list of the dictionary mentions `wanted`.
fn is(pos: &[String], wanted: &str) -> bool {
    pos.iter().any(|entry| entry.eq_ignore_ascii_case(wanted))
}

/// Builds the forms of `word` for the parts of speech `pos` describes.
///
/// The dictionary decides which kind of word this is, so a verb is not given a
/// plural and `better` is not offered for a noun. Nothing is guessed when the
/// dictionary did not say.
pub fn forms(word: &str, pos: &[String]) -> Vec<Form> {
    let word = word.trim();
    if !is_english(word) {
        return Vec::new();
    }
    let lower = word.to_lowercase();
    let mut forms = Vec::new();

    if is(pos, "noun") {
        let text = plural(&lower);
        forms.push(Form {
            tag: "plural",
            text,
        });
    }

    if is(pos, "verb") {
        let irregular = IRREGULAR_VERBS.iter().find(|(base, past, participle)| {
            *base == lower || *past == lower || *participle == lower
        });
        // The dictionary often describes the inflected form itself, so the
        // forms are built from the base the table knows rather than from what
        // was selected.
        let base = irregular.map_or(lower.as_str(), |(base, _, _)| *base);
        forms.push(Form {
            tag: "thirdPerson",
            text: third_person(base),
        });
        forms.push(Form {
            tag: "presentParticiple",
            text: present_participle(base),
        });
        match irregular {
            Some((_, past, participle)) => {
                forms.push(Form {
                    tag: "past",
                    text: (*past).to_string(),
                });
                if past != participle {
                    forms.push(Form {
                        tag: "pastParticiple",
                        text: (*participle).to_string(),
                    });
                }
            }
            // For a regular verb the past participle is the past tense, so
            // showing both would only repeat the same word.
            None => forms.push(Form {
                tag: "past",
                text: past_tense(base),
            }),
        }
    }

    if is(pos, "adjective") || is(pos, "adverb") {
        if let Some((comparative, superlative)) = comparative(&lower) {
            forms.push(Form {
                tag: "comparative",
                text: comparative,
            });
            forms.push(Form {
                tag: "superlative",
                text: superlative,
            });
        }
    }

    // A word that is already inflected should not gain a second ending.
    forms.retain(|form| form.text != lower && !form.text.is_empty());
    forms.dedup();
    forms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn texts(forms: &[Form]) -> Vec<&str> {
        forms.iter().map(|form| form.text.as_str()).collect()
    }

    fn tags(forms: &[Form]) -> Vec<&str> {
        forms.iter().map(|form| form.tag).collect()
    }

    /// Calls the builder under a name that a test local cannot shadow.
    fn build(word: &str, pos: &[String]) -> Vec<Form> {
        super::forms(word, pos)
    }

    #[test]
    fn a_verb_gets_its_ending_forms() {
        let built = build("walk", &pos(&["verb"]));
        assert_eq!(
            tags(&built),
            vec!["thirdPerson", "presentParticiple", "past"]
        );
        assert_eq!(texts(&built), vec!["walks", "walking", "walked"]);
    }

    #[test]
    fn spelling_rules_are_applied() {
        assert!(texts(&forms("study", &pos(&["verb"]))).contains(&"studies"));
        assert!(texts(&forms("study", &pos(&["verb"]))).contains(&"studied"));
        assert!(texts(&forms("stop", &pos(&["verb"]))).contains(&"stopping"));
        assert!(texts(&forms("stop", &pos(&["verb"]))).contains(&"stopped"));
        assert!(texts(&forms("make", &pos(&["verb"]))).contains(&"making"));
        assert!(texts(&forms("make", &pos(&["verb"]))).contains(&"makes"));
        assert!(texts(&forms("die", &pos(&["verb"]))).contains(&"dying"));
        assert!(texts(&forms("watch", &pos(&["verb"]))).contains(&"watches"));
        // A word with more than one syllable does not double the last letter.
        assert!(texts(&forms("visit", &pos(&["verb"]))).contains(&"visited"));
    }

    #[test]
    fn irregular_verbs_are_listed_in_full() {
        let built = build("run", &pos(&["verb"]));
        assert_eq!(texts(&built), vec!["runs", "running", "ran"]);
        let built = build("go", &pos(&["verb"]));
        assert_eq!(texts(&built), vec!["goes", "going", "went", "gone"]);
        // A form that was selected twice is looked up from its base.
        assert!(texts(&forms("went", &pos(&["verb"]))).contains(&"going"));
        assert!(texts(&forms("written", &pos(&["verb"]))).contains(&"wrote"));
    }

    #[test]
    fn a_noun_gets_its_plural() {
        assert_eq!(texts(&forms("book", &pos(&["noun"]))), vec!["books"]);
        assert_eq!(texts(&forms("child", &pos(&["noun"]))), vec!["children"]);
        assert_eq!(texts(&forms("city", &pos(&["noun"]))), vec!["cities"]);
        assert_eq!(texts(&forms("box", &pos(&["noun"]))), vec!["boxes"]);
        assert_eq!(texts(&forms("knife", &pos(&["noun"]))), vec!["knives"]);
        assert_eq!(texts(&forms("analysis", &pos(&["noun"]))), vec!["analyses"]);
    }

    #[test]
    fn an_adjective_is_compared() {
        let built = build("tall", &pos(&["adjective"]));
        assert_eq!(texts(&built), vec!["taller", "tallest"]);
        assert!(texts(&forms("happy", &pos(&["adjective"]))).contains(&"happier"));
        assert!(texts(&forms("good", &pos(&["adjective"]))).contains(&"better"));
        // Long and suffixed adjectives are compared with a separate word.
        assert!(forms("beautiful", &pos(&["adjective"])).is_empty());
        assert!(forms("useful", &pos(&["adjective"])).is_empty());
        // A comparison is not compared again.
        assert!(forms("better", &pos(&["adjective"])).is_empty());
        assert!(forms("higher", &pos(&["adjective"])).is_empty());
    }

    #[test]
    fn the_parts_of_speech_decide_what_is_shown() {
        assert!(forms("run", &pos(&["noun"]))
            .iter()
            .all(|form| form.tag == "plural"));
        assert!(forms("run", &pos(&["verb", "noun"])).len() > 1);
        // Nothing is guessed without a part of speech.
        assert!(forms("run", &[]).is_empty());
    }

    #[test]
    fn other_languages_and_phrases_are_left_alone() {
        assert!(forms("跑步", &pos(&["verb"])).is_empty());
        assert!(forms("con", &pos(&["verb"])).len() == 3);
        assert!(forms("", &pos(&["verb"])).is_empty());
    }
}
