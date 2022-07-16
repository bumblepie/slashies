use crate::subcommand::SubCommandTokenSections;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

pub fn impl_subcommandgroup_for_enum(
    identifier: Ident,
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
    quote! {
        impl slashies::SubCommandGroup for #identifier {
            fn parse(option: Option<&serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>) -> Result<Self, slashies::ParseError> {
                let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = option
                    .ok_or(slashies::ParseError::MissingOption)?
                    .options
                    .iter()
                    .map(|option| (option.name.clone(), option.clone()))
                    .collect();

                #(if let Some(value) = #parse_fetch {
                    return Ok(Self::#variant_identifier(value));
                })*

                Err(slashies::ParseError::MissingOption)
            }

            fn register_sub_options(
                option: &mut serenity::builder::CreateApplicationCommandOption,
            ) -> &mut serenity::builder::CreateApplicationCommandOption {
                option
                #(.create_sub_option(#registration_fn))*
            }
        }
    }.into()
}
