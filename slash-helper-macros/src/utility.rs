use proc_macro_error::abort;
use quote::quote;
use syn::{Attribute, Lit, Meta, NestedMeta};

pub(crate) fn get_choices(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
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

pub(crate) fn get_channel_types(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    attrs.iter().find(|attr| attr.path.is_ident("channel_types"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid \"channel_types\" attribute"),
        })
        .map(|(attr, meta)| match meta {
            Meta::List(list) => {
                list.nested.iter().map(|nested| match nested {
                    NestedMeta::Meta(Meta::Path(path)) => quote!{ #path },
                    _ => abort!(nested, "Invalid channel type"),
                }).collect()
            },
            _ => abort!(attr, "Invalid \"channel_types\" attribute. Attribute must be of the form channel_types(type1, type2...)"),
        }
    ).map(|channel_types: Vec<_>| {
        quote! { .channel_types(&[#(#channel_types,)*]) }
    })
}

pub(crate) fn get_description(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("doc"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid description docstring"),
        })
        .map(|(attr, meta)| match meta {
            Meta::NameValue(ref value) => match value.lit {
                Lit::Str(ref description) => description.value(),
                _ => abort!(attr, "Invalid description docstring",),
            },
            _ => abort!(attr, "Invalid description docstring",),
        })
}

pub(crate) fn get_minimum_value(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("min"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid \"min\" attribute"),
        })
        .map(|(attr, meta)| match meta {
            Meta::NameValue(name_value) => match name_value.lit {
                Lit::Int(value) => quote! { .min_int_value(#value) },
                Lit::Float(value) => quote! { .min_number_value(#value) },
                _ => abort!(
                    name_value,
                    "Only integer and floating point number \"min\" values are supported"
                ),
            },
            _ => abort!(
                attr,
                "Invalid \"min\" attribute. Attribute must be of the form #[min = value]"
            ),
        })
}

pub(crate) fn get_maximum_value(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("max"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid \"max\" attribute"),
        })
        .map(|(attr, meta)| match meta {
            Meta::NameValue(name_value) => match name_value.lit {
                Lit::Int(value) => quote! { .max_int_value(#value) },
                Lit::Float(value) => quote! { .max_number_value(#value) },
                _ => abort!(
                    name_value,
                    "Only integer and floating point number \"max\" values are supported"
                ),
            },
            _ => abort!(
                attr,
                "Invalid \"miax\" attribute. Attribute must be of the form #[max = value]"
            ),
        })
}
