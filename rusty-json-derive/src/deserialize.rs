use std::{cell::RefCell, rc::Rc};
use quote::quote;

use crate::TokenLayer;

pub(crate) fn try_from_json_macro_impl<'a>(type_name: &syn::Ident, fields: impl Iterator<Item = (&'a syn::Ident, Rc<RefCell<TokenLayer>>)>) -> proc_macro2::TokenStream
{
    let (idents, fields): (Vec<&syn::Ident>, Vec<Rc<RefCell<TokenLayer>>>) = fields.map(|(i, f)| (i, f)).unzip();

    let fields = fields.into_iter()
    .zip(idents.clone())
    .map
    (
        |(field, ident)|
        {
            token_layer_into_token_stream_recursive(field.clone(), Some(ident.to_string()), None, field)
        }
    )
    .collect::<Vec<_>>();

    let quote = quote!
    (
        impl TryFrom<JsonObject> for #type_name
        {
            type Error = String;

            fn try_from(json: JsonObject) -> Result<Self, Self::Error>
            {
                let values = match json
                {
                    JsonObject::Object { fields } => fields,
                    _ => return Err(format!("An object was expected while given json object was `{}`", &json)),
                };

                Ok
                (
                    Self
                    {
                        #( #idents : #fields, )*
                    }
                )
            }
        }
    );

    // println!("{}", quote);

    quote
}

fn token_layer_into_token_stream_recursive<'a>(layer: Rc<RefCell<TokenLayer>>, ident_str: Option<String>, ident_index: Option<usize>, parent_layer: Rc<RefCell<TokenLayer>>) -> proc_macro2::TokenStream
{
    match &*layer.borrow()
    {
        TokenLayer::Ident { ident, next } => token_layer_into_token_stream_recursive(next.clone(), Some(ident.to_string()), None, layer.clone()),
        TokenLayer::Tuple { items, size: _ } =>
        {
            let tokens = items.into_iter().enumerate()
            .map
            (
                |(i, f)|
                {
                    let tokens = token_layer_into_token_stream_recursive(f.clone(), ident_str.clone(), Some(i), layer.clone());
                    let is_parent_ident = if let TokenLayer::Ident { ident: _, next: _ } = &*parent_layer.borrow() { true } else { false };

                    match &*f.borrow()
                    {
                        TokenLayer::Tuple { items: _, size: _ } =>
                        {
                            if is_parent_ident
                            {
                                quote!
                                (
                                    if let Some(JsonObject::Array { array: values }) = values.get(#ident_str) { let values = &values[#i]; #tokens }
                                    else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                                )
                            }
                            else if let TokenLayer::Tuple { items: _, size: _ } = &*parent_layer.borrow()
                            {
                                quote!
                                (
                                    if let JsonObject::Array { array: values } = values { let values = &values[#i]; #tokens }
                                    else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                                )
                            }
                            else { tokens }
                        },
                        _ =>
                        {
                            let tokens = quote!
                            (
                                if let JsonObject::Array { array: values } = values { #tokens }
                                else { return Err(format!("Cannot deserialize `{}` to a tuple !", values)) }
                            );

                            if is_parent_ident
                            {
                                quote!
                                (
                                    if let Some(values) = values.get(#ident_str) { #tokens }
                                    else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                                )
                            }
                            else { tokens }
                        },
                    }
                }
            );

            quote!(( #( { #tokens }, )* ))
        },
        TokenLayer::Array { typ: _ } => todo!(),
        TokenLayer::Item { typ } =>
        {
            let ident_str = ident_str.expect("token_layer_into_token_stream_recursive(…, ident_str, …) was supposed to be Some(…) while it was None !");

            match &*parent_layer.borrow()
            {
                TokenLayer::Ident { ident: _, next: _ } =>
                {
                    quote!
                    (
                        if let Some(value) = values.get(#ident_str)
                        {
                            match value.try_into()
                            {
                                Ok(value) => value,
                                Err(msg) => return Err(format!("Failed to parse `{}` with error `{}`", #ident_str, msg)),
                            }
                        }
                        else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                    )
                },
                TokenLayer::Tuple { items: _, size: _ } =>
                {
                    let index = ident_index.expect("token_layer_into_token_stream_recursive(…, ident_index, …) was supposed to be Some(…) while it was None !");

                    quote!
                    (
                        let option: Result<#typ, _> = (&values[#index]).try_into();

                        match option
                        {
                            Ok(value) => value.clone(),
                            Err(msg) => return Err(format!("Failed to parse `{}` with error `{}`", #ident_str, msg)),
                        }
                    )
                },
                TokenLayer::Array { typ: _ } => todo!(),
                TokenLayer::Item { typ: _ } => panic!("`{0}` cannot be parent of `{0}`", stringify!(TokenLayer::Item)),
            }

            /* if let Some(index) = ident_index
            {
                quote!
                (
                    if let Some(value) = fields.get(#ident_str)
                    {
                        if let JsonObject::Array { array } = value
                        {
                            let option: Result<#typ, _> = (&array[#index]).try_into();

                            match option
                            {
                                Ok(value) => value.clone(),
                                Err(msg) => return Err(format!("Failed to parse `{}` with error `{}`", #ident_str, msg)),
                            }
                        }
                        else { return Err(format!("Cannot deserialize `{}` to a tuple !", value)) }
                    }
                    else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                )
            }
            else
            {
                quote!
                (
                    if let Some(value) = fields.get(#ident_str)
                    {
                        match value.try_into()
                        {
                            Ok(value) => value,
                            Err(msg) => return Err(format!("Failed to parse `{}` with error `{}`", #ident_str, msg)),
                        }
                    }
                    else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                )
            } */
        },
    }
}
