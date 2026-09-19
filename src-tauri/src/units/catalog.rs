//! The units Glossy knows, the currencies it can ask for, and the units each
//! target language expects to read.
//!
//! Everything is a plain table, so adding a unit is a line of data. Values are
//! mapped onto the base unit of their category with `base = value * scale +
//! offset`; only the temperature units need the offset.

/// What a unit measures. Conversions only ever happen inside one category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Length,
    Mass,
    Volume,
    Speed,
    Area,
    Temperature,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Length => "length",
            Category::Mass => "mass",
            Category::Volume => "volume",
            Category::Speed => "speed",
            Category::Area => "area",
            Category::Temperature => "temperature",
        }
    }
}

/// The two systems a target language reads in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    Metric,
    Imperial,
}

/// One measurement unit, as written by the user and as shown in the card.
#[derive(Debug, Clone, Copy)]
pub struct Unit {
    /// How the unit is written in the card, e.g. `km` or `°C`.
    pub display: &'static str,
    pub category: Category,
    pub system: System,
    /// `base = value * scale + offset`.
    pub scale: f64,
    pub offset: f64,
    /// Spellings that may stand after a number, lower case.
    pub aliases: &'static [&'static str],
}

/// One currency: the amount in a selection is converted with a live rate.
#[derive(Debug, Clone, Copy)]
pub struct Currency {
    /// ISO 4217 code, e.g. `USD`.
    pub code: &'static str,
    /// Symbol written before the amount, e.g. `$`.
    pub symbol: &'static str,
    /// Decimals the amount is shown with, per the currency's own convention.
    pub decimals: u8,
    /// Spellings that may stand before or after a number, lower case.
    pub aliases: &'static [&'static str],
}

/// `(0 - 32) * 5 / 9`, so that 0 °F lands on -17.78 °C.
const FAHRENHEIT_OFFSET: f64 = -160.0 / 9.0;
const FAHRENHEIT_SCALE: f64 = 5.0 / 9.0;

