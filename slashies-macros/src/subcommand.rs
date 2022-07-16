use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use syn::{Ident, Variant, Meta, Lit, DataEnum};
use quote::{quote, ToTokens};

use crate::{command::OptionTokenSections, utility};

pub fn impl_subcommand_for_struct(
    identifier: Ident,
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
    
    let output = quote!{
        impl slashies::SubCommand for #identifier {
            fn parse(
                option: Option<&serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>,
            ) -> Result<Self, slashies::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = option
                    .ok_or(slashies::ParseError::MissingOption)?
                    .options
                    .iter()
                    .map(|option| (option.name.clone(), option.clone()))
                    .collect();
                
                #(#parse_fetch)*

                Ok(Self {
                    #(#parse_struct_item)*
                })
            }
        
            fn register_sub_options(
                option: &mut serenity::builder::CreateApplicationCommandOption,
            ) -> &mut serenity::builder::CreateApplicationCommandOption {
                // Ensure required options are added first
                let mut options: Vec<(bool, Box<dyn Fn(&mut serenity::builder::CreateApplicationCommandOption) -> &mut serenity::builder::CreateApplicationCommandOption>)> = vec![
                    #((#is_required, Box::new(#registration_fn)),)*
                ];
                options.sort_by(|(req_a, _), (req_b, _)| match (req_a, req_b) {
                    (true, true) | (false, false) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                });
                
                let mut option = option;
                for (_, registration_fn) in options {
                    option = option.create_sub_option(registration_fn);
                }
                option
            }
        }
    }.into();
    output
}

#[derive(Debug)]
pub struct SubCommandTokenSections {
    pub parse_fetch: proc_macro2::TokenStream,
    pub variant_identifier: proc_macro2::Ident,
    pub registration_fn: proc_macro2::TokenStream,
}

pub fn subcommand_token_sections_from_enum_variant(variant: &Variant) -> SubCommandTokenSections {
    match variant.fields {
        syn::Fields::Unnamed(ref fields) => {
            let fields = &fields.unnamed;
            if fields.len() != 1 {
                abort!(fields, "Variants of a Command enum must be a tuple of length 1, containing only the subcommand");
            }
            let field = &fields[0];
            let variant_identifier = &variant.ident;
            let name_attr = variant
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("name"))
                .unwrap_or_else(|| {
                    abort!(
                        variant,
                        "Subcommand must specify a name via the \"name\" attribute",
                    );
                });
            let name_meta = name_attr
                .parse_meta()
                .unwrap_or_else(|_| abort!(name_attr, "Invalid \"name\" attribute"));
            let subcommand_name = match name_meta {
                Meta::NameValue(value) => value.lit,
                _ => abort!(name_attr, "Invalid \"name\" attribute"),
            };
            let description = utility::get_description(variant.attrs.as_slice())
                .unwrap_or_else(|| {
                    abort!(
                        variant,
                        "Subcommands must specify a description via a docstring"
                    )
                });
            let field_type = field.ty.to_token_stream();
            if variant
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("subcommandgroup"))
                .is_some()
            {
                SubCommandTokenSections {
                    parse_fetch: quote! {
                        match options.get(#subcommand_name) {
                            Some(option) => Some(<#field_type as slashies::SubCommandGroup>::parse(Some(option))?),
                            None => None,
                        }
                    },
                    variant_identifier: variant_identifier.clone(),
                    registration_fn: quote! {
                        |option: &mut serenity::builder::CreateApplicationCommandOption| {
                            let option = option
                                .kind(serenity::model::interactions::application_command::ApplicationCommandOptionType::SubCommandGroup)
                                .name(#subcommand_name)
                                .description(#description)
                                .required(false);
                            <#field_type as slashies::SubCommandGroup>::register_sub_options(option)
                        }
                    },
                }
            } else {
                SubCommandTokenSections {
                    parse_fetch: quote! {
                        match options.get(#subcommand_name) {
                            Some(option) => Some(<#field_type as slashies::SubCommand>::parse(Some(option))?),
                            None => None,
                        }
                    },
                    variant_identifier: variant_identifier.clone(),
                    registration_fn: quote! {
                        |option: &mut serenity::builder::CreateApplicationCommandOption| {
                            let option = option
                                .kind(serenity::model::interactions::application_command::ApplicationCommandOptionType::SubCommand)
                                .name(#subcommand_name)
                                .description(#description)
                                .required(false);
                            <#field_type as slashies::SubCommand>::register_sub_options(option)
                        }
                    },
                }
            }
        }
        _ => abort!(
            variant,
            "Subcommand variants for a Command enum must have unnamed fields"
        ),
    }
}

pub fn subcommands_for_enum(data: &DataEnum) -> Vec<SubCommandTokenSections> {
    data.variants
        .iter()
        .map(|variant| subcommand_token_sections_from_enum_variant(variant))
        .collect()
}

pub fn impl_command_for_enum(
    identifier: Ident,
    name: Lit,
    description: &str,
    sub_commands: Vec<SubCommandTokenSections>,
) -> TokenStream {
    let (parse_fetch, variant_identifier, registration_fn): (Vec<_>, Vec<_>, Vec<_>) = sub_commands
        .into_iter()
        .map(|sub_command| {
            let SubCommandTokenSections {
                parse_fetch,
                variant_identifier,
                registration_fn,
            } = sub_command;
            (parse_fetch, variant_identifier, registration_fn)
        })
        .multiunzip();

    let output = quote! {
        impl slashies::Command for #identifier {
            fn parse(command: &serenity::model::interactions::application_command::ApplicationCommandInteraction) -> Result<Self, slashies::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = command.data
                    .options
                    .iter()
                    .map(|option| (option.name.clone(), option.clone()))
                    .collect();

                #(if let Some(value) = #parse_fetch {
                    return Ok(Self::#variant_identifier(value));
                })*
                Err(slashies::ParseError::MissingOption)
            }

            fn name() -> String {
                #name.to_owned()
            }

            fn register(command: &mut serenity::builder::CreateApplicationCommand) -> &mut serenity::builder::CreateApplicationCommand {
                command
                    .name(#name)
                    .description(#description)
                    #(.create_option(#registration_fn))*
            }
        }
    };
    output.into()
}