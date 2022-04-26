use slash_helper_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with invalid max type
    #[max = "abc"]
    bad_field: String,

    /// Ok field
    #[max = 0]
    good_field: i64,
}

fn main() {}
