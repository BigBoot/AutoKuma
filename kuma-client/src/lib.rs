#![doc(html_logo_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]
#![doc(html_favicon_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]
#![doc = include_str!("../README.md")]

use shadow_rs::shadow;

shadow!(shadow_build);

#[doc(hidden)]
pub mod build {
    pub use super::shadow_build::*;

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

#[cfg(feature = "private-api")]
#[doc(hidden)]
pub mod deserialize;

#[cfg(not(feature = "private-api"))]
#[doc(hidden)]
pub(crate) mod deserialize;

#[cfg(feature = "runtime")]
#[doc(hidden)]
pub mod client;
#[cfg(feature = "runtime")]
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod models;
#[doc(hidden)]
pub mod util;

#[cfg(feature = "runtime")]
#[doc(inline)]
pub use client::*;
#[cfg(feature = "runtime")]
#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use models::*;
pub use url::Url;