/// Every measurement unit Glossy converts. Multiples of the same physical unit
/// are separate rows on purpose: what matters is the unit the text used and the
/// unit the reader wants.
pub const UNITS: &[Unit] = &[
    // Length, base metre.
    Unit { display: "mm", category: Category::Length, system: System::Metric, scale: 0.001, offset: 0.0,
        aliases: &["mm", "millimetre", "millimetres", "millimeter", "millimeters", "毫米"] },
    Unit { display: "cm", category: Category::Length, system: System::Metric, scale: 0.01, offset: 0.0,
        aliases: &["cm", "centimetre", "centimetres", "centimeter", "centimeters", "厘米", "公分"] },
    Unit { display: "m", category: Category::Length, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["m", "metre", "metres", "meter", "meters", "米"] },
    Unit { display: "km", category: Category::Length, system: System::Metric, scale: 1000.0, offset: 0.0,
        aliases: &["km", "kilometre", "kilometres", "kilometer", "kilometers", "公里", "千米"] },
    // `in` on its own is a preposition, so only the spelled out forms and the
    // unambiguous `inch`/`inches` are accepted.
    Unit { display: "in", category: Category::Length, system: System::Imperial, scale: 0.0254, offset: 0.0,
        aliases: &["inch", "inches", "英寸"] },
    Unit { display: "ft", category: Category::Length, system: System::Imperial, scale: 0.3048, offset: 0.0,
        aliases: &["ft", "foot", "feet", "英尺"] },
    Unit { display: "yd", category: Category::Length, system: System::Imperial, scale: 0.9144, offset: 0.0,
        aliases: &["yd", "yds", "yard", "yards", "码"] },
    Unit { display: "mi", category: Category::Length, system: System::Imperial, scale: 1609.344, offset: 0.0,
        aliases: &["mi", "mile", "miles", "英里"] },

    // Mass, base kilogram.
    Unit { display: "mg", category: Category::Mass, system: System::Metric, scale: 0.000001, offset: 0.0,
        aliases: &["mg", "milligram", "milligrams", "毫克"] },
    Unit { display: "g", category: Category::Mass, system: System::Metric, scale: 0.001, offset: 0.0,
        aliases: &["g", "gram", "grams", "gramme", "grammes", "克"] },
    Unit { display: "kg", category: Category::Mass, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["kg", "kgs", "kilogram", "kilograms", "kilo", "kilos", "公斤", "千克"] },
    Unit { display: "t", category: Category::Mass, system: System::Metric, scale: 1000.0, offset: 0.0,
        aliases: &["tonne", "tonnes", "metric ton", "metric tons", "公吨", "吨"] },
    // An English `ton` is the 2000 pound short ton, which is what a reader of an
    // English text means by it.
    Unit { display: "tn", category: Category::Mass, system: System::Imperial, scale: 907.18474, offset: 0.0,
        aliases: &["ton", "tons", "short ton", "short tons"] },
    Unit { display: "oz", category: Category::Mass, system: System::Imperial, scale: 0.028349523125, offset: 0.0,
        aliases: &["oz", "ounce", "ounces", "盎司"] },
    Unit { display: "lb", category: Category::Mass, system: System::Imperial, scale: 0.45359237, offset: 0.0,
        aliases: &["lb", "lbs", "pound", "pounds", "磅"] },
    Unit { display: "st", category: Category::Mass, system: System::Imperial, scale: 6.35029318, offset: 0.0,
        aliases: &["st", "stone", "stones", "英石"] },

    // Volume, base litre.
    Unit { display: "ml", category: Category::Volume, system: System::Metric, scale: 0.001, offset: 0.0,
        aliases: &["ml", "millilitre", "millilitres", "milliliter", "milliliters", "毫升"] },
    Unit { display: "l", category: Category::Volume, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["l", "litre", "litres", "liter", "liters", "升"] },
    Unit { display: "fl oz", category: Category::Volume, system: System::Imperial, scale: 0.0295735295625, offset: 0.0,
        aliases: &["fl oz", "fl. oz", "fluid ounce", "fluid ounces", "液量盎司"] },
    Unit { display: "cup", category: Category::Volume, system: System::Imperial, scale: 0.2365882365, offset: 0.0,
        aliases: &["cup", "cups", "杯"] },
    Unit { display: "pt", category: Category::Volume, system: System::Imperial, scale: 0.473176473, offset: 0.0,
        aliases: &["pt", "pint", "pints", "品脱"] },
    Unit { display: "qt", category: Category::Volume, system: System::Imperial, scale: 0.946352946, offset: 0.0,
        aliases: &["qt", "quart", "quarts", "夸脱"] },
    Unit { display: "gal", category: Category::Volume, system: System::Imperial, scale: 3.785411784, offset: 0.0,
        aliases: &["gal", "gallon", "gallons", "加仑"] },

    // Speed, base metre per second.
    Unit { display: "km/h", category: Category::Speed, system: System::Metric, scale: 0.2777777777777778, offset: 0.0,
        aliases: &["km/h", "kmh", "kph", "kmph", "kilometres per hour", "kilometers per hour", "公里每小时", "千米每小时"] },
    Unit { display: "m/s", category: Category::Speed, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["m/s"] },
    Unit { display: "mph", category: Category::Speed, system: System::Imperial, scale: 0.44704, offset: 0.0,
        aliases: &["mph", "miles per hour", "英里每小时"] },

    // Area, base square metre.
    Unit { display: "cm²", category: Category::Area, system: System::Metric, scale: 0.0001, offset: 0.0,
        aliases: &["cm2", "cm²", "square centimetre", "square centimetres", "square centimeter", "square centimeters", "平方厘米"] },
    Unit { display: "m²", category: Category::Area, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["m2", "m²", "square metre", "square metres", "square meter", "square meters", "平方米"] },
    Unit { display: "ha", category: Category::Area, system: System::Metric, scale: 10000.0, offset: 0.0,
        aliases: &["ha", "hectare", "hectares", "公顷"] },
    Unit { display: "km²", category: Category::Area, system: System::Metric, scale: 1000000.0, offset: 0.0,
        aliases: &["km2", "km²", "square kilometre", "square kilometres", "square kilometer", "square kilometers", "平方公里", "平方千米"] },
    Unit { display: "in²", category: Category::Area, system: System::Imperial, scale: 0.00064516, offset: 0.0,
        aliases: &["in2", "in²", "square inch", "square inches", "平方英寸"] },
    Unit { display: "ft²", category: Category::Area, system: System::Imperial, scale: 0.09290304, offset: 0.0,
        aliases: &["ft2", "ft²", "sq ft", "sqft", "square foot", "square feet", "平方英尺"] },
    Unit { display: "yd²", category: Category::Area, system: System::Imperial, scale: 0.83612736, offset: 0.0,
        aliases: &["yd2", "yd²", "square yard", "square yards", "平方码"] },
    Unit { display: "acre", category: Category::Area, system: System::Imperial, scale: 4046.8564224, offset: 0.0,
        aliases: &["acre", "acres", "英亩"] },

    // Temperature, base degree Celsius.
    Unit { display: "°C", category: Category::Temperature, system: System::Metric, scale: 1.0, offset: 0.0,
        aliases: &["°c", "℃", "celsius", "centigrade", "摄氏度"] },
    Unit { display: "°F", category: Category::Temperature, system: System::Imperial, scale: FAHRENHEIT_SCALE, offset: FAHRENHEIT_OFFSET,
        // A bare `°` is how American writing shortens Fahrenheit.
        aliases: &["°f", "℉", "fahrenheit", "华氏度", "°"] },
    // Kelvin is metric, but nobody writes it in prose, so it is always shown in
    // the unit the target language reads.
    Unit { display: "K", category: Category::Temperature, system: System::Metric, scale: 1.0, offset: -273.15,
        // No bare `K`: `5K` is a five kilometre run and `5k` is five thousand,
        // neither of which is a temperature.
        aliases: &["kelvin", "开尔文"] },
];

