//! Unit and currency conversion for the text the user selected.
//!
//! Everything here is best effort: the popup shows the translation first and the
//! conversions are annotations next to it, so a slow or missing exchange rate
//! must never hold back, or break, a card.

mod catalog;
mod currency;

use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use catalog::{
    candidates, currency as currency_of, target_currency, target_system, Currency, System, Unit,
    CURRENCIES, UNITS,
};

/// How many conversions a single card shows.
const MAX_CONVERSIONS: usize = 4;
/// How long all the currency lookups of a single card may take together.
const CURRENCY_BUDGET: Duration = Duration::from_millis(2500);

/// One annotated unit switch, e.g. `12 ft` -> `≈ 3.66 m`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversion {
    /// `length`, `mass`, `volume`, `speed`, `area`, `temperature` or `currency`.
    pub category: String,
    /// The text as it was written, e.g. `12 ft`.
    pub original: String,
    /// The same amount in the unit the reader expects, e.g. `3.66 m`.
    pub converted: String,
    /// The ratio or formula behind the switch, e.g. `1 ft = 0.3048 m`.
    pub rate: String,
    /// Where a live rate came from, when the conversion needed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_source: Option<String>,
    /// The date the live rate was published, `YYYY-MM-DD`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_date: Option<String>,
    /// The only rate available was older than the freshness window.
    #[serde(default)]
    pub stale: bool,
}

/// Converts the units in `text` that a reader of `target_language` would not
/// expect, in the order they appear.
pub async fn conversions(
    client: &reqwest::Client,
    text: &str,
    target_language: &str,
    cache: Option<&Path>,
) -> Vec<Conversion> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    let system = target_system(target_language);
    let money_target = target_currency(target_language);
    let deadline = Instant::now() + CURRENCY_BUDGET;

    let mut out: Vec<Conversion> = Vec::new();
    for hit in scan(text, has_kana(text)) {
        let conversion = match hit.found {
            Found::Measure { unit, value } => measure(unit, value, &hit, system),
            Found::Money { code, value } => {
                let Some(target) = money_target else { continue };
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    continue;
                }
                let rate = tokio::time::timeout(
                    remaining,
                    currency::rate(client, code, target, cache),
                )
                .await
                .ok()
                .flatten();
                rate.and_then(|rate| money(&hit, code, value, target, &rate))
            }
        };
        let Some(conversion) = conversion else { continue };
        if out.iter().any(|seen| seen.original == conversion.original) {
            continue;
        }
        out.push(conversion);
        if out.len() >= MAX_CONVERSIONS {
            break;
        }
    }
    out
}

/// The conversion a measurement needs, or `None` when the reader already reads
/// that unit.
fn measure(unit: &'static Unit, value: f64, hit: &Hit, system: System) -> Option<Conversion> {
    // Kelvin is metric, but it is never what a reader wants to see.
    if unit.system == system && unit.display != "K" {
        return None;
    }
    let base = value * unit.scale + unit.offset;
    let target = choose(&candidates(unit.category, system), base)?;
    let converted = (base - target.offset) / target.scale;
    Some(Conversion {
        category: unit.category.as_str().to_string(),
        original: hit.original.clone(),
        converted: format!(
            "{} {}",
            group_thousands(&significant(converted, 3)),
            target.display
        ),
        rate: rate_line(unit, target),
        rate_source: None,
        rate_date: None,
        stale: false,
    })
}

/// The conversion an amount of money needs, or `None` when the reader is
/// already at home in that currency.
fn money(
    hit: &Hit,
    code: &str,
    value: f64,
    target: &str,
    rate: &currency::Rate,
) -> Option<Conversion> {
    if code.eq_ignore_ascii_case(target) {
        return None;
    }
    if !rate.value.is_finite() || rate.value <= 0.0 {
        return None;
    }
    let target = currency_of(target)?;
    Some(Conversion {
        category: "currency".to_string(),
        original: hit.original.clone(),
        converted: format_amount(target, value * rate.value),
        rate: format!("1 {} = {} {}", code, significant(rate.value, 6), target.code),
        rate_source: Some(rate.source.to_string()),
        rate_date: rate.date.clone(),
        stale: rate.stale,
    })
}

