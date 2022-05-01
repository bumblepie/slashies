use movie::MovieDatabase;
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::{Context, EventHandler},
    model::{
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
use slash_helper::{register_commands, ApplicationCommandInteractionHandler, InvocationError};
use slash_helper_macros::{Command, Commands};
use std::env::VarError;

mod movie;

/// Greet a user
#[derive(Debug, Command)]
#[name = "recommend"]
struct RecommendCommand {
    /// The genre of movie to recommend
    #[choice("Comedy", "comedy")]
    #[choice("Action", "action")]
    #[choice("Sci-Fi", "sci-fi")]
    genre: String,

    /// The minimum rating required for recommendations out of 5 stars
    #[min = 0.0]
    #[max = 5.0]
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
                                    .field("Director", &rec.director, false)
                                    .field(
                                        "Average Rating",
                                        format!("{:.2} / 5.0", rec.average_rating),
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

#[derive(Debug, Commands)]
enum Commands {
    Recommend(RecommendCommand),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command_interaction) => {
                Commands::parse(&ctx, &command_interaction)
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
        let commands = register_commands!(&ctx, guild_id, [RecommendCommand])
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

    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Movies>(MovieDatabase {});
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
