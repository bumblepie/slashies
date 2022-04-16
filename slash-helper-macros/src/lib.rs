use command::{impl_command_for_struct, impl_command_for_enum, options_for_struct_data, sub_commands_for_enum};
use commands::get_commands_variant_info;
use itertools::Itertools;
use proc_macro::{self, TokenStream};
use quote::{quote};
use subcommand::impl_subcommand_for_struct;
use syn::{parse_macro_input, DeriveInput, Ident, Meta};

mod command;
mod commands;
mod subcommand;

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

#[proc_macro_derive(Commands)]
pub fn derive_commands(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data, ..
    } = parse_macro_input!(input);

    let (variant_identifier, field_type): (Vec<Ident>, Vec<proc_macro2::TokenStream>) = match data {
        syn::Data::Enum(ref data) => {
            data.variants.iter()
                .map(|variant| get_commands_variant_info(variant))
                .map(|variant_info| (variant_info.variant_identifier, variant_info.field_type))
                .multiunzip()
        }
        _ => panic!("Can only derive Commands for enums"),
    };
    quote!{
        impl Commands {
            pub fn parse(
                _ctx: &Context,
                command: &ApplicationCommandInteraction,
            ) -> Result<Self, ParseError> {
                match command.data.name.as_ref() {
                    #(name if name == <#field_type as slash_helper::Command>::name() => Ok(Self::#variant_identifier(<#field_type as slash_helper::Command>::parse(command)?)),)*
                    _ => Err(ParseError::UnknownCommand),
                }
            }
        
            pub async fn invoke(
                &self,
                ctx: &Context,
                command_interaction: &ApplicationCommandInteraction,
            ) -> Result<(), InvocationError> {
                match self {
                    #(Self::#variant_identifier(command) => command.invoke(ctx, command_interaction).await,)*
                }
            }
        }
    }.into()
}