/// The unit a value is best written in: the largest candidate that is still at
/// least one, or the smallest one when the amount is tiny.
fn choose(candidates: &[&'static Unit], base: f64) -> Option<&'static Unit> {
    let mut smallest = None;
    for candidate in candidates.iter().rev() {
        let value = (base - candidate.offset) / candidate.scale;
        if value.abs() >= 1.0 {
            return Some(candidate);
        }
        smallest = Some(*candidate);
    }
    smallest
}

/// The line that explains a switch, e.g. `1 ft = 0.3048 m`.
fn rate_line(from: &Unit, to: &Unit) -> String {
    if from.category == catalog::Category::Temperature {
        return match (from.display, to.display) {
            ("°F", "°C") => "°C = (°F − 32) × 5/9".to_string(),
            ("°C", "°F") => "°F = °C × 9/5 + 32".to_string(),
            ("K", "°C") => "°C = K − 273.15".to_string(),
            ("K", "°F") => "°F = K × 9/5 − 459.67".to_string(),
            _ => format!("1 {} = {} {}", from.display, significant(from.scale, 7), to.display),
        };
    }
    format!(
        "1 {} = {} {}",
        from.display,
        significant(from.scale / to.scale, 7),
        to.display
    )
}

/// A number with the given number of significant digits, without trailing
/// zeros: `8.04672` with 3 digits is `8.05`, and `804.672` is `805`.
fn significant(value: f64, digits: i32) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let decimals = (digits - 1 - magnitude).clamp(0, 7) as usize;
    trim_zeros(format!("{value:.decimals$}"))
}

/// An amount of money as that currency writes it, e.g. `¥1,234.50`.
fn format_amount(currency: &Currency, value: f64) -> String {
    let decimals = currency.decimals as usize;
    let amount = group_thousands(&format!("{value:.decimals$}"));
    let symbol = currency.symbol;
    if symbol.is_empty() {
        format!("{} {amount}", currency.code)
    } else if symbol.ends_with(|c: char| c.is_ascii_alphanumeric()) {
        format!("{symbol} {amount}")
    } else {
        format!("{symbol}{amount}")
    }
}

/// `1234567.5` -> `1,234,567.5`.
fn group_thousands(text: &str) -> String {
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text, None),
    };
    let (sign, digits) = match integer.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", integer),
    };
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    match fraction {
        Some(fraction) => format!("{sign}{grouped}.{fraction}"),
        None => format!("{sign}{grouped}"),
    }
}

