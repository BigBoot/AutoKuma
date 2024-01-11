use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "ok")]
    pub ok: bool,
    #[serde(rename = "msg")]
    pub msg: Option<String>,
    #[serde(rename = "token")]
    pub token: Option<String>,
}
