//! Models related to Uptime Kuma docker hosts

use crate::deserialize::DeserializeNumberLenient;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub enum DockerConnectionType {
    #[serde(rename = "socket")]
    Socket,
    #[serde(rename = "tcp")]
    Tcp,
}

/// Represents a docker host in Uptime Kuma.
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct DockerHost {
    /// The unique identifier for the docker host.
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    /// The name of the docker host.
    #[serde(rename = "name")]
    pub name: Option<String>,

    /// The connection type.
    #[serde(rename = "dockerType")]
    #[serde(alias = "connection_type")]
    pub connection_type: Option<DockerConnectionType>,

    /// The docker host. Depending on the connection type, this could be a uri or a path to a socket.
    #[serde(rename = "dockerDaemon")]
    #[serde(alias = "host")]
    #[serde(alias = "path")]
    pub host: Option<String>,

    /// The user identifier associated with the docker host.
    #[serde(rename = "userId")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub user_id: Option<i32>,
}

impl DockerHost {
    pub fn new() -> Self {
        Default::default()
    }
}

/// A list of docker hosts.
pub type DockerHostList = Vec<DockerHost>;
