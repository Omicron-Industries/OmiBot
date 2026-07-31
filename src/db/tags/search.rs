use crate::db::DbId;
use serenity::all::GuildId;
use sqlx::PgPool;

pub async fn search_tags(
    gid: GuildId,
    search_term: &str,
    db: &PgPool,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT name
        FROM tags
        WHERE guild_id = $1
          AND enabled = true
          AND name % $2
        ORDER BY similarity(name, $2) DESC
        LIMIT 25
        "#,
        gid.db_id(),
        search_term,
    )
    .fetch_all(db)
    .await
}
