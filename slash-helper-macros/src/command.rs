use crate::utility;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, Ident, Lit};

/// For each command option, we need four sections of code:
/// - parse_fetch: Parse the field from a discord command interaction option into a variable
/// - parse_struct_item: Add the field to the resulting struct
/// - is_required: Whether the command option is required (used to order command options as required options must be added first when registering the command)
/// - registration_fn: Register the command option
#[derive(Debug)]
pub struct OptionTokenSections {
    pub parse_fetch: proc_macro2::TokenStream,
    pub parse_struct_item: proc_macro2::TokenStream,
    pub is_required: proc_macro2::TokenStream,
    pub registration_fn: proc_macro2::TokenStream,
}

fn option_token_sections_from_field(field: &Field) -> OptionTokenSections {
    let field_ident = field.ident.as_ref().unwrap_or_else(|| {
        abort!(
            field.ty,
            "Unnamed struct fields are not supported for commands",
        );
    });
    let option_name = field_ident.to_string();

    let description = utility::get_description(field.attrs.as_slice()).unwrap_or_else(|| {
        abort!(
            field_ident,
            "Command options must specify a description via a docstring"
        )
    });
    let choices = utility::get_choices(field.attrs.as_slice());
    let channel_types = utility::get_channel_types(field.attrs.as_slice());

    let field_type = field.ty.to_token_stream();
    OptionTokenSections {
        parse_fetch: quote! {
            let #field_ident = <#field_type as slash_helper::parsable::ParsableCommandOption>::parse_from(options.get(#option_name))?;
        },
        parse_struct_item: quote! {
            #field_ident,
        },
        is_required: quote! {
            <#field_type as slash_helper::parsable::ParsableCommandOption>::is_required()
        },
        registration_fn: quote! {
            |option: &mut serenity::builder::CreateApplicationCommandOption| option
                .kind(<#field_type as slash_helper::parsable::ParsableCommandOption>::application_command_option_type())
                .name(#option_name)
                .description(#description)
                .required(<#field_type as slash_helper::parsable::ParsableCommandOption>::is_required())
                #(#choices)*
                #channel_types
        },
    }
}

pub fn options_for_struct_data(data: &DataStruct) -> Vec<OptionTokenSections> {
    match data.fields {
        syn::Fields::Named(_) => data
            .fields
            .iter()
            .map(|field| option_token_sections_from_field(field))
            .collect(),
        syn::Fields::Unit => Vec::new(),
        _ => abort!(
            data.fields,
            "Can only derive Command for unit structs or structs with named fields"
        ),
    }
}

pub fn impl_command_for_struct(
    identifier: Ident,
    name: Lit,
    description: &str,
    options: Vec<OptionTokenSections>,
) -> TokenStream {
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
        impl slash_helper::Command for #identifier {
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

            fn name() -> String {
                #name.to_owned()
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

#[cfg(test)]
mod test {
    #[test]
    fn examples_fail_with_correct_error() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/command/*.rs");
    }
}
