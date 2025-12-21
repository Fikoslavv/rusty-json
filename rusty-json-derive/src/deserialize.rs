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

                Ok ( Self { #( #idents : #fields, )* })
            }
        }

        impl TryFrom<&JsonObject> for #type_name
        {
            type Error = String;

            fn try_from(json: &JsonObject) -> Result<Self, Self::Error>
            {
                let values = match json
                {
                    JsonObject::Object { fields } => fields,
                    _ => return Err(format!("An object was expected while given json object was `{}`", &json)),
                };

                Ok(Self { #( #idents : #fields, )* })
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
            let ident_str_opt = ident_str;
            let ident_str = ident_str_opt.clone().expect("token_layer_into_token_stream_recursive(…, ident_str, …) was supposed to be Some(…) while it was None !");
            let mapper: Box<dyn Fn((usize, &Rc<RefCell<TokenLayer>>)) -> proc_macro2::TokenStream> = match &*parent_layer.borrow()
            {
                TokenLayer::Ident { ident: _, next: _ } => Box::new
                (
                    |(i, f): (usize, &Rc<RefCell<TokenLayer>>)|
                    {
                        let tokens = token_layer_into_token_stream_recursive(f.clone(), ident_str_opt.clone(), Some(i), layer.clone());

                        match &*f.borrow()
                        {
                            TokenLayer::Ident { ident: _, next: _ } => panic!("rusty_json::JsonDeserialize error - a tuple cannot have an Ident as one of its children !"),
                            TokenLayer::Tuple { items: _, size: _ } => quote!
                            (
                                if let Some(JsonObject::Array { array: values }) = values.get(#ident_str)
                                {
                                    if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                    else { return Err(format!("An array was expected to deserialize a tuple within a tuple !")) }
                                }
                                else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                            ),
                            TokenLayer::Array { typ: _ } => quote!
                            (
                                if let Some(JsonObject::Array { array: values }) = values.get(#ident_str)
                                {
                                    if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                    else { return Err(format!("An array was expected to deserialize an array within a tuple !")); }
                                }
                                else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                            ),
                            TokenLayer::Item { typ: _ } => quote!
                            (
                                if let Some(JsonObject::Array { array: values }) = values.get(#ident_str) { #tokens }
                                else { return Err(format!("Cannot deserialize `{}` to a tuple !", #ident_str)); }
                            ),
                        }
                    }
                ),
                TokenLayer::Tuple { items: _, size: _ } => Box::new
                (
                    |(i, f) : (_, _)|
                    {
                        let tokens = token_layer_into_token_stream_recursive(f.clone(), ident_str_opt.clone(), Some(i), layer.clone());

                        match &*f.borrow()
                        {
                            TokenLayer::Ident { ident: _, next: _ } => panic!("rusty_json::JsonDeserialize error - a tuple cannot have an Ident as one of its item !"),
                            TokenLayer::Tuple { items: _, size: _ } => quote!
                            (
                                if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                            ),
                            TokenLayer::Array { typ: _ } => quote!
                            (
                                if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                else { return Err(format!("Value `{}` was not found !", #ident_str)); }
                            ),
                            TokenLayer::Item { typ: _ } => tokens,
                        }
                    }
                ),
                TokenLayer::Array { typ: _ } => Box::new
                (
                    |(i, f): (_, _)|
                    {
                        let tokens = token_layer_into_token_stream_recursive(f.clone(), ident_str_opt.clone(), Some(i), layer.clone());

                        let tokens = match &*f.borrow()
                        {
                            TokenLayer::Ident { ident: _, next: _ } => panic!("rusty_json::JsonDeserialize error - a tuple cannot have an Ident as one of its item !"),
                            TokenLayer::Array { typ: _ } => quote!
                            (
                                if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                else { return Err(format!("Only an array can be deserialized to a tuple !")); }
                            ),
                            TokenLayer::Tuple { items: _, size: _ } => quote!
                            (
                                if let JsonObject::Array { array: values } = &values[#i] { #tokens }
                                else { return Err(format!("Only an array can be deserialized to a tuple !")); }
                            ),
                            TokenLayer::Item { typ: _ } => tokens,
                        };

                        quote!
                        (
                            if let JsonObject::Array { array: values } = values { #tokens }
                            else { return Err(format!("An array was expected to deserialize a tuple !")); }
                        )
                    }
                ),
                TokenLayer::Item { typ: _ } => panic!("rusty_json::JsonDeserialize error - a tuple cannot have an item as its parent !"),
            };

            let tokens = items.into_iter().enumerate().map(mapper);

            quote!(( #( { #tokens }, )* ))
        },
        TokenLayer::Array { typ } =>
        {
            let tokens = token_layer_into_token_stream_recursive(typ.clone(), ident_str.clone(), None, layer.clone());
            let tokens = quote!
            (
                let mut vec: Vec<_> = Vec::with_capacity(values.len());

                for values in values { vec.push(#tokens); }

                std::array::from_fn(|i| vec[i])
            );

            match &*parent_layer.borrow()
            {
                TokenLayer::Item { typ: _ } => panic!("Serialization macro error !  Item cannot be the parent of an array !"),
                TokenLayer::Tuple { items: _, size: _ } => tokens,
                TokenLayer::Array { typ: _ } => quote!
                (
                    if let JsonObject::Array { array: values } = values { #tokens }
                    else { return Err(format!("An array was expected to deserialize to an array !")); }
                ),
                TokenLayer::Ident { ident: _, next: _ } =>
                {
                    let ident_str = ident_str.expect("token_layer_into_token_stream_recursive(…, ident_str, …) was supposed to be Some(…) while it was None !");

                    quote!
                    (
                        if let Some(values) = values.get(#ident_str)
                        {
                            if let JsonObject::Array { array: values } = values { #tokens }
                            else { return Err(format!("Field `{}` was expected to be an array while it was `{}`", #ident_str, values)) }
                        }
                        else { return Err(format!("Field `{}` was not found !", #ident_str)) }
                    )
                },
            }
        },
        TokenLayer::Item { typ } =>
        {
            let ident_str = ident_str.expect("token_layer_into_token_stream_recursive(…, ident_str, …) was supposed to be Some(…) while it was None !");

            match &*parent_layer.borrow()
            {
                TokenLayer::Ident { ident: _, next: _ } => quote!
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
                ),
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
                TokenLayer::Array { typ: _ } => quote!(values.try_into().unwrap()),
                TokenLayer::Item { typ: _ } => panic!("`{0}` cannot be parent of `{0}`", stringify!(TokenLayer::Item)),
            }
        },
    }
}
