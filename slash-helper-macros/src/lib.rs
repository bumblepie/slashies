use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta};

#[proc_macro_derive(Command, attributes(name))]
pub fn describe(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);
    if let syn::Data::Struct(data) = data {
        if let syn::Fields::Named(_fields) = data.fields {
            unimplemented!();
        } else if syn::Fields::Unit == data.fields {
            let name_meta = attrs
                .iter()
                .find(|attr| attr.path.is_ident("name"))
                .expect("Command must specify a name via the \"name\" attribute")
                .parse_meta()
                .expect("Invalid \"name\" attribute");
            let doc_meta = attrs
                .iter()
                .find(|attr| attr.path.is_ident("doc"))
                .expect("Command must specify a description via a docstring")
                .parse_meta()
                .expect("Invalid docstring");
            if let (Meta::NameValue(name_val), Meta::NameValue(doc_val)) = (name_meta, doc_meta) {
                let name = name_val.lit;
                let description = doc_val.lit;
                let output = quote! {
                    impl Command for #ident {
                        fn parse(_command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
                            Ok(Self)
                        }

                        fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
                            command
                                .name(#name)
                                .description(#description)
                        }
                    }
                };
                output.into()
            } else {
                panic!("Invalid \"name\" attribute");
            }
        } else {
            panic!("Can only derive Command for unit structs or structs with named fields");
        }
    } else {
        panic!("Can only derive Command for structs");
    }
}

// impl Command for CountCommand {
//     fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
//         let phrase = command
//             .data
//             .options
//             .iter()
//             .find(|option| option.name == "phrase")
//             .ok_or(ParseError::MissingOption)?
//             .resolved
//             .clone()
//             .ok_or(ParseError::MissingOption)?;
//         if let ApplicationCommandInteractionDataOptionValue::String(phrase) = phrase {
//             Ok(Self { phrase })
//         } else {
//             Err(ParseError::InvalidOption)
//         }
//     }

//     fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
//         command
//             .name(COUNT_COMMAND_NAME)
//             .description("Count the number of syllables in a given phrase")
//             .create_option(|option| {
//                 option
//                     .name("phrase")
//                     .description("The phrase to count")
//                     .kind(ApplicationCommandOptionType::String)
//                     .required(true)
//             })
//     }
// }
