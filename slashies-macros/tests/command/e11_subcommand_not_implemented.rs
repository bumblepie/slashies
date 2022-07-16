use slashies_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand {
    /// A variant that doe snot implement SubCommand
    #[name = "BadSubcommand"]
    Sub(SubCommand),
}

struct SubCommand;

fn main() {}
