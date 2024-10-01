use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub enum Name {
    Monitor(String),
    Notification(String),
    DockerHost(String),
    Tag(String),
}

impl Name {
    pub fn name(&self) -> &str {
        match self {
            Name::Monitor(name) => name,
            Name::Notification(name) => name,
            Name::DockerHost(name) => name,
            Name::Tag(name) => name,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Name::Monitor(_) => "monitor",
            Name::Notification(_) => "notification",
            Name::DockerHost(_) => "docker host",
            Name::Tag(_) => "tag",
        }
    }
}
