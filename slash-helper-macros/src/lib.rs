use itertools::Itertools;
use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Field, Lit, Meta};

/// For each command option, we need four sections of code:
/// - parse_fetch: Parse the field from a discord command interaction option into a variable
/// - parse_struct_item: Add the field to the resulting struct
/// - is_required: Whether the command option is required (used to order command options as required options must be added first when registering the command)
/// - registration_fn: Register the command option
#[derive(Debug)]
struct OptionTokenSections {
    parse_fetch: proc_macro2::TokenStream,
    parse_struct_item: proc_macro2::TokenStream,
    is_required: proc_macro2::TokenStream,
    registration_fn: proc_macro2::TokenStream,
}

fn option_token_sections_from_field(field: &Field) -> OptionTokenSections {
    let field_ident = field
        .ident
        .as_ref()
        .expect("Unnamed struct fields are not supported");
    let option_name = field_ident.to_string();
    let doc_meta = field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("doc"))
        .expect("Command options must specify a description via a docstring")
        .parse_meta()
        .expect("Invalid docstring");
    let description = match doc_meta {
        Meta::NameValue(ref value) => match value.lit {
            Lit::Str(ref description) => description.value(),
            _ => panic!("Invalid description docstring"),
        },
        _ => panic!("Invalid description docstring"),
    };
    let field_type = field.ty.to_token_stream();
    OptionTokenSections {
        parse_fetch: quote! {
            let #field_ident = <#field_type as slash_helper::ParsableCommandOption>::parse_from(options.get(#option_name))?;
        },
        parse_struct_item: quote! {
            #field_ident,
        },
        is_required: quote! {
            <#field_type as slash_helper::ParsableCommandOption>::is_required()
        },
        registration_fn: quote! {
            |option: &mut serenity::builder::CreateApplicationCommandOption| option
                .kind(<#field_type as slash_helper::ParsableCommandOption>::application_command_option_type())
                .name(#option_name)
                .description(#description)
                .required(<#field_type as slash_helper::ParsableCommandOption>::is_required())
        },
    }
}

#[proc_macro_derive(Command, attributes(name, option_type))]
pub fn derive_commmand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);
    let options: Vec<OptionTokenSections> = if let syn::Data::Struct(data) = data {
        if let syn::Fields::Named(_) = data.fields {
            data.fields
                .into_iter()
                .map(|field| option_token_sections_from_field(&field))
                .collect()
        } else if syn::Fields::Unit == data.fields {
            Vec::new()
        } else {
            panic!("Can only derive Command for unit structs or structs with named fields");
        }
    } else {
        panic!("Can only derive Command for structs");
    };

    let name_meta = attrs
        .iter()
        .find(|attr| attr.path.is_ident("name"))
        .expect("Command must specify a name via the \"name\" attribute")
        .parse_meta()
        .expect("Invalid \"name\" attribute");
    let name = match name_meta {
        Meta::NameValue(value) => value.lit,
        _ => panic!("Invalid \"name\" attribute"),
    };
    let doc_meta = attrs
        .iter()
        .find(|attr| attr.path.is_ident("doc"))
        .expect("Command must specify a description via a docstring")
        .parse_meta()
        .expect("Invalid docstring");
    let description = match doc_meta {
        Meta::NameValue(value) => value.lit,
        _ => panic!("Invalid description docstring"),
    };
    let (parse_fetch, parse_struct_item, is_required, registration_fn): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = options
        .into_iter()
        .map(|option| {
            let OptionTokenSections {
                parse_fetch,
                parse_struct_item,
                is_required,
                registration_fn,
            } = option;
            (parse_fetch, parse_struct_item, is_required, registration_fn)
        })
        .multiunzip();

    let output = quote! {
        impl Command for #ident {
            fn parse(command: &serenity::model::interactions::application_command::ApplicationCommandInteraction) -> Result<Self, slash_helper::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = command.data
                    .options
                    .iter()
                    .map(|option| (option.name.clone(), option.clone()))
                    .collect();

                #(#parse_fetch)*

                Ok(Self {
                    #(#parse_struct_item)*
                })
            }

            fn register(command: &mut serenity::builder::CreateApplicationCommand) -> &mut serenity::builder::CreateApplicationCommand {
                // Ensure required options are added first
                let mut options: Vec<(bool, Box<dyn Fn(&mut serenity::builder::CreateApplicationCommandOption) -> &mut serenity::builder::CreateApplicationCommandOption>)> = vec![
                    #((#is_required, Box::new(#registration_fn)),)*
                ];
                options.sort_by(|(req_a, _), (req_b, _)| match (req_a, req_b) {
                    (true, true) | (false, false) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                });

                let mut command = command
                    .name(#name)
                    .description(#description);

                for (_, registration_fn) in options {
                    command = command.create_option(registration_fn);
                }
                command
            }
        }
    };
    output.into()
}
