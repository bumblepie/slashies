use slashies_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with invalid min type
    #[min = "abc"]
    bad_field: String,

    /// Ok field
    #[min = 0]
    good_field: i64,
}

fn main() {}
