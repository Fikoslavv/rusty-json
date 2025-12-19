mod serialize;
mod deserialize;

use proc_macro::TokenStream;
use quote::quote;
use std::rc::Rc;
use std::cell::RefCell;

use serialize::{into_json_macro_impl, to_json_macro_impl};

use crate::deserialize::try_from_json_macro_impl;

#[derive(Debug, Clone)]
pub(crate) enum TokenLayer
{
    Ident { ident: syn::Ident, next: Rc<RefCell<TokenLayer>> },
    Tuple { items: Vec<Rc<RefCell<TokenLayer>>>, size: usize },
    Array { typ: Rc<RefCell<TokenLayer>> },
    Item { typ: syn::Type },
}

#[proc_macro_derive(JsonSerialize)]
pub fn json_serialize_macro_derive(input: TokenStream) -> TokenStream
{
    let tree = syn::parse(input).unwrap();
    let (type_name, fields) = token_stream_into_syn_tree(&tree);

    let serialize_mut = into_json_macro_impl(type_name, fields.clone());
    let serialize_ref = to_json_macro_impl(type_name, fields);

    quote!(#serialize_mut #serialize_ref).into()
}

#[proc_macro_derive(IntoJson)]
pub fn into_json_macro_derive(input: TokenStream) -> TokenStream
{
    let tree = syn::parse(input).unwrap();
    let (type_name, fields) = token_stream_into_syn_tree(&tree);
    into_json_macro_impl(type_name, fields).into()
}

#[proc_macro_derive(ToJson)]
pub fn to_json_macro_derive(input: TokenStream) -> TokenStream
{
    let tree = syn::parse(input).unwrap();
    let (type_name, fields) = token_stream_into_syn_tree(&tree);
    to_json_macro_impl(type_name, fields).into()
}

#[proc_macro_derive(JsonDeserialize)]
pub fn json_deserialize_macro_derive(input: TokenStream) -> TokenStream
{
    let tree = syn::parse(input).unwrap();
    let (type_name, fields) = token_stream_into_syn_tree(&tree);
    try_from_json_macro_impl(type_name, fields).into()
}

fn token_stream_into_syn_tree(tree: &syn::DeriveInput) -> (&syn::Ident, impl Iterator<Item = (&syn::Ident, Rc<RefCell<TokenLayer>>)> + Clone)
{
    let (ident, iter) = match &tree.data
    {
        syn::Data::Struct(str) =>
        {
            if let syn::Fields::Named(named) = &str.fields
            {
                (
                    &tree.ident,
                    named.named.iter().map(|f| (f.ident.as_ref().unwrap(), f)).map(|(i, f)| (i, field_into_token_layers(f)))
                )
            }
            else { panic!("syn::Data:Struct did not have syn::Fields::Named !") }
        },
        syn::Data::Enum(_) => panic!("Enums are not currently supported !"),
        syn::Data::Union(_) => panic!("Unions are not currently supported !"),
    };

    (ident, iter.map(|(i, f)| (i, Rc::new(RefCell::new(f)))))
}

pub(crate) fn field_into_token_layers(field: &syn::Field) -> TokenLayer
{
    let fields_name_str = field.ident.as_ref().unwrap().to_string();
    let layer_root = Rc::new(RefCell::new(TokenLayer::Ident { ident: field.ident.as_ref().unwrap().clone(), next: Rc::new(RefCell::new(TokenLayer::Item { typ: field.ty.clone() })) }));
    let mut types: std::collections::VecDeque<(syn::Type, Rc<RefCell<TokenLayer>>)> = std::collections::VecDeque::with_capacity(1);
    types.push_front((field.ty.clone(), layer_root.clone()));

    while let Some((typ, layer)) = types.pop_front()
    {
        match typ
        {
            syn::Type::Path(_) =>
            {
                match &mut *layer.borrow_mut()
                {
                    &mut TokenLayer::Ident { ident: _, next: _ } | &mut TokenLayer::Array { typ: _ } | TokenLayer::Item { typ: _ } => (),
                    &mut TokenLayer::Tuple { ref mut items, size: _ } => items.push(Rc::new(RefCell::new(TokenLayer::Item { typ }))),
                }
            },
            syn::Type::Array(data) =>
            {
                let array = Rc::new(RefCell::new(TokenLayer::Array { typ: Rc::new(RefCell::new(TokenLayer::Item { typ: (*data.elem).clone() })) }));
                types.push_back((*data.elem.clone(), array.clone()));

                match &mut *layer.borrow_mut()
                {
                    &mut TokenLayer::Ident { ident: _, ref mut next } => *next = array,
                    &mut TokenLayer::Array { ref mut typ } => *typ = array,
                    &mut TokenLayer::Tuple { ref mut items, size: _ } => items.push(array),
                    _ => panic!("Unsupported type is parent of an array !"),
                }
            },
            syn::Type::BareFn(_) => panic!("Bare functions are not supported !"),
            syn::Type::Group(_) => panic!("Groups are not supported !"),
            syn::Type::ImplTrait(_) => panic!("Impl traits are not supported !"),
            syn::Type::Infer(_) => panic!("Infered types are not supported !"),
            syn::Type::Macro(_) => panic!("Macros are not supported !"),
            syn::Type::Never(_) => panic!("Never is not supported !"),
            syn::Type::Ptr(_) => panic!("Pointers are not supported !"),
            syn::Type::Reference(_) => panic!("References are not supported !"),
            syn::Type::Slice(_) => panic!("Slices are not supported !"),
            syn::Type::TraitObject(_) => panic!("Trait objects are not supported !"),
            syn::Type::Tuple(data) =>
            {
                let tuple = Rc::new(RefCell::new(TokenLayer::Tuple { size: data.elems.len(), items: Vec::with_capacity(data.elems.len()) }));

                for i in 0..data.elems.len()
                {
                    types.push_back((data.elems[i].clone(), tuple.clone()));
                }

                match &mut *layer.borrow_mut()
                {
                    &mut TokenLayer::Ident { ident: _, ref mut next } => *next = tuple,
                    &mut TokenLayer::Array { ref mut typ } => *typ = tuple,
                    &mut TokenLayer::Tuple { ref mut items, size: _ } => items.push(tuple),
                    _ => panic!("Unsupported type is parent of a tuple !"),
                }
            },
            syn::Type::Verbatim(_) => panic!("Verbatims are not supported !"),
            _ => panic!("Field `{}` is of unsupported type !", fields_name_str),
        }
    }

    layer_root.borrow().clone()
}
