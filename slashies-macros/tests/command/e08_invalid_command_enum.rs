use slashies_macros::Command;

/// An command with an invalid subcommand structure
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand {
    SubCommand { named: u64, fields: u64 },
}

/// An command with an invalid subcommand structure
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand2 {
    SubCommand(u64, u64),
}

/// An command with an invalid subcommand structure
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand3 {
    SubCommand,
}

fn main() {}
