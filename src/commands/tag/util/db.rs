use crate::commands::tag::util::FetchTagModel;
use crate::commands::tag::util::TagKind;
use crate::BotState;
use std::sync::Arc;

pub async fn fetch_tag_resolved(
    name: &str,
    gid: i64,
    state: Arc<BotState>,
) -> Result<Option<FetchTagModel>, sqlx::Error> {
    sqlx::query_as!(
        FetchTagModel,
        r#"
        SELECT
            COALESCE(target.id, source.id) AS "id!",
            COALESCE(target.guild_id, source.guild_id) AS "guild_id!",
            COALESCE(target.owner_id, source.owner_id) AS "owner_id!",
            COALESCE(target.name, source.name) AS "name!",
            COALESCE(target.kind, source.kind) AS "kind!: TagKind",
            COALESCE(target.payload, source.payload) AS "payload!",
            COALESCE(target.t_created, source.t_created) AS "t_created!",
            COALESCE(target.t_updated, source.t_updated) AS "t_updated!",
            COALESCE(target.enabled, source.enabled) AS "enabled!",
            null as alias_target_name
        FROM tags AS source
        LEFT JOIN tags AS target
            ON source.kind = 'alias'
           AND target.id = (source.payload->>'target_id')::int
        WHERE source.guild_id = $1
          AND source.name = $2;
        "#,
        gid,
        name
    )
    .fetch_optional(&state.db_pool)
    .await
}
