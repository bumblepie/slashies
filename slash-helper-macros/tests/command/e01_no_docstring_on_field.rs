use slash_helper_macros::Command;

/// A command with an invalid field that has no description
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    bad_field: u64,
    /// Ok field
    good_field: u64,
}

fn main() {}
