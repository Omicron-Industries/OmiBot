use crate::commands::CommandContext;
use crate::db::permissions::admin_permissions;
use serenity::all::Permissions;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    ManageTags = 0b00000001,
    ManageDetect = 0b00000010,
    ManageSettings = 0b00000100,
    ManageAdmins = 0b00001000,
}

impl TryFrom<i32> for Permission {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0b00000001 => Ok(Permission::ManageTags),
            0b00000010 => Ok(Permission::ManageDetect),
            0b00000100 => Ok(Permission::ManageSettings),
            0b00001000 => Ok(Permission::ManageAdmins),
            _ => Err(()),
        }
    }
}

impl Permission {
    pub fn name(self) -> &'static str {
        match self {
            Permission::ManageTags => "manage_tags",
            Permission::ManageDetect => "manage_detect",
            Permission::ManageSettings => "manage_settings",
            Permission::ManageAdmins => "manage_admins",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "manage_tags" => Some(Permission::ManageTags),
            "manage_detect" => Some(Permission::ManageDetect),
            "manage_settings" => Some(Permission::ManageSettings),
            "manage_admins" => Some(Permission::ManageAdmins),
            _ => None,
        }
    }

    pub fn value(self) -> i32 {
        self as i32
    }

    pub fn from_value(value: i32) -> Vec<Self> {
        let mut permissions = Vec::new();
        for i in 0..32 {
            if (value & (1 << i)) != 0 {
                if let Ok(perm) = Self::try_from(1 << i) {
                    permissions.push(perm);
                }
            }
        }
        permissions
    }
}

pub async fn get_admin_action_msg(
    ctx: &CommandContext,
    perm_required: Permission,
) -> Option<String> {
    let guild = match ctx.get_guild_id().to_guild_cached(&ctx.serenity_ctx.cache) {
        Some(guild) => guild.clone(),
        None => return Some("Failed to get guild.".to_string()),
    };

    let member = match guild
        .member(&ctx.serenity_ctx.http, ctx.get_author_id())
        .await
    {
        Ok(member) => member,
        Err(e) => return Some(format!("Failed to get member: {:?}", e)),
    };

    let channel = match ctx.msg.channel(&ctx.serenity_ctx.http).await {
        Ok(channel) => match channel.guild() {
            Some(channel) => channel,
            None => return Some("Command was not run in a guild channel.".to_string()),
        },
        Err(e) => return Some(format!("Failed to get channel: {:?}", e)),
    };

    let permissions = guild.user_permissions_in(&channel, &member);

    if !permissions.contains(Permissions::ADMINISTRATOR) {
        return match admin_permissions(ctx.get_guild_id(), ctx.get_author_id(), &ctx.state.db_pool)
            .await
        {
            Err(e) => Some(format!("Failed to retrieve admins status: {:?}", e)),
            Ok(perms) => {
                if perms.contains(&perm_required) {
                    None
                } else {
                    Some(format!(
                        "This command requires admin permissions ({}) to execute!",
                        perm_required.name()
                    ))
                }
            }
        };
    }
    None
}
