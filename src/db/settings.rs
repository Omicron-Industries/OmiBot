use crate::BotState;
use crate::db::DbId;
use crate::settings::GuildSettings;
use serenity::all::GuildId;

pub async fn set_prefix(gid: GuildId, prefix: char, state: &BotState) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"
        INSERT INTO guilds_settings (guild_id, prefix)
        VALUES ($1, $2)
        ON CONFLICT (guild_id)
        DO UPDATE SET prefix = EXCLUDED.prefix
        "#,
        gid.db_id(),
        prefix.to_string(),
    )
    .execute(&state.db_pool)
    .await?;
    let success = res.rows_affected() > 0;

    if success {
        state
            .guild_cache
            .settings
            .insert(
                gid,
                GuildSettings {
                    prefix: prefix.to_string(),
                },
            )
            .await
    }

    Ok(success)
}
