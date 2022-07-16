use movie::MovieDatabase;
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::{Context, EventHandler},
    model::{
        channel::{ChannelType, PartialChannel},
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionResponseType,
        },
        prelude::Ready,
    },
    prelude::{GatewayIntents, TypeMapKey},
    Client,
};
use slashies::{
    register_commands, ApplicationCommandInteractionHandler, Commands, InvocationError,
};
use slashies_macros::{Command, Commands};
use std::env::VarError;

mod movie;

/// Recommend me a movie!
#[derive(Debug, Command)]
#[name = "recommend"]
struct RecommendCommand {
    /// The genre of movie to recommend
    #[choice("Action")]
    #[choice("Adventure")]
    #[choice("Animation")]
    #[choice("Biography")]
    #[choice("Comedy")]
    #[choice("Crime")]
    #[choice("Documentary")]
    #[choice("Drama")]
    #[choice("Family")]
    #[choice("Fantasy")]
    #[choice("Film Noir")]
    #[choice("History")]
    #[choice("Horror")]
    #[choice("Musical")]
    #[choice("Music")]
    #[choice("Mystery")]
    #[choice("Romance")]
    #[choice("Sci-Fi")]
    #[choice("Short")]
    #[choice("Sport")]
    #[choice("Thriller")]
    #[choice("War")]
    #[choice("Western")]
    genre: String,

    /// The minimum rating required for recommendations out of 10
    #[min = 0.0]
    #[max = 10.0]
    min_rating: Option<f64>,

    /// The number of recommendations desired. Defaults to one.
    #[min = 1]
    #[max = 3]
    num_recommendations: Option<i64>,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for RecommendCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let data = ctx.data.read().await;
        let movie_db = data.get::<Movies>().expect("Expected MovieDB in TypeMap");
        let recommendations = movie_db.get_movie_recommendations(
            &self.genre,
            &self.min_rating,
            &self.num_recommendations,
        );
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        let embeds = recommendations
                            .iter()
                            .map(|rec| {
                                let mut embed = CreateEmbed::default();
                                embed
                                    .title(&rec.title)
                                    .field("Release Year", rec.release_year, false)
                                    .field("Director(s)", &rec.directors.join(", "), false)
                                    .field(
                                        "Average Rating",
                                        format!("{:.2} / 10.0", rec.average_rating),
                                        false,
                                    )
                                    .field("Genres", rec.genres.join(", "), false);
                                embed
                            })
                            .collect();
                        data.add_embeds(embeds)
                    })
            })
            .await
            .expect("Error sending movie recommendations");
        Ok(())
    }
}

/// Set the channel in which new movie releases are announced
#[derive(Debug, Command)]
#[name = "set_releases_channel"]
struct SetReleasesChannelCommand {
    /// The channel to announce releases in
    #[channel_types(ChannelType::Text, ChannelType::News)]
    channel: PartialChannel,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for SetReleasesChannelCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        // We don't actually do anything as we don't actually track movie releases, but you might implement
        // this by saving the channel id somewhere that can be accessed by a webhook triggered whenever a movie is released
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.content(format!(
                            "Ok, I've set the movie releases channel to be {}",
                            self.channel
                                .name
                                .as_ref()
                                .expect("Expected channel to have a name")
                        ))
                    })
            })
            .await
            .expect("Error sending command response");
        Ok(())
    }
}

#[derive(Debug, Commands)]
enum BotCommands {
    Recommend(RecommendCommand),
    SetReleasesChannelCommand(SetReleasesChannelCommand),
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
        let commands = register_commands!(
            &ctx,
            guild_id,
            [RecommendCommand, SetReleasesChannelCommand]
        )
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

struct Movies;
impl TypeMapKey for Movies {
    type Value = MovieDatabase;
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = std::env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse::<u64>()
        .expect("Invalid user id");
    let imdb_sqlite_file =
        std::env::var("IMDB_SQLITE_FILE").expect("Expected IMDB_SQLITE_FILE to be set");

    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Movies>(MovieDatabase { imdb_sqlite_file });
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
