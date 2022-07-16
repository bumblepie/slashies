use command::{impl_command_for_struct, options_for_struct_data};
use commands::get_commands_variant_info;
use itertools::Itertools;
use proc_macro::{self, TokenStream};
use proc_macro_error::{proc_macro_error, abort};
use quote::quote;
use subcommand::{impl_subcommand_for_struct, impl_command_for_enum, subcommands_for_enum};
use subcommandgroup::impl_subcommandgroup_for_enum;
use syn::{parse_macro_input, DeriveInput, Ident, Meta};

mod command;
mod commands;
mod subcommand;
mod subcommandgroup;
mod utility;

#[proc_macro_error]
#[proc_macro_derive(Command, attributes(name, subcommandgroup, choice, channel_types, min, max))]
pub fn derive_commmand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let name_attr = attrs
        .iter()
        .find(|attr| attr.path.is_ident("name"))
        .unwrap_or_else(|| abort!(ident, "Command must specify a name via the \"name\" attribute"));
    let name_meta = name_attr
        .parse_meta()
        .unwrap_or_else(|_| abort!(name_attr, "Invalid \"name\" attribute"));
    let name = match name_meta {
        Meta::NameValue(value) => value.lit,
        _ => abort!(name_attr, "Invalid \"name\" attribute"),
    };
    let description = utility::get_description(attrs.as_slice())
        .unwrap_or_else(|| abort!(ident, "Command must specify a description via a docstring"));

    match data {
        syn::Data::Struct(ref data) => {
            impl_command_for_struct(ident, name, &description, options_for_struct_data(data))
        }
        syn::Data::Enum(ref data) => {
            impl_command_for_enum(ident, name, &description, subcommands_for_enum(data))
        }
        _ => abort!(ident, "Can only derive Command for structs (regular commands) or enums (commands with subcommands)"),
    }
}

#[proc_macro_error]
#[proc_macro_derive(SubCommand, attributes(name, choice, channel_types, min, max))]
pub fn derive_subcommmand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, ..
    } = parse_macro_input!(input);

    match data {
        syn::Data::Struct(ref data) => {
            impl_subcommand_for_struct(ident, options_for_struct_data(data))
        }
        _ => abort!(ident, "Can only derive SubCommand for structs"),
    }
}

#[proc_macro_error]
#[proc_macro_derive(SubCommandGroup, attributes(name))]
pub fn derive_subcommmandgroup(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, ..
    } = parse_macro_input!(input);

    match data {
        syn::Data::Enum(ref data) => {
            impl_subcommandgroup_for_enum(ident, subcommands_for_enum(data))
        }
        _ => abort!(ident, "Can only derive SubCommandGroup for enums"),
    }
}

#[proc_macro_error]
#[proc_macro_derive(Commands)]
pub fn derive_commands(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, ..
    } = parse_macro_input!(input);

    let (variant_identifier, field_type): (Vec<Ident>, Vec<proc_macro2::TokenStream>) = match data {
        syn::Data::Enum(ref data) => {
            data.variants.iter()
                .map(|variant| get_commands_variant_info(variant))
                .map(|variant_info| (variant_info.variant_identifier, variant_info.field_type))
                .multiunzip()
        }
        _ => abort!(ident, "Can only derive Commands for enums"),
    };
    quote!{
        #[serenity::async_trait]
        impl slashies::Commands for #ident {
            fn parse(
                _ctx: &serenity::prelude::Context,
                command: &serenity::model::prelude::application_command::ApplicationCommandInteraction,
            ) -> Result<Self, slashies::ParseError> {
                match command.data.name.as_ref() {
                    #(name if name == <#field_type as slashies::Command>::name() => Ok(Self::#variant_identifier(<#field_type as slashies::Command>::parse(command)?)),)*
                    _ => Err(slashies::ParseError::UnknownCommand),
                }
            }
        
            async fn invoke(
                &self,
                ctx: &serenity::prelude::Context,
                command_interaction: &serenity::model::prelude::application_command::ApplicationCommandInteraction,
            ) -> Result<(), slashies::InvocationError> {
                match self {
                    #(Self::#variant_identifier(command) => command.invoke(ctx, command_interaction).await,)*
                }
            }
        }
    }.into()
}

#[proc_macro_error]
#[proc_macro_derive(ApplicationCommandInteractionHandler)]
pub fn derive_application_command_interaction_handler(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, ..
    } = parse_macro_input!(input);

    let variant_identifier: Vec<Ident> = match data {
        syn::Data::Enum(ref data) => {
            data.variants.iter()
                .map(|variant| variant.ident.clone())
                .collect()
        }
        _ => abort!(ident, "Can only derive ApplicationCommandInteractionHandler for enums"),
    };
    quote!{
        #[serenity::async_trait]
        impl slashies::ApplicationCommandInteractionHandler for #ident {
            async fn invoke(
                &self,
                ctx: &serenity::prelude::Context,
                command_interaction: &ApplicationCommandInteraction,
            ) -> Result<(), InvocationError> {
                match self {
                    #(Self::#variant_identifier(command) => command.invoke(ctx, command_interaction).await,)*
                }
            }
        }
    }.into()
}