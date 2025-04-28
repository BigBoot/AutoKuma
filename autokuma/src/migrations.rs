use crate::{app_state::AppState, error::Result, name::Name};
use futures_util::future::BoxFuture;
use itertools::Itertools as _;
use kuma_client::Client;
use log::{error, info};
use seq_macro::seq;

use std::{env, sync::LazyLock};

static MIGRATIONS: LazyLock<
    Vec<for<'a> fn(&'a AppState, &'a Client) -> BoxFuture<'a, Result<()>>>,
> = LazyLock::new(|| {
    let mut migrations: Vec<for<'a> fn(&'a AppState, &'a Client) -> BoxFuture<'a, Result<()>>> =
        vec![];

    seq!(N in 1..=2 {
        migrations.push(|state, client| Box::pin(migrate_v~N(state, client)));
    });

    migrations
});
static CURRENT_VERSION: LazyLock<i32> = LazyLock::new(|| MIGRATIONS.len() as i32);

pub async fn migrate(state: &AppState, kuma: &Client) -> Result<()> {
    loop {
        let version = state.db.get_version()?;

        if version > *CURRENT_VERSION {
            error!("Database version {} is higher than the current version ({}), refusing to continue.", version, *CURRENT_VERSION);
            return Ok(());
        }

        if version < *CURRENT_VERSION {
            info!("Migrating database to version {}", version + 1);
            let migration = MIGRATIONS[version as usize];
            migration(state, kuma).await?;
            state.db.set_version(version + 1)?;
            continue;
        }

        break;
    }

    Ok(())
}

async fn migrate_v1(state: &AppState, kuma: &Client) -> Result<()> {
    let autokuma_tag = kuma
        .get_tags()
        .await?
        .iter()
        .find(|x| {
            x.name
                .as_ref()
                .is_some_and(|name| name == &state.config.tag_name)
        })
        .map(|tag| tag.tag_id)
        .flatten();

    if let Some(autokuma_tag) = autokuma_tag {
        if !env::var("AUTOKUMA__MIGRATE").is_ok_and(|x| x == "true") {
            error!(
                    "Migration required, but AUTOKUMA__MIGRATE is not set to 'true', refusing to continue to avoid data loss. Please read the CHANGELOG and then set AUTOKUMA__MIGRATE=true to continue."
                );
            return Ok(());
        }

        let entries = kuma
            .get_monitors()
            .await?
            .iter()
            .filter_map(|(_, monitor)| {
                monitor
                    .common()
                    .tags()
                    .iter()
                    .find(|x| x.tag_id == Some(autokuma_tag))
                    .map(|tag| tag.value.clone())
                    .flatten()
                    .map(|name| (name, monitor.common().id().unwrap_or(-1)))
            })
            .collect_vec();

        info!("Migrating {} monitors", entries.len());

        for (name, id) in entries {
            state.db.store_id(Name::Monitor(name), id)?;
        }

        kuma.delete_tag(autokuma_tag).await?;
    }

    Ok(())
}

async fn migrate_v2(_state: &AppState, _kuma: &Client) -> Result<()> {
    // No manual migration needed

    Ok(())
}