/// The currencies a rate can be asked for.
pub const CURRENCIES: &[Currency] = &[
    Currency { code: "USD", symbol: "$", decimals: 2, aliases: &["$", "us$", "usd", "dollar", "dollars", "us dollar", "美元", "美金"] },
    Currency { code: "EUR", symbol: "€", decimals: 2, aliases: &["€", "eur", "euro", "euros", "欧元"] },
    Currency { code: "GBP", symbol: "£", decimals: 2, aliases: &["£", "gbp", "pound sterling", "quid", "英镑"] },
    Currency { code: "CNY", symbol: "¥", decimals: 2, aliases: &["¥", "￥", "cny", "rmb", "yuan", "元", "人民币", "块"] },
    // `¥` alone is claimed by CNY; a text written with kana is read as JPY.
    Currency { code: "JPY", symbol: "JP¥", decimals: 0, aliases: &["jp¥", "jpy", "yen", "日元", "日圆", "円"] },
    Currency { code: "KRW", symbol: "₩", decimals: 0, aliases: &["₩", "krw", "won", "韩元"] },
    Currency { code: "HKD", symbol: "HK$", decimals: 2, aliases: &["hk$", "hkd", "港元", "港币"] },
    Currency { code: "TWD", symbol: "NT$", decimals: 2, aliases: &["nt$", "twd", "新台币", "台币"] },
    Currency { code: "AUD", symbol: "A$", decimals: 2, aliases: &["a$", "aud", "澳元", "澳币"] },
    Currency { code: "CAD", symbol: "C$", decimals: 2, aliases: &["c$", "cad", "加元"] },
    Currency { code: "CHF", symbol: "CHF", decimals: 2, aliases: &["chf", "swiss franc", "swiss francs", "瑞士法郎"] },
    Currency { code: "SGD", symbol: "S$", decimals: 2, aliases: &["s$", "sgd", "新加坡元", "新元"] },
    Currency { code: "NZD", symbol: "NZ$", decimals: 2, aliases: &["nz$", "nzd", "新西兰元"] },
    Currency { code: "RUB", symbol: "₽", decimals: 2, aliases: &["₽", "rub", "ruble", "rubles", "rouble", "卢布"] },
    Currency { code: "UAH", symbol: "₴", decimals: 2, aliases: &["₴", "uah", "hryvnia", "格里夫纳"] },
    Currency { code: "TRY", symbol: "₺", decimals: 2, aliases: &["₺", "turkish lira", "里拉"] },
    Currency { code: "INR", symbol: "₹", decimals: 2, aliases: &["₹", "inr", "rupee", "rupees", "卢比"] },
    Currency { code: "THB", symbol: "฿", decimals: 2, aliases: &["฿", "thb", "baht", "泰铢"] },
    Currency { code: "VND", symbol: "₫", decimals: 0, aliases: &["₫", "vnd", "dong", "越南盾"] },
    Currency { code: "IDR", symbol: "Rp", decimals: 0, aliases: &["idr", "rupiah", "印尼盾", "印尼卢比"] },
    Currency { code: "MYR", symbol: "RM", decimals: 2, aliases: &["myr", "ringgit", "令吉", "马币"] },
    Currency { code: "PHP", symbol: "₱", decimals: 2, aliases: &["₱", "php", "peso", "pesos", "比索"] },
    Currency { code: "BRL", symbol: "R$", decimals: 2, aliases: &["r$", "brl", "real", "reais", "雷亚尔"] },
    Currency { code: "MXN", symbol: "MX$", decimals: 2, aliases: &["mxn", "墨西哥比索"] },
    Currency { code: "ZAR", symbol: "ZAR", decimals: 2, aliases: &["zar", "rand", "兰特"] },
    Currency { code: "SEK", symbol: "SEK", decimals: 2, aliases: &["sek", "swedish krona", "瑞典克朗"] },
    Currency { code: "NOK", symbol: "NOK", decimals: 2, aliases: &["nok", "挪威克朗"] },
    Currency { code: "DKK", symbol: "DKK", decimals: 2, aliases: &["dkk", "丹麦克朗"] },
    Currency { code: "PLN", symbol: "PLN", decimals: 2, aliases: &["pln", "zloty", "兹罗提"] },
    Currency { code: "CZK", symbol: "CZK", decimals: 2, aliases: &["czk", "koruna", "捷克克朗"] },
    Currency { code: "HUF", symbol: "HUF", decimals: 0, aliases: &["huf", "forint", "福林"] },
    Currency { code: "RON", symbol: "RON", decimals: 2, aliases: &["ron", "leu", "列伊"] },
    Currency { code: "ILS", symbol: "₪", decimals: 2, aliases: &["₪", "ils", "shekel", "谢克尔"] },
    Currency { code: "SAR", symbol: "SAR", decimals: 2, aliases: &["sar", "riyal", "里亚尔"] },
    Currency { code: "AED", symbol: "AED", decimals: 2, aliases: &["aed", "dirham", "迪拉姆"] },
];

