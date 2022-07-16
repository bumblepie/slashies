use slashies_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with badly formatted max attribute
    #[max]
    bad_field: i64,

    /// Ok field
    #[max = 0]
    good_field: i64,
}

fn main() {}
