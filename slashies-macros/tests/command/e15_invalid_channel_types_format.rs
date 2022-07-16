use serenity::model::channel::PartialChannel;
use slashies_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with badly formatted channel types
    #[channel_types]
    bad_field: PartialChannel,

    /// Ok field
    #[channel_types("text")]
    good_field: PartialChannel,
}

fn main() {}
