use slashies_macros::Commands;

#[derive(Commands)]
enum BadCommands {
    DoSomething(u64),
}

fn main() {}
