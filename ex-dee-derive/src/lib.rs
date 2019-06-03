#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::Meta;
use quote::ToTokens;

#[proc_macro_derive(XDROut, attributes(array))]
pub fn xdr_out_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_xdr_out_macro(&ast)
}

#[proc_macro_derive(XDRIn, attributes(array))]
pub fn xdr_in_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_xdr_in_macro(&ast)
}

#[derive(Debug)]
struct Member {
    pub name: proc_macro2::Ident,
    pub v_type: proc_macro2::TokenStream,
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
    let mut index: u32 = 0;
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
            (syn::Fields::Unnamed(_), _) => {
                members.push(Enum {
                    unit: false,
                    index: index,
                    name: variant.ident.clone(),
                });
                index += 1;
            }
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
            (name, false, i) => {
                result.push(
                    format!(
                        "{}(ref val) => {{{}.write_xdr(out)?; val.write_xdr(out)}},",
                        name, i
                    )
                    .parse()
                    .unwrap(),
                );
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
                    v_type: field.ty.clone().into_token_stream(),
                });
            }
            Ok(members)
        }
        _ => Err(()),
    }
}

fn get_calls_struct_out(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
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

fn get_calls_struct_in(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var, &i.v_type) {
            (name, 0, 0, v_type) => format!("let {}_result = {}::read_xdr(buffer)?; read += {}_result.1;", name, v_type, name)
                .parse()
                .unwrap(),
            _ => "".to_string().parse().unwrap(),
        })
        .collect())
}
fn get_struct_build_in(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var) {
            (name, 0, 0) => format!("{}: {}_result.0,", name, name)
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
            let calls = get_calls_struct_out(data).unwrap();
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
        _ => panic!("XDROut macro only works with enums and structs."),
    };
    gen.into()
}

fn impl_xdr_in_macro(ast: &syn::DeriveInput) -> TokenStream {
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let name = &ast.ident;
            let calls = get_calls_struct_in(data).unwrap();
            let struct_build = get_struct_build_in(data).unwrap();
            quote! {
                impl<In: Read> XDRIn<In> for #name {
                    fn read_xdr(buffer: &mut In) -> Result<(Self, u64), Error> {
                        let mut read: u64 = 0;
                        #(#calls)*
                        Ok((
                            #name {
                              #(#struct_build)*
                            },
                            read
                        ))
                    }
                }

            }
        }
        _ => panic!("XDRIn macro only works with enums and structs."),
    };

    gen.into()
}
