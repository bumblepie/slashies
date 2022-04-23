use slash_helper_macros::Command;

/// A command with an invalid field that has an invalid description
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    #[doc]
    bad_field: u64,
    /// Ok field
    good_field: u64,
}

fn main() {}
