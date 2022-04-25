use serenity::model::channel::PartialChannel;
use slash_helper_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with channel types that don't exist
    #[channel_types("telegram", "telepathy")]
    bad_field: PartialChannel,

    /// Ok field
    #[channel_types("text")]
    good_field: PartialChannel,
}

fn main() {}
