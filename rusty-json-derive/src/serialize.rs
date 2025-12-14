use std::cell::RefCell;
use std::rc::Rc;
use quote::quote;

use crate::{TokenLayer, field_into_token_layers};

pub(crate) fn into_json_macro_impl<'a>(type_name: &syn::Ident, fields: impl Iterator<Item = &'a syn::Field>) -> proc_macro2::TokenStream
{
    let fields = fields_into_token_stream_iter(fields, false);

    quote!
    {
        impl Into<JsonObject> for #type_name
        {
            fn into(self) -> JsonObject
            {
                let mut fields: std::collections::BTreeMap<String, JsonObject> = std::collections::BTreeMap::new();

                #( fields.insert(#fields); )*

                JsonObject::Object { fields }
            }
        }
    }
}

pub(crate) fn to_json_macro_impl<'a>(type_name: &syn::Ident, fields: impl Iterator<Item = &'a syn::Field>) -> proc_macro2::TokenStream
{
    let fields = fields_into_token_stream_iter(fields, true);

    quote!
    {
        impl Into<JsonObject> for &#type_name
        {
            fn into(self) -> JsonObject
            {
                let mut fields: std::collections::BTreeMap<String, JsonObject> = std::collections::BTreeMap::new();

                #( fields.insert(#fields); )*

                JsonObject::Object { fields }
            }
        }
    }
}

fn fields_into_token_stream_iter<'a>(fields: impl Iterator<Item = &'a syn::Field>, should_clone_items: bool) -> impl Iterator<Item = proc_macro2::TokenStream>
{
    fields.into_iter()
    .map(|f| (f.ident.as_ref().unwrap().to_string(), f))
    .map(|(ident, f)| (ident, field_into_token_layers(f)))
    .map(|(ident, t)| (ident, Rc::new(RefCell::new(t))))
    .map(move |(ident, t)| (ident, token_layer_into_token_stream_recursive(t, None, should_clone_items)))
    .map(|(ident, json)| quote!(#ident.to_string(), #json))
}

fn token_layer_into_token_stream_recursive(layer: Rc<RefCell<TokenLayer>>, ident: Option<proc_macro2::TokenStream>, require_cloning_items: bool) -> proc_macro2::TokenStream
{
    match &*layer.borrow()
    {
        TokenLayer::Item =>
        {
            let ident = ident.expect("token_layer_into_token_stream_recursive(…, ident) was supposed to be Some(…) while it was None !");
            if require_cloning_items { quote!(#ident.clone().into()) } else { quote!(#ident.into()) }
        },
        TokenLayer::Ident { ident, next } => token_layer_into_token_stream_recursive(next.clone(), Some(quote!(self.#ident)), require_cloning_items),
        TokenLayer::Tuple { items, size } =>
        {
            let tokens = items.into_iter().enumerate()
            .map(|(i, x)| (syn::Index::from(i), x))
            .map(|(i, x)| token_layer_into_token_stream_recursive(x.clone(), Some(quote!(#ident.#i)), require_cloning_items))
            .collect::<Vec<_>>();

            quote!
            (
                {
                    let mut array = Vec::with_capacity(#size);
                    #( array.push(#tokens); )*
                    JsonObject::Array { array }
                }
            )
        },
        TokenLayer::Array { typ } =>
        {
            let ident_item = quote!(item);
            let tokens = match &*typ.borrow()
            {
                TokenLayer::Item => quote!(#ident_item.into()),
                TokenLayer::Tuple { items: _, size: _ } => token_layer_into_token_stream_recursive(typ.clone(), Some(ident_item), require_cloning_items),
                TokenLayer::Array { typ: _ } => token_layer_into_token_stream_recursive(typ.clone(), Some(ident_item), require_cloning_items),
                _ => panic!("Unexpected token found in an array !"),
            };

            quote!({ JsonObject::Array { array: #ident.iter().map(|item| { #tokens }).collect::<Vec<_>>() } })
        },
    }
}
