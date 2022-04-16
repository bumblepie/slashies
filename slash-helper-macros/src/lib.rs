use itertools::Itertools;
use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Field, Ident, Lit, Meta, Variant};

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
        },
    }
}

fn options_for_struct_data(data: &DataStruct) -> Vec<OptionTokenSections> {
    match data.fields {
        syn::Fields::Named(_) => data
            .fields
            .iter()
            .map(|field| option_token_sections_from_field(field))
            .collect(),
        syn::Fields::Unit => Vec::new(),
        _ => panic!("Can only derive Command for unit structs or structs with named fields"),
    }
}

fn impl_command_for_struct(
    identifier: Ident,
    name: Lit,
    description: Lit,
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
        impl Command for #identifier {
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

#[derive(Debug)]
struct SubCommandTokenSections {
    parse_fetch: proc_macro2::TokenStream,
    variant_identifier: proc_macro2::Ident,
    registration_fn: proc_macro2::TokenStream,
}

fn subcommand_token_sections_from_enum_variant(variant: &Variant) -> SubCommandTokenSections {
    match variant.fields {
        syn::Fields::Unnamed(ref fields) => {
            let fields = &fields.unnamed;
            if fields.len() != 1 {
                panic!("Variants of a Command enum must be a tuple of length 1, containing only the subcommand");
            }
            let field = &fields[0];
            let variant_identifier = &variant.ident;
            let name_meta = variant
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("name"))
                .expect("Subcommand must specify a name via the \"name\" attribute")
                .parse_meta()
                .expect("Invalid \"name\" attribute");
            let subcommand_name = match name_meta {
                Meta::NameValue(value) => value.lit,
                _ => panic!("Invalid \"name\" attribute"),
            };
            let doc_meta = variant
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("doc"))
                .expect("Subcommands must specify a description via a docstring")
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
            SubCommandTokenSections {
                parse_fetch: quote! {
                    match options.get(#subcommand_name) {
                        Some(option) => Some(<#field_type as slash_helper::SubCommand>::parse(Some(option))?),
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
                        <#field_type as slash_helper::SubCommand>::register_sub_options(option)
                    }
                },
            }
        }
        _ => panic!("Can only derive Command for enums with unnamed tuple variants"),
    }
}

fn sub_commands_for_enum(data: &DataEnum) -> Vec<SubCommandTokenSections> {
    data.variants
        .iter()
        .map(|variant| subcommand_token_sections_from_enum_variant(variant))
        .collect()
}

fn impl_command_for_enum(
    identifier: Ident,
    name: Lit,
    description: Lit,
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
        impl Command for #identifier {
            fn parse(command: &serenity::model::interactions::application_command::ApplicationCommandInteraction) -> Result<Self, slash_helper::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = command.data
                    .options
                    .iter()
                    .map(|option| (option.name.clone(), option.clone()))
                    .collect();

                #(if let Some(value) = #parse_fetch {
                    return Ok(Self::#variant_identifier(value));
                })*
                Err(slash_helper::ParseError::MissingOption)
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

#[proc_macro_derive(Command, attributes(name))]
pub fn derive_commmand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

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

    match data {
        syn::Data::Struct(ref data) => {
            impl_command_for_struct(ident, name, description, options_for_struct_data(data))
        }
        syn::Data::Enum(ref data) => {
            impl_command_for_enum(ident, name, description, sub_commands_for_enum(data))
        }
        _ => panic!("Can only derive Command for structs"),
    }
}

fn impl_subcommand_for_struct(
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
        impl SubCommand for #identifier {
            fn parse(
                option: Option<&serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>,
            ) -> Result<Self, slash_helper::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = option
                    .ok_or(slash_helper::ParseError::MissingOption)?
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

#[proc_macro_derive(SubCommand, attributes(name, option_type))]
pub fn derive_subcommmand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, ..
    } = parse_macro_input!(input);

    match data {
        syn::Data::Struct(ref data) => {
            impl_subcommand_for_struct(ident, options_for_struct_data(data))
        }
        _ => panic!("Can only derive SubCommand for structs"),
    }
}