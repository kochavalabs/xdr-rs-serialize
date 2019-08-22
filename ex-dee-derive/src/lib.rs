#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::Meta;

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
    pub e_type: Option<proc_macro2::Ident>,
    pub unit: bool,
    pub index: i32,
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
    let mut index: i32 = 0;
    for variant in &data.variants {
        match (&variant.fields, &variant.discriminant) {
            (syn::Fields::Unit, Some(expr)) => match expr.1 {
                syn::Expr::Lit(ref e_lit) => match e_lit.lit {
                    syn::Lit::Int(ref i_val) => members.push(Enum {
                        unit: true,
                        index: i_val.value() as i32,
                        name: variant.ident.clone(),
                        e_type: None,
                    }),
                    _ => {}
                },
                _ => {}
            },
            (syn::Fields::Unnamed(un), None) => {
                let types: Vec<_> = un
                    .unnamed
                    .iter()
                    .filter_map(|f| match f.ty.clone() {
                        syn::Type::Path(t_path) => Some(t_path.path.segments),
                        _ => None,
                    })
                    .flatten()
                    .map(|s| s.ident)
                    .collect();
                let ident = match types.len() {
                    0 => None,
                    1 => Some(types[0].clone()),
                    _ => panic!("Cannot have a union with more than one type."),
                };
                members.push(Enum {
                    unit: false,
                    index: index,
                    name: variant.ident.clone(),
                    e_type: ident,
                });
                index += 1;
            }
            _ => {}
        }
    }
    Ok(members)
}

fn get_calls_enum_in(
    data: &syn::DataEnum,
    enum_name: &syn::Ident,
) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let enums = get_enums(data)?;
    let mut result = Vec::new();
    for enu in enums.iter() {
        match (&enu.name, enu.unit, enu.index, &enu.e_type) {
            (name, true, i, None) => {
                result.push(
                    format!("{} => Ok(({}::{}, 4)),", i, enum_name, name)
                        .parse()
                        .unwrap(),
                );
            }
            (name, false, i, Some(typ)) => {
                result.push(
                    format!(
                        "{} => {{let result = {}::read_xdr(&buffer[4..])?; Ok(({}::{}(result.0), result.1 + 4))}},",
                        i,
                        typ.to_string().replace("<", "::<"),
                        enum_name,
                        name
                    )
                    .parse()
                    .unwrap(),
                );
            }
            (name, false, i, None) => {
                result.push(
                    format!(
                        "{} => {{let result = <()>::read_xdr(&buffer[..])?; Ok(({}::{}(result.0), result.1 + 4))}},",
                        i,
                        enum_name,
                        name
                    )
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

fn get_calls_enum_out(data: &syn::DataEnum) -> Result<Vec<proc_macro2::TokenStream>, ()> {
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
        .map(|i| {
            match (
                &i.name,
                i.fixed,
                i.var,
                i.v_type.to_string() == "String",
                i.v_type.to_string().replace(" ", "") == "Vec<u8>",
            ) {
                (name, 0, 0, false, false) => format!("written += self.{}.write_xdr(out)?;", name)
                    .parse()
                    .unwrap(),
                (name, fixed, 0, false, false) => format!(
                    "written += write_fixed_array(&self.{}, {}, out)?;",
                    name, fixed
                )
                .parse()
                .unwrap(),
                (name, fixed, 0, false, true) => format!(
                    "written += write_fixed_opaque(&self.{}, {}, out)?;",
                    name, fixed
                )
                .parse()
                .unwrap(),
                (name, 0, var, true, false) => format!(
                    "written += write_var_string(self.{}.clone(), {}, out)?;",
                    name, var
                )
                .parse()
                .unwrap(),
                (name, 0, var, false, false) => {
                    format!("written += write_var_array(&self.{}, {}, out)?;", name, var)
                        .parse()
                        .unwrap()
                }
                _ => "".to_string().parse().unwrap(),
            }
        })
        .collect())
}

fn get_calls_struct_in(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var, &i.v_type) {
            (name, 0, 0, v_type) => format!(
                "let {}_result = {}::read_xdr(&buffer[read as usize..])?; read += {}_result.1;",
                name,
                v_type.to_string().replace("<", "::<"),
                name
            )
            .parse()
            .unwrap(),

            (name, fixed, 0, v_type) if v_type.to_string().replace(" ", "") != "Vec<u8>" => {
                format!(
                "let {}_result: ({}, u64) = read_fixed_array({}, &buffer[read as usize..])?; read += {}_result.1;",
                name, v_type, fixed, name
            )
                .parse()
                .unwrap()
            }
            (name, 0, var, v_type) if v_type.to_string() == "String" => format!(
                "let {}_result: ({}, u64) = read_var_string({}, &buffer[read as usize..])?; read += {}_result.1;",
                name, v_type, var, name
            )
            .parse()
            .unwrap(),
            (name, 0, var, v_type) if v_type.to_string().replace(" ", "") != "Vec<u8>" => format!(
                "let {}_result: ({}, u64) = read_var_array({}, &buffer[read as usize..])?; read += {}_result.1;",
                name, v_type, var, name
            )
            .parse()
            .unwrap(),
            (name, fixed, 0, v_type) => format!(
                "let {}_result: ({}, u64) = read_fixed_opaque({}, &buffer[read as usize..])?; read += {}_result.1;",
                name, v_type, fixed, name
            )
            .parse()
            .unwrap(),
            (name, 0, var, v_type) => format!(
                "let {}_result: ({}, u64) = read_var_opaque({}, &buffer[read as usize..])?; read += {}_result.1;",
                name, v_type, var, name
            )
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
            (name, _, _) => format!("{}: {}_result.0,", name, name).parse().unwrap(),
        })
        .collect())
}

fn impl_xdr_out_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let calls = get_calls_struct_out(data).unwrap();
            quote! {
                impl XDROut for #name {
                    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
                        let mut written: u64 = 0;
                        #(#calls)*
                        Ok(written)
                    }
                }
            }
        }
        syn::Data::Enum(data) => {
            let matches = get_calls_enum_out(data).unwrap();
            let names = std::iter::repeat(name);
            quote! {
                impl XDROut for #name {
                    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
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
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let calls = get_calls_struct_in(data).unwrap();
            let struct_build = get_struct_build_in(data).unwrap();
            quote! {
                impl XDRIn for #name {
                    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
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
        syn::Data::Enum(data) => {
            let matches = get_calls_enum_in(data, name).unwrap();
            quote! {
                impl XDRIn for #name {
                    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
                        let enum_val = i32::read_xdr(buffer)?.0;
                        match enum_val {
                            #(#matches)*
                            _ => Err(Error::InvalidEnumValue)
                        }
                    }
                }
            }
        }
        _ => panic!("XDRIn macro only works with enums and structs."),
    };

    gen.into()
}
