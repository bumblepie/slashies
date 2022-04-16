use quote::ToTokens;
use syn::{Ident, Variant};

pub struct CommandsVariantInfo {
    pub variant_identifier: Ident,
    pub field_type: proc_macro2::TokenStream,
}

pub fn get_commands_variant_info(variant: &Variant) -> CommandsVariantInfo {
    match variant.fields {
        syn::Fields::Unnamed(ref fields) => {
            let fields = &fields.unnamed;
            if fields.len() != 1 {
                panic!("Variants of a Command enum must be a tuple of length 1, containing only the subcommand");
            }
            let field = &fields[0];
            let variant_identifier = variant.ident.clone();
            let field_type = field.ty.to_token_stream();
            CommandsVariantInfo {
                variant_identifier,
                field_type,
            }
        }
        _ => panic!("Can only derive Command for enums with unnamed tuple variants"),
    }
}
