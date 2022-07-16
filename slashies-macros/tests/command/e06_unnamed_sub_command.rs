use slashies_macros::Command;

/// An command with an unnamed subcommand
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand {
    Sub(SubCommand),
}

struct SubCommand;

fn main() {}
