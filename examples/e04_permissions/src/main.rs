use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::{Channel, ChannelType, PartialChannel},
        guild::Role,
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionResponseType,
        },
        prelude::Ready,
    },
    prelude::{GatewayIntents, Mentionable},
    Client,
};
use slashies::{
    parsable::UserInput, register_commands, ApplicationCommandInteractionHandler, Commands,
    InvocationError,
};
use slashies_macros::{
    ApplicationCommandInteractionHandler, Command, Commands, SubCommand, SubCommandGroup,
};
use std::env::VarError;

/// Get or edit a user or group's permissions
#[derive(Debug, Command, ApplicationCommandInteractionHandler)]
#[name = "permissions"]
enum PermissionsCommand {
    /// Get or edit a user's permissions
    #[name = "user"]
    #[subcommandgroup]
    User(UserSubCommandGroup),

    /// Get or edit a role's permissions
    #[name = "role"]
    #[subcommandgroup]
    Role(RoleSubCommandGroup),
}

#[derive(Debug, SubCommandGroup, ApplicationCommandInteractionHandler)]
enum UserSubCommandGroup {
    /// Edit permissions for a user
    #[name = "edit"]
    Edit(EditPermissionsForUserCommand),
    /// Get permissions for a user
    #[name = "get"]
    Get(GetPermissionsForUserCommand),
}

#[derive(Debug, SubCommandGroup, ApplicationCommandInteractionHandler)]
enum RoleSubCommandGroup {
    /// Edit permissions for a role
    #[name = "edit"]
    Edit(EditPermissionsForRoleCommand),
    /// Get permissions for a role
    #[name = "get"]
    Get(GetPermissionsForRoleCommand),
}

#[derive(Debug, SubCommand)]
struct EditPermissionsForUserCommand {
    /// The user to edit
    pub user: UserInput,

    /// The channel permissions to edit. If omitted, the guild permissions will be edited
    #[channel_types(ChannelType::Text)]
    pub channel: Option<PartialChannel>,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for EditPermissionsForUserCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let response_string = if let Some(ref channel) = self.channel {
            format!(
                "Editing permissions for user {0} in {1}...",
                self.user.user.mention(),
                channel.id.mention(),
            )
        } else {
            format!(
                "Editing guild permissions for user {0}...",
                self.user.user.mention()
            )
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(response_string))
            })
            .await
            .expect("Failed to send response");
        Ok(())
    }
}

#[derive(Debug, SubCommand)]
struct GetPermissionsForUserCommand {
    /// The user to get
    pub user: UserInput,

    /// The channel permissions to get. If omitted, the guild permissions will be returned
    pub channel: Option<PartialChannel>,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for GetPermissionsForUserCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let guild = command
            .guild_id
            .expect("Command should only be called from a guild")
            .to_partial_guild(&ctx.http)
            .await
            .expect("Error getting guild");
        let member = guild
            .member(&ctx.http, self.user.user.id)
            .await
            .expect("Error getting member");
        let response_string = if let Some(ref channel) = self.channel {
            if let Channel::Guild(channel) = channel
                .id
                .to_channel(&ctx.http)
                .await
                .expect("Error getting channel")
            {
                let permissions = guild
                    .user_permissions_in(&channel, &member)
                    .expect("Error getting permissions");
                format!(
                    "User {} has the following permissions in channel {}:\n{}",
                    self.user.user.mention(),
                    channel.mention(),
                    permissions.get_permission_names().join("\n"),
                )
            } else {
                panic!("Channel was not guild channel");
            }
        } else {
            let permissions = self
                .user
                .member
                .as_ref()
                .expect("Command can only be called from guild")
                .permissions
                .expect("Error getting permissions");
            format!(
                "User {} has the following permissions in this guild:\n{}",
                self.user.user.mention(),
                permissions.get_permission_names().join("\n"),
            )
        };
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(response_string))
            })
            .await
            .expect("Failed to send response");
        Ok(())
    }
}

#[derive(Debug, SubCommand)]
struct EditPermissionsForRoleCommand {
    /// The role to edit
    pub role: Role,

    /// The channel permissions to edit. If omitted, the guild permissions will be edited
    pub channel: Option<PartialChannel>,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for EditPermissionsForRoleCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let response_string = if let Some(ref channel) = self.channel {
            format!(
                "Editing permissions for role {0} in {1}...",
                self.role.mention(),
                channel.id.mention(),
            )
        } else {
            format!(
                "Editing guild permissions for role {0}...",
                self.role.mention()
            )
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(response_string))
            })
            .await
            .expect("Failed to send response");
        Ok(())
    }
}

#[derive(Debug, SubCommand)]
struct GetPermissionsForRoleCommand {
    /// The user to get
    pub role: Role,

    /// The channel permissions to get. If omitted, the guild permissions will be returned
    pub channel: Option<PartialChannel>,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for GetPermissionsForRoleCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let guild = command
            .guild_id
            .expect("Command should only be called from a guild")
            .to_partial_guild(&ctx.http)
            .await
            .expect("Error getting guild");
        let response_string = if let Some(ref channel) = self.channel {
            if let Channel::Guild(channel) = channel
                .id
                .to_channel(&ctx.http)
                .await
                .expect("Error getting channel")
            {
                let permissions = guild
                    .role_permissions_in(&channel, &self.role)
                    .expect("Error getting permissions");
                format!(
                    "Role {} has the following permissions in channel {}:\n{}",
                    self.role.mention(),
                    channel.mention(),
                    permissions.get_permission_names().join("\n"),
                )
            } else {
                panic!("Channel was not guild channel");
            }
        } else {
            let permissions = self.role.permissions;
            format!(
                "Role {} has the following permissions in this guild:\n{}",
                self.role.mention(),
                permissions.get_permission_names().join("\n"),
            )
        };
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(response_string))
            })
            .await
            .expect("Failed to send response");
        Ok(())
    }
}

#[derive(Debug, Commands)]
enum BotCommands {
    Permissions(PermissionsCommand),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command_interaction) => {
                BotCommands::parse(&ctx, &command_interaction)
                    .expect("Failed to parse command")
                    .invoke(&ctx, &command_interaction)
                    .await
                    .expect("Failed to invoke command");
            }
            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild_id = std::env::var("TEST_GUILD_ID").map(|id| {
            id.parse()
                .map(|id| GuildId(id))
                .expect("Invalid test guild id id")
        });
        let guild_id = match guild_id {
            Ok(id) => Some(id),
            Err(VarError::NotPresent) => None,
            _ => panic!("Invalid guild id provided at $TEST_GUILD_ID"),
        };
        let commands = register_commands!(&ctx, guild_id, [PermissionsCommand])
            .expect("Unable to register commands");
        println!(
            "Registered {} commands {}",
            commands.len(),
            match guild_id {
                Some(id) => format!("for guild_id: {}", id),
                None => "globally".to_owned(),
            },
        );
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = std::env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse::<u64>()
        .expect("Invalid user id");
    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
