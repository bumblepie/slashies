# A simple way to create slash commands for Discord bots

Slashies helps to reduce the boiler plate code needed to create slash commands for a Discord bot. It
is built on top of [Serenity](https://github.com/serenity-rs/serenity). It focuses on providing
traits that you can derive using the `slashies_macros` crate for most straightforward use cases, but
gives you the escape hatch of implementing these traits yourself if you want to do something more
complex.

To get an understanding of how it works, check out the examples, and see the docs for a reference for
the various parts of the crate and how to use them.

Next steps for the crate:
- [x] Set up some examples to help show common use cases
- [ ] Add examples and docs for using message components
- [ ] Implement a trait for user/message commands
- [ ] Implement a trait for autocomplete interactions and support them in the derive macros
- [ ] Implement a trait for easy permissions for commands
- [ ] Enforce at compile time Discord's restrictions around things like:
    - number of command options
    - number of choices for multi-choice inputs
    - legal characters in names