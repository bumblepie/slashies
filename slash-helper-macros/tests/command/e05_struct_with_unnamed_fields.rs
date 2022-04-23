use slash_helper_macros::Command;

/// An invalid command with unnamed fields
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand(u64, u64);

fn main() {}
