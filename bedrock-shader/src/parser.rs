use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until},
    character::complete::alphanumeric0,
    multi::separated_list0,
    sequence::separated_pair,
    IResult,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::Attribute;

use crate::{Entry, EntryVariant, Group, Index, Output, Vertex};

#[derive(Debug)]
pub struct ShaderParser {
    pub vertex_buffers: Vec<Vertex>,
    pub index_buffers: Vec<Index>,
    pub bind_groups: Vec<Group>,
    pub outputs: Vec<Output>,
}

impl ShaderParser {
    pub fn new() -> Self {
        Self {
            vertex_buffers: vec![],
            index_buffers: vec![],
            bind_groups: vec![],
            outputs: vec![],
        }
    }

    pub fn process_field(&mut self, field: syn::Field) {
        if field.attrs.is_empty() {
            return;
        }
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = field.ty.to_token_stream();
        for attr in &field.attrs {
            self.process_field_attr(attr.clone(), &field_name, field_type.clone());
        }
    }

    pub fn process_field_attr(
        &mut self,
        attr: Attribute,
        field_name: &str,
        field_type: TokenStream2,
    ) {
        let ident = attr.path.get_ident().unwrap().to_string();
        let inner = attr.tokens.to_string();
        match ident.as_str() {
            "texture2d" => {
                let args = inner_args(&inner);
                self.bind_groups.last_mut().unwrap().entries.push(Entry {
                    ident: field_name.to_string(),
                    ty: field_type.to_string(),
                    variant: EntryVariant::Texture2D {
                        width: args.get("width").unwrap().parse().unwrap(),
                        height: args.get("height").unwrap().parse().unwrap(),
                    },
                })
            }
            "uniform" => {
                let args = inner_args(&inner);
                self.bind_groups.last_mut().unwrap().entries.push(Entry {
                    ident: field_name.to_string(),
                    ty: field_type.to_string(),
                    variant: EntryVariant::Uniform {
                        size: args.get("size").unwrap().parse().unwrap(),
                    },
                });
            }
            "group" => {
                self.bind_groups.push(Group {
                    ident: format!("group_{}", self.bind_groups.len()),
                    vertex: inner.contains("vertex"),
                    fragment: inner.contains("fragment"),
                    entries: vec![],
                });
            }
            "vertex" => self.vertex_buffers.push(Vertex {}),
            "output" => self.outputs.push(Output {}),
            _ => panic!("Unknown attribute: {}", ident),
        }
    }
}

fn inner_args(inner: &str) -> HashMap<&str, &str> {
    let inner = inner.get(1..inner.len() - 1).unwrap();
    let args = crate::parser::comma_tuple(inner).unwrap().1;
    let args = args
        .into_iter()
        .map(|args| key_val(args).unwrap().1)
        .collect::<HashMap<_, _>>();
    args
}

pub fn comma_tuple(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(tag(", "), take_till(|c| c == ',' || c == ')'))(input)
}

pub fn key_val(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(take_until(" "), tag(" = "), alphanumeric0)(input)
}

// // Returns a parser which applies the given parser and then consumes 1 byte off the remainder
// fn take1<'a, O, F>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
// where
//     F: FnMut(&'a str) -> IResult<&'a str, O>,
// {
//     move |input: &'a str| {
//         let (input, output) = parser(input)?;
//         Ok((take(1usize)(input)?.1, output))
//     }
// }
