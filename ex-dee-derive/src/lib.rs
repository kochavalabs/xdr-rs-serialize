#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::Meta;

#[proc_macro_derive(XDROut, attributes(array))]
pub fn xdr_out_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_xdr_out_macro(&ast)
}

#[derive(Debug)]
struct Member {
    pub name: proc_macro2::Ident,
    pub fixed: u32,
    pub var: u32,
}

fn get_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "array" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}

fn get_members(fields: &syn::Fields) -> Result<Vec<Member>, ()> {
    match fields {
        syn::Fields::Named(ref named) => {
            let mut result = Vec::new();
            for field in named.named.iter() {
                let mut fixed: u32 = 0;
                let mut var: u32 = 0;
                for meta_items in field.attrs.iter().filter_map(get_meta_items) {
                    for meta_item in meta_items {
                        match meta_item {
                            Meta(NameValue(ref m)) if m.ident == "fixed" => match m.lit {
                                syn::Lit::Int(ref val) => {
                                    fixed = val.value() as u32;
                                }
                                _ => {}
                            },
                            Meta(NameValue(ref m)) if m.ident == "var" => match m.lit {
                                syn::Lit::Int(ref val) => {
                                    var = val.value() as u32;
                                }
                                _ => {}
                            },
                            _ => {}
                        };
                    }
                }
                result.push(Member {
                    name: field.ident.clone().unwrap(),
                    fixed: fixed,
                    var: var,
                });
            }
            Ok(result)
        }
        _ => Err(()),
    }
}

fn impl_xdr_out_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let members = match &ast.data {
        syn::Data::Struct(data_struct) => get_members(&data_struct.fields).unwrap(),
        _ => panic!("Contract macro only works with trait declarations!"),
    };
    let calls: Vec<proc_macro2::TokenStream> = members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var) {
            (name, 0, 0) => format!("written += self.{}.write_xdr(out)?;", name)
                .parse()
                .unwrap(),
            (name, fixed, 0) => format!(
                "written += write_fixed_array(&self.{}, {}, out)?;",
                name, fixed
            )
            .parse()
            .unwrap(),
            _ => "asdf".to_string().parse().unwrap(),
        })
        .collect();
    let gen = quote! {
        impl<Out: Write> XDROut<Out> for #name {
            fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
                let mut written: u64 = 0;
                #(#calls)*
                Ok(written)
            }
        }
    };
    gen.into()
}
