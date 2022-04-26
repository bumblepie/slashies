use slash_helper_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with badly formatted min attribute
    #[min]
    bad_field: i64,

    /// Ok field
    #[min = 0]
    good_field: i64,
}

fn main() {}
