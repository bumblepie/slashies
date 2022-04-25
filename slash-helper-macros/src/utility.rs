use std::collections::HashMap;

use proc_macro_error::abort;
use quote::quote;
use serenity::model::channel::ChannelType;
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

pub(crate) fn channel_type_from_string(name: &str) -> Option<proc_macro2::TokenStream> {
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
            ChannelType::Directory.name(),
            quote! { serenity::model::channel::ChannelType::Directory },
        ),
        (
            ChannelType::Forum.name(),
            quote! { serenity::model::channel::ChannelType::Forum },
        ),
        (
            ChannelType::Unknown.name(),
            quote! { serenity::model::channel::ChannelType::Unknown },
        ),
    ]);
    all_types.get(name).map(|tokens| tokens.clone())
}

pub(crate) fn get_channel_types(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
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

pub(crate) fn get_description(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("doc"))
        .map(|attr| match attr.parse_meta() {
            Ok(meta) => (attr, meta),
            _ => abort!(attr, "Invalid docstring"),
        })
        .map(|(attr, meta)| match meta {
            Meta::NameValue(ref value) => match value.lit {
                Lit::Str(ref description) => description.value(),
                _ => abort!(attr, "Invalid docstring",),
            },
            _ => abort!(attr, "Invalid docstring",),
        })
}
