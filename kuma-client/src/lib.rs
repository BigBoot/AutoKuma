#![doc(html_logo_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]
#![doc(html_favicon_url = "https://cdn.jsdelivr.net/gh/BigBoot/AutoKuma@master/icon.svg")]

#[doc(hidden)]
pub mod build {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
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
