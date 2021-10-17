#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(dead_code)]

use std::borrow::Cow;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct Config {
    pub search: _Config__search,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct _Config__search {
    pub api_keys: _Config__search__api_keys,
    pub api_url: Cow<'static, str>,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct _Config__search__api_keys {
    pub private: Cow<'static, str>,
    pub public: Cow<'static, str>,
}

pub const CONFIG: Config = Config {
    search: _Config__search {
        api_keys: _Config__search__api_keys {
            private: Cow::Borrowed("3642d24252d9d9fc5f7850f0c33eda00f55367b548188524331e2c21ac06456e"),
            public: Cow::Borrowed("a0daf23b8db4ff617b7bd476e658595ced4289be8cb4b20253687aee42ad4043"),
        },
        api_url: Cow::Borrowed("https://search.wnelson.dev"),
    },
};
