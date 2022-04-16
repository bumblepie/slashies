use itertools::Itertools;
use proc_macro::TokenStream;
use syn::Ident;
use quote::quote;

use crate::command::OptionTokenSections;

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