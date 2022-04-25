use std::collections::HashMap;

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use serenity::model::channel::ChannelType;
use syn::{Attribute, DataStruct, Field, Ident, Lit, Meta, NestedMeta};

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

    let doc_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("doc"))
        .unwrap_or_else(|| {
            abort!(
                field_ident,
                "Command options must specify a description via a docstring",
            );
        });

    let doc_meta = doc_attr.parse_meta().unwrap_or_else(|_| {
        abort!(doc_attr, "Invalid docstring",);
    });
    let description = match doc_meta {
        Meta::NameValue(ref value) => match value.lit {
            Lit::Str(ref description) => description.value(),
            _ => abort!(doc_attr, "Invalid docstring",),
        },
        _ => abort!(doc_attr, "Invalid docstring",),
    };
    let choices = get_choices(field.attrs.as_slice());
    let channel_types = get_channel_types(field.attrs.as_slice());

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

fn get_choices(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("choice"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid \"choice\" attribute"),
        })
        .map(|(attr, meta)| match meta {
            Meta::List(list) => {
                let list = list.nested;
                let (name_meta, value_meta) = match list.len() {
                    1 => (&list[0], &list[0]),
                    2 => (&list[0], &list[1]),
                    _ => abort!(attr, "Invalid \"choices\" attribute. Attribute must be of the form choice(name, value) or choice(value)"),
                };
                let name = match name_meta {
                    NestedMeta::Lit(lit) => lit,
                    _ => abort!(attr, "Invalid \"choices\" attribute"),
                };
                match value_meta {
                    NestedMeta::Lit(Lit::Str(ref lit_str)) => quote! {
                        .add_string_choice(#name, #lit_str)
                    },
                    NestedMeta::Lit(Lit::Int(ref lit_int)) => quote! {
                        .add_int_choice(#name, #lit_int)
                    },
                    NestedMeta::Lit(Lit::Float(ref lit_num)) => quote! {
                        .add_number_choice(#name, #lit_num)
                    },
                    _ => abort!(attr, "Invalid \"choices\" attribute - can only have string, integer or number choices"),
                }
            }
            _ => abort!(attr, "Invalid \"choices\" attribute. Attribute must be of the form choice(name, value) or choice(value)"),
        })
        .collect::<Vec<_>>()
}

fn channel_type_from_string(name: &str) -> Option<proc_macro2::TokenStream> {
    let all_types = HashMap::from([
        (
            ChannelType::Text.name(),
            quote! { serenity::model::channel::ChannelType::Text },
        ),
        (
            ChannelType::Private.name(),
            quote! { serenity::model::channel::ChannelType::Private },
        ),
        (
            ChannelType::Voice.name(),
            quote! { serenity::model::channel::ChannelType::Voice },
        ),
        (
            ChannelType::Category.name(),
            quote! { serenity::model::channel::ChannelType::Category },
        ),
        (
            ChannelType::News.name(),
            quote! { serenity::model::channel::ChannelType::News },
        ),
        (
            ChannelType::NewsThread.name(),
            quote! { serenity::model::channel::ChannelType::NewsThread },
        ),
        (
            ChannelType::PublicThread.name(),
            quote! { serenity::model::channel::ChannelType::PublicThread },
        ),
        (
            ChannelType::PrivateThread.name(),
            quote! { serenity::model::channel::ChannelType::PrivateThread },
        ),
        (
            ChannelType::Stage.name(),
            quote! { serenity::model::channel::ChannelType::Stage },
        ),
        (
            ChannelType::Unknown.name(),
            quote! { serenity::model::channel::ChannelType::Unknown },
        ),
    ]);
    all_types.get(name).map(|tokens| tokens.clone())
}

fn get_channel_types(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    attrs.iter().find(|attr| attr.path.is_ident("channel_types"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid \"channel_type\" attribute"),
        })
        .map(|(attr, meta)| match meta {
            Meta::List(list) => {
                list.nested.iter().map(|nested| match nested {
                    NestedMeta::Lit(Lit::Str(ref value)) => {
                        channel_type_from_string(&value.value()).unwrap_or_else(|| abort!(nested, "Invalid channel type"))
                    },
                    _ => abort!(nested, "Invalid channel type"),
                }).collect()
            },
            _ => abort!(attr, "Invalid \"channel_type\" attribute. Attribute must be of the form channel_type(type1, type2...)"),
        }
    ).map(|channel_types: Vec<_>| {
        quote! { .channel_types(&[#(#channel_types,)*]) }
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn examples_fail_with_correct_error() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/command/*.rs");
    }
}
