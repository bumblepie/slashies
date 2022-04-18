# Running the examples
## Setup the bot
To run the examples, you'll first need to set up an application in the discord developer portal. The instructions can be found at https://discord.com/developers/docs/getting-started - you'll only need to complete the steps from the overview up to (and including) installing the app.

## Clone the repository and create the config file
These instructions assume you're already familiar with git. The next step is to clone the repository and make a copy of the `example.env` file in the `examples` directory. Rename it to `dev.env` and replace the placeholder values as follows:
- `DISCORD_TOKEN` is the token for the bot retrieved from the discord developer portal
- `DISCORD_USER_ID` is the application id of the bot, also retrieved from the discord developer portal
- `TEST_GUILD_ID` is the id of the discord guild/server that you are testing the bot in - you can find this by right clicking the server icon in discord and clicking `Copy ID`
Once you have populated the `dev.env` file, you can make sure those environment variables are set by running `source dev.env`. You'll need to run this each time you open a terminal to run the bot. Remember, these are secrets, so never check them into source control or share them with anyone.

## Starting the bot
You can now change into the directory of the example you want to run and simply run `cargo run` to start the bot. Note that some examples require additional permissions, which might mean you need to edit the bot's permissions in discord or enable intents in the developer portal if you see error messages