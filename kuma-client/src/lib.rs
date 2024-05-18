#![doc(html_logo_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]
#![doc(html_favicon_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod build {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));

    pub const SHORT_VERSION: &str = const_str::format!(
        "{}{}",
        LAST_TAG,
        if const_str::equal!(TAG, "") {
            const_str::format!(
                "-{}{}",
                SHORT_COMMIT,
                if !GIT_CLEAN { "-dirty" } else { "" }
            )
        } else {
            ""
        }
    );
    pub const LONG_VERSION: &str = const_str::format!(
        r#"{}
branch: {}
commit_hash: {} 
build_time: {}
build_env: {}, {}"#,
        SHORT_VERSION,
        BRANCH,
        SHORT_COMMIT,
        BUILD_TIME,
        RUST_VERSION,
        RUST_CHANNEL
    );
}

pub(crate) mod deserialize;

#[doc(hidden)]
pub mod client;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod models;
#[doc(hidden)]
pub mod util;

#[doc(inline)]
pub use client::*;
#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use models::*;
pub use url::Url;
