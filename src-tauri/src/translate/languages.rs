//! Which languages each translation channel can be asked for.
//!
//! Both language bars offer exactly what the chosen channel translates, so a
//! language a channel would refuse is never picked in the first place. This
//! module is the only place that knows; the windows ask for the tables
//! (`service_languages`) instead of keeping a copy of their own, and a
//! translation that asks for an unserved language anyway is clamped here.

use serde::Serialize;

use crate::settings::Service;

use super::normalize_lang_code;

/// Every language the bars offer, in the order they list them.
pub const ALL: [&str; 31] = [
    "en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "pt", "it", "ru", "uk", "nl", "pl", "tr",
    "ar", "hi", "th", "vi", "id", "ms", "cs", "da", "fi", "el", "he", "hu", "no", "ro", "sk", "sv",
];

/// Baidu's 常见语种列表: what a standard account of the 通用文本翻译 API takes.
///
/// The rest of Baidu's 201 languages answer with error `58001` unless the
/// account is an enterprise 尊享版, so they are left out rather than offered and
/// then refused.
const BAIDU: [&str; 23] = [
    "en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "pt", "it", "ru", "nl", "pl", "ar", "th",
    "vi", "cs", "da", "fi", "el", "hu", "ro", "sv",
];

/// 有道智云 文本翻译, whose list covers every language Glossy offers.
const YOUDAO: [&str; 31] = ALL;

/// The languages one channel translates, in menu order.
pub fn served(service: Service) -> &'static [&'static str] {
    match service {
        Service::CloudBaidu => &BAIDU,
        Service::CloudYoudao => &YOUDAO,
        Service::Google => &ALL,
    }
}

/// Whether the channel translates one language code.
pub fn serves(service: Service, code: &str) -> bool {
    let code = normalize_lang_code(code);
    served(service).contains(&code.as_str())
}

/// What to translate into when the channel does not translate the language the
/// settings name. Every channel takes the first entry of its table.
pub fn fallback(service: Service) -> &'static str {
    served(service).first().copied().unwrap_or("en")
}

/// One channel and the languages it translates, as the windows read them.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLanguages {
    pub id: &'static str,
    pub languages: Vec<&'static str>,
}

/// The whole table, in the order the dropdown lists the channels.
pub fn table() -> Vec<ServiceLanguages> {
    Service::ALL
        .iter()
        .copied()
        .map(|service| ServiceLanguages {
            id: service.id(),
            languages: served(service).to_vec(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tables are read as menus, so their order has to be the menu's order.
    fn is_in_menu_order(codes: &[&str]) -> bool {
        let mut last = -1i32;
        codes.iter().all(|code| {
            let at = ALL
                .iter()
                .position(|candidate| candidate == code)
                .map(|index| index as i32)
                .unwrap_or(-1);
            let in_order = at > last;
            last = at;
            in_order
        })
    }

    #[test]
    fn every_table_only_names_languages_of_the_menu() {
        for service in Service::ALL {
            assert!(is_in_menu_order(served(service)), "{:?}", service);
            assert!(served(service).contains(&fallback(service)));
        }
    }

    #[test]
    fn every_channel_translates_english_and_its_own_fallback() {
        for service in Service::ALL {
            assert!(serves(service, "en"), "{:?}", service);
            assert!(serves(service, fallback(service)), "{:?}", service);
        }
    }

    #[test]
    fn youdao_and_google_take_every_language_of_the_menu() {
        for service in [Service::CloudYoudao, Service::Google] {
            for code in ALL {
                assert!(serves(service, code), "{:?} {code}", service);
            }
        }
    }

    #[test]
    fn baidu_leaves_out_the_languages_it_only_gives_the_enterprise_plan() {
        for code in ["uk", "tr", "hi", "id", "ms", "he", "no", "sk"] {
            assert!(!serves(Service::CloudBaidu, code), "{code}");
        }
        for code in [
            "en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "ru", "vi",
        ] {
            assert!(serves(Service::CloudBaidu, code), "{code}");
        }
    }

    #[test]
    fn codes_are_compared_the_way_the_providers_write_them() {
        assert!(serves(Service::CloudBaidu, "zh"));
        assert!(serves(Service::CloudBaidu, "cht"));
        assert!(serves(Service::CloudBaidu, "zh_TW"));
        assert!(serves(Service::CloudBaidu, "JP"));
        // A language the menu never had is not served by anybody.
        assert!(!serves(Service::Google, "ceb"));
    }

    #[test]
    fn the_table_lists_every_channel() {
        assert_eq!(table().len(), Service::ALL.len());
        assert_eq!(table()[0].id, "cloud-baidu");
    }
}
