use proc_macro_error::abort;
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
                abort!(
                    variant,
                    "Variants of a Commands enum must have exactly one unnamed field"
                );
            }
            let field = &fields[0];
            let variant_identifier = variant.ident.clone();
            let field_type = field.ty.to_token_stream();
            CommandsVariantInfo {
                variant_identifier,
                field_type,
            }
        }
        _ => abort!(
            variant,
            "Variants of a Commands enum must have exactly one unnamed field"
        ),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn examples_fail_with_correct_error() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/commands/*.rs");
    }
}