/// The units a target language reads in, smallest first. The card picks the
/// largest one that still reads as a number a person would say out loud.
const PREFERRED: &[(Category, System, &[&str])] = &[
    (Category::Length, System::Metric, &["mm", "cm", "m", "km"]),
    // Yards and short tons are real imperial units, but nobody reaches for them
    // when reading a foreign measurement, so they are only ever a source.
    (Category::Length, System::Imperial, &["in", "ft", "mi"]),
    (Category::Mass, System::Metric, &["mg", "g", "kg", "t"]),
    (Category::Mass, System::Imperial, &["oz", "lb"]),
    (Category::Volume, System::Metric, &["ml", "l"]),
    (Category::Volume, System::Imperial, &["fl oz", "cup", "pt", "qt", "gal"]),
    // A speed is always written the way the reader says it, not scaled.
    (Category::Speed, System::Metric, &["km/h"]),
    (Category::Speed, System::Imperial, &["mph"]),
    (Category::Area, System::Metric, &["cm²", "m²", "ha", "km²"]),
    (Category::Area, System::Imperial, &["in²", "ft²", "acre"]),
    (Category::Temperature, System::Metric, &["°C"]),
    (Category::Temperature, System::Imperial, &["°F"]),
];

/// The unit with this display name.
pub fn unit(display: &str) -> Option<&'static Unit> {
    UNITS.iter().find(|unit| unit.display == display)
}

/// The units a language reads in, smallest first.
pub fn candidates(category: Category, system: System) -> Vec<&'static Unit> {
    PREFERRED
        .iter()
        .find(|(entry, entry_system, _)| *entry == category && *entry_system == system)
        .map(|(_, _, displays)| displays.iter().filter_map(|name| unit(name)).collect())
        .unwrap_or_default()
}

/// The currency with this ISO code.
pub fn currency(code: &str) -> Option<&'static Currency> {
    CURRENCIES
        .iter()
        .find(|currency| currency.code.eq_ignore_ascii_case(code))
}

