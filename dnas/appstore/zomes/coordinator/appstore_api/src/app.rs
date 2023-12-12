use crate::{AppResult, ANCHOR_AGENTS, ANCHOR_APPS, ANCHOR_PUBLISHERS};
use appstore::{
    AppEntry, DeprecationNotice, EntityId, GetEntityInput, LinkTypes, UpdateEntityInput,
};
use hc_crud::{create_entity, get_entity, now, update_entity, Entity};
use hdk::prelude::*;

#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub icon_src: String,
    pub publisher: EntityId,
    pub source: String,
    pub hashes: String,
    pub changelog: Option<String>,
    pub metadata: Option<String>,

    // optional
    pub editors: Option<Vec<AgentPubKey>>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

pub fn create(mut input: CreateInput) -> AppResult<Entity<AppEntry>> {
    debug!("Creating App: {}", input.title);
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;
    let default_editors = vec![pubkey.clone()];

    if let Some(ref mut editors) = input.editors {
        if !editors.contains(&pubkey) {
            editors.splice(0..0, default_editors.clone());
        }
    }

    let app = AppEntry {
        title: input.title,
        subtitle: input.subtitle,
        description: input.description,
        icon_src: input.icon_src,
        publisher: input.publisher.clone(),
        source: input.source,
        hashes: input.hashes,
        changelog: input.changelog,
        metadata: input.metadata,

        editors: input.editors.unwrap_or(default_editors),

        author: pubkey,
        published_at: input.published_at.unwrap_or(default_now),
        last_updated: input.last_updated.unwrap_or(default_now),

        deprecation: None,
    };
    let entity = create_entity(&app)?;

    {
        // Path via Agent's Apps
        for agent in entity.content.editors.iter() {
            let (_, pathhash) = hc_utils::path(
                ANCHOR_AGENTS,
                vec![
                    // hc_utils::agentid()?,
                    agent.to_string(),
                    ANCHOR_APPS.to_string(),
                ],
            );
            entity.link_from(&pathhash, LinkTypes::App, None)?;
        }
    }
    {
        // Path via Publisher's Apps
        let (_, pathhash) = hc_utils::path(
            ANCHOR_PUBLISHERS,
            vec![input.publisher.to_string(), ANCHOR_APPS.to_string()],
        );
        entity.link_from(&pathhash, LinkTypes::App, None)?;
    }
    {
        // Path via All Apps
        let (_, pathhash) = hc_utils::path_base(ANCHOR_APPS);
        entity.link_from(&pathhash, LinkTypes::App, None)?;
    }

    Ok(entity)
}

pub fn get(input: GetEntityInput) -> AppResult<Entity<AppEntry>> {
    debug!("Get app: {}", input.id);
    let entity: Entity<AppEntry> = get_entity(&input.id)?;

    Ok(entity)
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon_src: Option<String>,
    pub source: Option<String>,
    pub hashes: Option<String>,
    pub metadata: Option<String>,
    pub editors: Option<Vec<AgentPubKey>>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

pub fn update(input: UpdateInput) -> AppResult<Entity<AppEntry>> {
    debug!("Updating App: {}", input.base);
    let props = input.properties.clone();
    let mut previous: Option<AppEntry> = None;

    let entity = update_entity(&input.base, |mut current: AppEntry, _| {
        previous = Some(current.clone());

        current.title = props.title.unwrap_or(current.title);
        current.subtitle = props.subtitle.unwrap_or(current.subtitle);
        current.description = props.description.unwrap_or(current.description);
        current.source = props.source.unwrap_or(current.source);
        current.hashes = props.hashes.unwrap_or(current.hashes);
        current.metadata = props.metadata;
        current.icon_src = props.icon_src.unwrap_or(current.icon_src);
        current.published_at = props.published_at.unwrap_or(current.published_at);
        current.last_updated = props.last_updated.unwrap_or(current.last_updated);

        Ok(current)
    })?;

    // let previous = previous.unwrap();

    Ok(entity)
}

#[derive(Debug, Deserialize)]
pub struct DeprecateInput {
    pub base: ActionHash,
    pub message: String,
}

pub fn deprecate(input: DeprecateInput) -> AppResult<Entity<AppEntry>> {
    debug!("Deprecating hApp: {}", input.base);
    let entity = update_entity(&input.base, |mut current: AppEntry, _| {
        current.deprecation = Some(DeprecationNotice {
            message: input.message.to_owned(),
            recommended_alternatives: None,
        });

        Ok(current)
    })?;

    Ok(entity)
}