fn trim_zeros(text: String) -> String {
    if !text.contains('.') {
        return text;
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// A measurement or an amount of money found in the text.
#[derive(Debug, Clone, Copy)]
enum Found {
    Measure { unit: &'static Unit, value: f64 },
    Money { code: &'static str, value: f64 },
}

/// One match: the amount plus the exact text it was written with.
#[derive(Debug, Clone)]
struct Hit {
    original: String,
    found: Found,
}

/// Every number in the text that has a unit or a currency attached.
fn scan(text: &str, kana: bool) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (value, start, end) in numbers(text) {
        if let Some((currency, prefix)) = prefix_money(&text[..start], kana) {
            hits.push(Hit {
                original: text[start - prefix..end].trim().to_string(),
                found: Found::Money { code: currency.code, value },
            });
            continue;
        }
        let Some((consumed, found)) = suffix_match(&text[end..], kana) else {
            continue;
        };
        let found = match found {
            Suffix::Measure(unit) => Found::Measure { unit, value },
            Suffix::Money(currency) => Found::Money { code: currency.code, value },
        };
        hits.push(Hit {
            original: text[start..end + consumed].trim().to_string(),
            found,
        });
    }
    hits
}

#[derive(Debug, Clone, Copy)]
enum Suffix {
    Measure(&'static Unit),
    Money(&'static Currency),
}

/// The unit or currency code written after a number, together with the bytes it
/// takes up (including the space before it).
fn suffix_match(after: &str, kana: bool) -> Option<(usize, Suffix)> {
    let skipped = after.len() - after.trim_start().len();
    let rest = &after[skipped..];

    let mut best: Option<(usize, Suffix)> = None;
    let mut take = |length: usize, found: Suffix| {
        if best.as_ref().is_none_or(|(current, _)| length > *current) {
            best = Some((length, found));
        }
    };

    for unit in UNITS {
        for alias in unit.aliases {
            if let Some(length) = alias_then(rest, alias) {
                take(length, Suffix::Measure(unit));
            }
        }
    }
    for currency in CURRENCIES {
        for alias in currency.aliases {
            if let Some(length) = alias_then(rest, alias) {
                take(length, Suffix::Money(resolve_yen(currency, alias, kana)));
            }
        }
    }

    best.map(|(length, found)| (skipped + length, found))
}

/// The bytes `rest` gives to `alias`, if the number is followed by it.
///
/// A multi word alias also matches its plural, because that is how an amount is
/// written: `200 US dollars`, not `200 US dollar`.
fn alias_then(rest: &str, alias: &str) -> Option<usize> {
    if starts_with_ci(rest, alias) && boundary_after(rest, alias.len()) {
        return Some(alias.len());
    }
    let plural = alias.len() + 1;
    if alias.contains(' ')
        && !alias.ends_with('s')
        && rest.len() >= plural
        && rest.is_char_boundary(plural)
        && starts_with_ci(rest, alias)
        && rest[alias.len()..plural].eq_ignore_ascii_case("s")
        && boundary_after(rest, plural)
    {
        return Some(plural);
    }
    None
}

/// A currency symbol written before a number, e.g. the `$` of `$200`.
fn prefix_money(before: &str, kana: bool) -> Option<(&'static Currency, usize)> {
    let trimmed = before.trim_end();
    let skipped = before.len() - trimmed.len();

    let mut best: Option<(&'static Currency, usize)> = None;
    for currency in CURRENCIES {
        for alias in currency.aliases {
            let length = alias.len();
            if trimmed.len() < length || !trimmed.is_char_boundary(trimmed.len() - length) {
                continue;
            }
            let head = &trimmed[trimmed.len() - length..];
            if !head.eq_ignore_ascii_case(alias) {
                continue;
            }
            // A symbol is only read as one when no word runs into it, but
            // writing without spaces is normal, so only ASCII joins count.
            let joined = trimmed[..trimmed.len() - length]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric());
            if joined {
                continue;
            }
            if best.is_none_or(|(_, current)| length > current) {
                best = Some((resolve_yen(currency, alias, kana), length));
            }
        }
    }

    best.map(|(currency, length)| (currency, skipped + length))
}

/// `¥` is the symbol of two currencies: a text written with kana means the yen.
fn resolve_yen(
    currency: &'static Currency,
    alias: &str,
    kana: bool,
) -> &'static Currency {
    if kana && currency.code == "CNY" && (alias == "¥" || alias == "￥") {
        return currency_of("JPY").unwrap_or(currency);
    }
    currency
}

/// Every number in the text with the byte range it occupies.
fn numbers(text: &str) -> Vec<(f64, usize, usize)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let attached = index > 0 && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'.');
        if !byte.is_ascii_digit() || attached {
            index += 1;
            continue;
        }
        let start = index;
        let mut decimal = false;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte.is_ascii_digit() {
                index += 1;
            } else if byte == b',' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
                index += 1;
            } else if byte == b'.' && !decimal && bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
                decimal = true;
                index += 1;
            } else {
                break;
            }
        }
        let raw: String = text[start..index].chars().filter(|c| *c != ',').collect();
        if let Ok(value) = raw.parse::<f64>() {
            out.push((value, start, index));
        }
    }
    out
}

fn starts_with_ci(text: &str, alias: &str) -> bool {
    text.len() >= alias.len()
        && text.is_char_boundary(alias.len())
        && text[..alias.len()].eq_ignore_ascii_case(alias)
}

/// A unit is only read as a unit when no word continues it: `5 minutes` is not
/// five metres.
fn boundary_after(text: &str, length: usize) -> bool {
    text[length..]
        .chars()
        .next()
        .is_none_or(|next| !next.is_ascii_alphanumeric())
}