/// Which system a target language reads lengths, weights and temperatures in.
///
/// Only English is treated as imperial: every other language Glossy offers
/// writes metric units.
pub fn target_system(language: &str) -> System {
    if primary(language) == "en" {
        System::Imperial
    } else {
        System::Metric
    }
}

/// The currency a reader of this language expects to see a foreign amount in.
pub fn target_currency(language: &str) -> Option<&'static str> {
    let code = match language.trim().to_ascii_lowercase().replace('_', "-").as_str() {
        "zh" | "zh-cn" | "zh-hans" => "CNY",
        "zh-tw" | "zh-hk" | "zh-hant" => "TWD",
        "en" => "USD",
        "ja" => "JPY",
        "ko" => "KRW",
        "fr" | "de" | "es" | "it" | "nl" | "el" | "fi" | "sk" | "pt" => "EUR",
        "ru" => "RUB",
        "uk" => "UAH",
        "tr" => "TRY",
        "hi" => "INR",
        "th" => "THB",
        "vi" => "VND",
        "id" => "IDR",
        "ms" => "MYR",
        "pl" => "PLN",
        "cs" => "CZK",
        "da" => "DKK",
        "sv" => "SEK",
        "no" => "NOK",
        "hu" => "HUF",
        "ro" => "RON",
        "he" => "ILS",
        "ar" => "SAR",
        _ => return None,
    };
    Some(code)
}

/// The language without its region, lower case.
fn primary(language: &str) -> String {
    language
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .split('-')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preferred_unit_exists() {
        for (category, system, displays) in PREFERRED {
            assert!(!displays.is_empty());
            for display in *displays {
                let unit = unit(display).unwrap_or_else(|| panic!("{display} is not a known unit"));
                assert_eq!(unit.category, *category, "{display} measures the wrong thing");
                assert_eq!(unit.system, *system, "{display} is in the wrong system");
            }
        }
    }

    #[test]
    fn preferred_units_are_ordered_from_small_to_large() {
        for (category, system, displays) in PREFERRED {
            let scales: Vec<f64> = displays
                .iter()
                .map(|display| unit(display).expect("known unit").scale)
                .collect();
            for pair in scales.windows(2) {
                assert!(pair[0] < pair[1], "{category:?}/{system:?} is out of order: {scales:?}");
            }
        }
    }

    #[test]
    fn aliases_are_lower_case_and_unique() {
        let mut seen: Vec<&str> = Vec::new();
        for unit in UNITS {
            assert!(!unit.aliases.is_empty(), "{} has no alias", unit.display);
            for alias in unit.aliases {
                assert_eq!(*alias, alias.to_lowercase(), "`{alias}` of {} is not lower case", unit.display);
                assert!(!seen.contains(alias), "`{alias}` is claimed twice");
                seen.push(alias);
            }
        }
        for currency in CURRENCIES {
            for alias in currency.aliases {
                assert_eq!(*alias, alias.to_lowercase(), "`{alias}` is not lower case");
                assert!(!seen.contains(alias), "`{alias}` is both a unit and a currency");
                seen.push(alias);
            }
        }
    }

    #[test]
    fn a_language_gets_the_units_and_the_currency_its_readers_use() {
        assert_eq!(target_system("en"), System::Imperial);
        assert_eq!(target_system("EN-US"), System::Imperial);
        assert_eq!(target_system("zh-CN"), System::Metric);
        assert_eq!(target_system("ja"), System::Metric);
        assert_eq!(target_currency("zh-CN"), Some("CNY"));
        assert_eq!(target_currency("zh_TW"), Some("TWD"));
        assert_eq!(target_currency("en"), Some("USD"));
        assert_eq!(target_currency("de"), Some("EUR"));
        assert_eq!(target_currency("fil"), None);
    }

    #[test]
    fn a_currency_is_found_by_its_code() {
        assert_eq!(currency("cny").map(|entry| entry.code), Some("CNY"));
        assert_eq!(currency("USD").map(|entry| entry.decimals), Some(2));
        assert_eq!(currency("jpy").map(|entry| entry.decimals), Some(0));
        assert!(currency("xyz").is_none());
    }
}
