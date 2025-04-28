use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub enum Name {
    Monitor(String),
    Notification(String),
    DockerHost(String),
    Tag(String),
    StatusPage(String),
}

impl Name {
    pub fn name(&self) -> &str {
        match self {
            Name::Monitor(name) => name,
            Name::Notification(name) => name,
            Name::DockerHost(name) => name,
            Name::Tag(name) => name,
            Name::StatusPage(name) => name,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Name::Monitor(_) => "monitor",
            Name::Notification(_) => "notification",
            Name::DockerHost(_) => "docker host",
            Name::Tag(_) => "tag",
            Name::StatusPage(_) => "status page",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum EntitySelector {
    Monitor(String, i32),
    Notification(String, i32),
    DockerHost(String, i32),
    Tag(String, i32),
    StatusPage(String, String),
}

impl EntitySelector {
    pub fn name(&self) -> &str {
        match self {
            EntitySelector::Monitor(name, _) => name,
            EntitySelector::Notification(name, _) => name,
            EntitySelector::DockerHost(name, _) => name,
            EntitySelector::Tag(name, _) => name,
            EntitySelector::StatusPage(name, _) => name,
        }
    }

    pub fn type_name(&self) -> &'static str {
        Name::from(self.clone()).type_name()
    }
}

impl From<EntitySelector> for Name {
    fn from(selector: EntitySelector) -> Self {
        match selector {
            EntitySelector::Monitor(name, _) => Name::Monitor(name),
            EntitySelector::Notification(name, _) => Name::Notification(name),
            EntitySelector::DockerHost(name, _) => Name::DockerHost(name),
            EntitySelector::Tag(name, _) => Name::Tag(name),
            EntitySelector::StatusPage(name, _) => Name::StatusPage(name),
        }
    }
}
