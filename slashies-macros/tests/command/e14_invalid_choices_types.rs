use slashies_macros::Command;

/// An invalid command
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand {
    /// Field with badly formatted choices
    #[choice("z", true)]
    bad_field_3: u64,

    /// Ok field
    #[choice(0)]
    #[choice("one", 1)]
    good_field: u64,
}

fn main() {}
