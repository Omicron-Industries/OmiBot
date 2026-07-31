DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_type
        WHERE typname = 'tag_kind'
    ) THEN
CREATE TYPE tag_kind AS ENUM ('text', 'alias', 'embed', 'script');
END IF;
END
$$;

create table if not exists admins
(
    id          integer generated always as identity
    constraint admins_pk
    primary key,
    guild_id    bigint                                             not null,
    member_id   bigint                                             not null,
    t_created   timestamp with time zone default CURRENT_TIMESTAMP not null,
    permissions bigint                                             not null,
    constraint admins_uk
    unique (guild_id, member_id)
    );

comment on table admins is 'Users that have privileges over the bot';

create table if not exists tags
(
    id        integer generated always as identity
    constraint tags_pk
    primary key,
    guild_id  bigint                                             not null,
    owner_id  bigint                                             not null,
    name      varchar(32)                                        not null
    constraint check_name
    check ((name)::text ~ '^[a-z0-9_-]+$'::text),
    kind      tag_kind                                           not null,
    payload   jsonb                                              not null,
    t_created timestamp with time zone default CURRENT_TIMESTAMP not null,
    t_updated timestamp with time zone default CURRENT_TIMESTAMP not null,
                            enabled   boolean                  default true              not null,
                            target_id integer generated always as (
                            CASE
                            WHEN (kind = 'alias'::tag_kind) THEN ((payload ->> 'target_id'::text))::integer
    ELSE NULL::integer
    END) stored
    constraint tags_tags_id_fk
    references tags
                        on delete cascade,
    constraint tags_uk
    unique (guild_id, name),
    constraint check_payload
    check (
              CASE kind
              WHEN 'alias'::tag_kind THEN (payload ? 'target_id'::text)
    WHEN 'text'::tag_kind THEN (payload ? 'content'::text)
    WHEN 'script'::tag_kind THEN (payload ? 'script'::text)
    WHEN 'embed'::tag_kind THEN (payload ? 'embed'::text)
    ELSE false
    END)
    );

comment on column tags.kind is 'text, script, alias, embed';

create table if not exists guilds_settings
(
    guild_id bigint                   not null
    constraint guilds_settings_pk
    primary key,
    prefix   char default '%'::bpchar not null
);

create table if not exists bans
(
    id        integer generated always as identity
    constraint bans_pk
    primary key,
    guild_id  bigint                                             not null,
    user_id   bigint                                             not null,
    banned_by bigint                                             not null,
    t_banned  timestamp with time zone default CURRENT_TIMESTAMP not null,
    constraint bans_uk
    unique (user_id, guild_id)
    );

comment on table bans is 'user bans';

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS tags_name_trgm_idx
    ON tags USING gin (name gin_trgm_ops);