/// Japanese is the one language where `¥` is not the yuan.
fn has_kana(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{3040}'..='\u{30ff}').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str, language: &str) -> Vec<Conversion> {
        let system = target_system(language);
        let hits = scan(text, has_kana(text));
        let mut out: Vec<Conversion> = Vec::new();
        for hit in hits {
            if let Found::Measure { unit, value } = hit.found {
                if let Some(conversion) = measure(unit, value, &hit, system) {
                    out.push(conversion);
                }
            }
        }
        out
    }

    fn single(text: &str, language: &str) -> Conversion {
        let found = read(text, language);
        assert_eq!(found.len(), 1, "expected one conversion in `{text}`: {found:?}");
        found.into_iter().next().expect("one conversion")
    }

    fn money_of(text: &str, language: &str) -> Vec<(&'static str, f64)> {
        scan(text, has_kana(text))
            .into_iter()
            .filter_map(|hit| match hit.found {
                Found::Money { code, value } => Some((code, value)),
                Found::Measure { .. } => None,
            })
            .collect()
    }

    #[test]
    fn a_measurement_becomes_the_unit_the_reader_uses() {
        assert_eq!(single("The walk is 5 mi long.", "zh-CN").converted, "8.05 km");
        assert_eq!(single("The room is 12 ft wide.", "zh-CN").converted, "3.66 m");
        assert_eq!(single("It weighs 100 lb.", "zh-CN").converted, "45.4 kg");
        assert_eq!(single("A 60 mph wind.", "zh-CN").converted, "96.6 km/h");
        assert_eq!(single("The pool is 25 m.", "en").converted, "82 ft");
        assert_eq!(single("It holds 2 l of water.", "en").converted, "2.11 qt");
    }

    #[test]
    fn temperature_uses_the_offset_and_the_formula() {
        let hot = single("It hit 100 °F today.", "zh-CN");
        assert_eq!(hot.converted, "37.8 °C");
        assert_eq!(hot.rate, "°C = (°F − 32) × 5/9");
        assert_eq!(single("Water boils at 100°C.", "en").converted, "212 °F");
        assert_eq!(single("Water freezes at 273.15 kelvin.", "en").converted, "32 °F");
        assert_eq!(single("The core reaches 6000 kelvin.", "en").converted, "10,340 °F");
    }

    #[test]
    fn a_unit_the_reader_already_uses_is_left_alone() {
        assert!(read("It is 12 m long.", "zh-CN").is_empty());
        assert!(read("It is 12 km long.", "zh-CN").is_empty());
        assert!(read("It is 100°C outside.", "zh-CN").is_empty());
        assert!(read("It is 12 feet long.", "en").is_empty());
    }

    #[test]
    fn a_longer_unit_alias_wins() {
        let speed = single("It travels at 30 m/s.", "en");
        assert_eq!(speed.converted, "67.1 mph");
        assert_eq!(single("The bag holds 5 kg.", "en").converted, "11 lb");
        assert_eq!(single("It is 3 km away.", "en").converted, "1.86 mi");
    }

    #[test]
    fn words_that_only_start_with_a_unit_are_not_units() {
        assert!(read("Wait 5 minutes.", "zh-CN").is_empty());
        assert!(read("He sat in 5 inches of water.", "zh-CN").len() == 1);
        assert!(read("The band played 5 gigs.", "zh-CN").is_empty());
        assert!(read("The cost rose 5 percent.", "zh-CN").is_empty());
    }

    #[test]
    fn an_amount_of_money_becomes_the_readers_currency() {
        assert_eq!(money_of("It costs $200.", "zh-CN"), vec![("USD", 200.0)]);
        assert_eq!(money_of("It costs 200 dollars.", "zh-CN"), vec![("USD", 200.0)]);
        assert_eq!(money_of("售价 200 元。", "en"), vec![("CNY", 200.0)]);
        assert_eq!(money_of("It costs €50.", "en"), vec![("EUR", 50.0)]);
        assert_eq!(money_of("通常 3 美元。", "ja"), vec![("USD", 3.0)]);
        assert_eq!(money_of("It costs 200 US dollars.", "zh-CN"), vec![("USD", 200.0)]);
        assert_eq!(money_of("It costs 15 Swiss francs.", "zh-CN"), vec![("CHF", 15.0)]);
    }

    #[test]
    fn the_yen_is_read_from_the_script_around_it() {
        assert_eq!(money_of("これは ¥500 です。", "en"), vec![("JPY", 500.0)]);
        assert_eq!(money_of("这个 ¥500。", "en"), vec![("CNY", 500.0)]);
    }

    #[test]
    fn a_word_next_to_a_number_is_not_a_currency() {
        // `try` is deliberately not a currency code, and `won` needs a number.
        assert!(money_of("Give it 3 tries.", "zh-CN").is_empty());
        assert_eq!(money_of("$12 and 5 usd", "zh-CN").len(), 2);
    }

    #[test]
    fn a_rate_line_explains_the_switch() {
        assert_eq!(single("The walk is 5 mi long.", "zh-CN").rate, "1 mi = 1.609344 km");
        assert_eq!(single("The room is 12 ft wide.", "zh-CN").rate, "1 ft = 0.3048 m");
        assert_eq!(single("It weighs 100 lb.", "zh-CN").rate, "1 lb = 0.4535924 kg");
    }

    #[test]
    fn a_switch_is_a_whole_number_a_person_would_say() {
        assert_eq!(significant(8.04672, 3), "8.05");
        assert_eq!(significant(804.672, 3), "805");
        assert_eq!(significant(0.0176, 3), "0.0176");
        assert_eq!(significant(37.7778, 3), "37.8");
        assert_eq!(significant(212.0, 3), "212");
        assert_eq!(significant(0.0, 3), "0");
        assert_eq!(significant(1.609344, 7), "1.609344");
    }

    #[test]
    fn money_is_written_the_way_its_currency_is() {
        let dollars = currency_of("USD").expect("USD");
        assert_eq!(format_amount(dollars, 1234.5), "$1,234.50");
        let yen = currency_of("JPY").expect("JPY");
        assert_eq!(format_amount(yen, 1234.56), "JP¥1,235");
        assert_eq!(group_thousands("-1234567"), "-1,234,567");
        assert_eq!(group_thousands("12"), "12");
    }

    #[test]
    fn a_live_rate_becomes_a_conversion_with_its_source() {
        let rate = currency::Rate {
            value: 7.1234,
            source: "exchangerate-api.com",
            date: Some("2026-01-31".to_string()),
            stale: false,
        };
        let hits = scan("It costs $200.", false);
        let hit = hits.first().expect("one hit");
        let conversion = money(hit, "USD", 200.0, "CNY", &rate).expect("a conversion");
        assert_eq!(conversion.category, "currency");
        assert_eq!(conversion.original, "$200");
        assert_eq!(conversion.converted, "¥1,424.68");
        assert_eq!(conversion.rate, "1 USD = 7.1234 CNY");
        assert_eq!(conversion.rate_source.as_deref(), Some("exchangerate-api.com"));
        assert_eq!(conversion.rate_date.as_deref(), Some("2026-01-31"));
        assert!(!conversion.stale);
    }

    #[test]
    fn the_readers_own_currency_is_left_alone() {
        let rate = currency::Rate {
            value: 1.0,
            source: "exchangerate-api.com",
            date: None,
            stale: false,
        };
        let hits = scan("It costs $200.", false);
        let hit = hits.first().expect("one hit");
        // CNY is not a target of the USD table, and equal currencies never
        // reach this far, so the lookup simply finds nothing.
        assert!(money(hit, "USD", 200.0, "USD", &rate).is_none());
    }

    #[test]
    fn a_sentence_with_several_units_keeps_them_in_order() {
        let found = read("The room is 12 ft by 10 ft and the bed is 5 lb.", "zh-CN");
        let originals: Vec<&str> = found.iter().map(|entry| entry.original.as_str()).collect();
        assert_eq!(originals, vec!["12 ft", "10 ft", "5 lb"]);
    }
}
