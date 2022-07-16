use slashies_macros::Command;

/// An invalid command (not a struct or enum)
#[derive(Command)]
#[name = "BadCommand"]
union BadCommand {
    x: u64,
}

fn main() {}
