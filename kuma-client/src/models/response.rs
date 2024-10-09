use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LoginResponse {
    Normal {
        #[serde(rename = "ok")]
        ok: bool,
        #[serde(rename = "msg")]
        msg: Option<String>,
        #[serde(rename = "token")]
        token: Option<String>,
    },
    TokenRequired {
        #[serde(rename = "tokenRequired")]
        token_required: bool,
    },
}
