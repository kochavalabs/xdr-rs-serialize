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

#[derive(Debug)]
struct Enum {
    pub name: proc_macro2::Ident,
    pub unit: bool,
    pub index: u32,
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

fn get_enums(data: &syn::DataEnum) -> Result<Vec<Enum>, ()> {
    let mut members = Vec::new();
    for variant in &data.variants {
        match (&variant.fields, &variant.discriminant) {
            (syn::Fields::Unit, Some(expr)) => match expr.1 {
                syn::Expr::Lit(ref e_lit) => match e_lit.lit {
                    syn::Lit::Int(ref i_val) => members.push(Enum {
                        unit: true,
                        index: i_val.value() as u32,
                        name: variant.ident.clone(),
                    }),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
    Ok(members)
}

fn get_calls_enum(data: &syn::DataEnum) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let enums = get_enums(data)?;
    let mut result = Vec::new();
    for enu in enums.iter() {
        match (&enu.name, enu.unit, enu.index) {
            (name, true, i) => {
                result.push(
                    format!("{} => {}.write_xdr(out),", name, i)
                        .parse()
                        .unwrap(),
                );
            }
            _ => {
                return Err(());
            }
        }
    }
    Ok(result)
}

fn get_members(data: &syn::DataStruct) -> Result<Vec<Member>, ()> {
    match data.fields {
        syn::Fields::Named(ref named) => {
            let mut members = Vec::new();
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
                members.push(Member {
                    name: field.ident.clone().unwrap(),
                    fixed: fixed,
                    var: var,
                });
            }
            Ok(members)
        }
        _ => Err(()),
    }
}

fn get_calls_struct(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
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
            (name, 0, var) => format!("written += write_var_array(&self.{}, {}, out)?;", name, var)
                .parse()
                .unwrap(),
            _ => "".to_string().parse().unwrap(),
        })
        .collect())
}

fn impl_xdr_out_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let calls = get_calls_struct(data).unwrap();
            quote! {
                impl<Out: Write> XDROut<Out> for #name {
                    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
                        let mut written: u64 = 0;
                        #(#calls)*
                        Ok(written)
                    }
                }
            }
        }
        syn::Data::Enum(data) => {
            let matches = get_calls_enum(data).unwrap();
            let names = std::iter::repeat(name);
            quote! {
                impl<Out: Write> XDROut<Out> for #name {
                    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
                        match *self {
                            #(#names::#matches)*
                            _ => Err(Error::InvalidEnumValue)
                        }
                    }
                }
            }
        }
        _ => panic!("Contract macro only works with trait declarations!"),
    };
    gen.into()
}
