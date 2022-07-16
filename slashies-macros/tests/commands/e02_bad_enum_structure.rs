use slashies_macros::Commands;

#[derive(Commands)]
enum BadCommands {
    HasNamedFields { named_field: u64 },
}

#[derive(Commands)]
enum BadCommands2 {
    HasNoFields,
}

#[derive(Commands)]
enum BadCommands3 {
    HasMultipleUnnamedFields(u64, u64),
}

fn main() {}
