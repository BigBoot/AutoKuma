#![allow(dead_code)]

use kube::CustomResourceExt;

include!("mod.rs");

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&crate::sources::kubernetes_source::KumaEntity::crd()).unwrap()
    )
}
