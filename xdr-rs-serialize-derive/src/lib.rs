#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::Meta;

#[proc_macro_derive(XDROut, attributes(array, discriminant))]
pub fn xdr_out_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_xdr_out_macro(&ast)
}

#[proc_macro_derive(XDRIn, attributes(array, discriminant))]
pub fn xdr_in_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_xdr_in_macro(&ast)
}

#[derive(Debug, Clone)]
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

fn get_meta_items(attr: &syn::Attribute, ident: &str) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.is_ident(ident) {
        match attr.parse_meta() {
            Ok(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}

fn get_array_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    get_meta_items(attr, "array")
}

fn get_discriminant_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    get_meta_items(attr, "discriminant")
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
                        index: i_val.base10_digits().parse::<i32>().unwrap(),
                        name: variant.ident.clone(),
                        e_type: None,
                    }),
                    _ => {}
                },
                _ => {}
            },
            (syn::Fields::Unnamed(un), None) => {
                let mut member_index: i32 = index;
                for meta_items in variant.attrs.iter().filter_map(get_discriminant_meta_items) {
                    for meta_item in meta_items {
                        match meta_item {
                            Meta(NameValue(ref m)) if m.path.is_ident("value") => match m.lit {
                                syn::Lit::Str(ref val) => {
                                    member_index = val.value().parse::<i32>().unwrap();
                                }
                                _ => {}
                            },
                            _ => {}
                        };
                    }
                }

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
                    index: member_index,
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

fn get_calls_enum_in_xdr(
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

fn get_calls_enum_in_json(
    data: &syn::DataEnum,
    enum_name: &syn::Ident,
) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let enums = get_enums(data)?;
    let mut result = Vec::new();
    for enu in enums.iter() {
        match (&enu.name, enu.unit, enu.index, &enu.e_type) {
            (name, true, i, None) => {
                result.push(
                    format!("{} => Ok({}::{}),", i, enum_name, name)
                        .parse()
                        .unwrap(),
                );
            }
            (name, false, i, Some(typ)) => {
                result.push(
                    format!(
                        "{} => {{let result = {}::read_json(enum_val.clone())?; Ok({}::{}(result))}},",
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
                        "{} => {{let result = <()>::read_json(enum_val.clone())?; Ok({}::{}(result))}},",
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

fn get_calls_enum_out_xdr(data: &syn::DataEnum) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let enums = get_enums(data)?;
    let mut result = Vec::new();
    for enu in enums.iter() {
        match (&enu.name, enu.unit, enu.index) {
            (name, true, i) => {
                result.push(
                    format!("{} => ({} as i32).write_xdr(out),", name, i)
                        .parse()
                        .unwrap(),
                );
            }
            (name, false, i) => {
                result.push(
                    format!(
                        "{}(ref val) => {{let mut written = 0; written += ({} as i32).write_xdr(out)?; written += val.write_xdr(out)?; Ok(written)}},",
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

fn get_calls_enum_out_json(data: &syn::DataEnum) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let enums = get_enums(data)?;
    let mut result = Vec::new();
    for enu in enums.iter() {
        match (&enu.name, enu.unit, enu.index) {
            (name, true, i) => {
                result.push(
                    format!("{} => ({} as i32).write_json(out),", name, i)
                        .parse()
                        .unwrap(),
                );
            }
            (name, false, i) => {
                result.push(
                    format!(
                        r#"{}(ref val) => {{let mut written = 0; written += out.write("{{\"type\":".as_bytes()).unwrap() as u64;  written += ({} as i32).write_json(out)?; written += out.write(",\"data\":".as_bytes()).unwrap() as u64; written +=  val.write_json(out)?; written += out.write("}}".as_bytes()).unwrap() as u64; Ok(written)}},"#,
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
                for meta_items in field.attrs.iter().filter_map(get_array_meta_items) {
                    for meta_item in meta_items {
                        match meta_item {
                            Meta(NameValue(ref m)) if m.path.is_ident("fixed") => match m.lit {
                                syn::Lit::Int(ref val) => {
                                    fixed = val.base10_digits().parse::<u32>().unwrap();
                                }
                                _ => {}
                            },
                            Meta(NameValue(ref m)) if m.path.is_ident("var") => match m.lit {
                                syn::Lit::Int(ref val) => {
                                    var = val.base10_digits().parse::<u32>().unwrap();
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

fn member_to_json_dict(mem: &Member, skip_name: bool) -> Result<String, ()> {
    let mut lines: Vec<String> = Vec::new();
    if !skip_name {
        let name_str = format!(
            r#"written += out.write("\"{}\":".as_bytes()).unwrap() as u64;"#,
            mem.name
        );
        lines.push(name_str);
    }

    let out = match (
        &mem.name,
        mem.fixed,
        mem.var,
        mem.v_type.to_string() == "String",
        mem.v_type.to_string().replace(" ", "") == "Vec<u8>",
    ) {
        (name, 0, 0, false, false) => format!("written += self.{}.write_json(out)?;", name),
        (name, fixed, 0, false, false) => format!(
            "written += write_fixed_array_json(&self.{}, {}, out)?;",
            name, fixed
        ),
        (name, fixed, 0, false, true) => format!(
            "written += write_fixed_opaque_json(&self.{}, {}, out)?;",
            name, fixed
        ),
        (name, 0, var, false, true) => format!(
            "written += write_var_opaque_json(&self.{}, {}, out)?;",
            name, var
        ),
        (name, 0, var, true, false) => format!(
            "written += write_var_string_json(self.{}.clone(), {}, out)?;",
            name, var
        ),
        (name, 0, var, false, false) => format!(
            "written += write_var_array_json(&self.{}, {}, out)?;",
            name, var
        ),
        _ => "".to_string(),
    };
    lines.push(out);
    Ok(lines.join("\n"))
}

fn get_calls_struct_out_json(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    let mut lines: Vec<String> = Vec::new();
    if members.len() == 1 && members[0].name == "t".to_string() {
        lines.push(member_to_json_dict(&members[0], true)?);
        return Ok(vec![lines.join("\n").parse().unwrap()]);
    }
    lines.push(r#"written += out.write("{".as_bytes()).unwrap() as u64;"#.to_string());
    if members.len() == 0 {
        lines.push(r#"written += out.write("}".as_bytes()).unwrap() as u64;"#.to_string());
        return Ok(vec![lines.join("\n").parse().unwrap()]);
    }
    let mem = members[0].clone();
    lines.push(member_to_json_dict(&mem, false)?);
    if members.len() == 1 {
        lines.push(r#"written += out.write("}".as_bytes()).unwrap() as u64;"#.to_string());
        return Ok(vec![lines.join("\n").parse().unwrap()]);
    }

    for mem in members[1..].iter() {
        lines.push(r#"written += out.write(",".as_bytes()).unwrap() as u64;"#.to_string());
        lines.push(member_to_json_dict(mem, false)?);
    }
    lines.push(r#"written += out.write("}".as_bytes()).unwrap() as u64;"#.to_string());
    Ok(vec![lines.join("\n").parse().unwrap()])
}

fn get_calls_struct_out_xdr(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
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
                (name, 0, var, false, true) => format!(
                    "written += write_var_opaque(&self.{}, {}, out)?;",
                    name, var
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

fn get_calls_struct_in_xdr(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
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

fn get_calls_struct_in_json(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    let obj_fun = |i: &proc_macro2::Ident| -> String {
        if members.len() == 1 && members[0].name == "t" {
            return "jval".to_string();
        }
        format!(
            r#"obj.ok_or_else(|| Error::invalid_json())?.get("{}").ok_or_else(|| Error::invalid_json())?"#,
            i
        )
    };
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var, &i.v_type) {
            (name, 0, 0, v_type) => format!(
                r#"let {}_result = {}::read_json({}.clone())?;"#,
                name,
                v_type.to_string().replace("<", "::<"),
                obj_fun(name)
            )
            .parse()
            .unwrap(),
            (name, fixed, 0, v_type) if v_type.to_string().replace(" ", "") != "Vec<u8>" => {
                format!(
                    r#"let {}_result: {} = read_fixed_array_json({}, {}.clone())?;"#,
                    name,
                    v_type,
                    fixed,
                    obj_fun(name)
                )
                .parse()
                .unwrap()
            }
            (name, 0, var, v_type) if v_type.to_string() == "String" => format!(
                r#"let {}_result: {} = read_var_string_json({}, {}.clone())?;"#,
                name,
                v_type,
                var,
                obj_fun(name)
            )
            .parse()
            .unwrap(),
            (name, 0, var, v_type) if v_type.to_string().replace(" ", "") != "Vec<u8>" => format!(
                r#"let {}_result: {} = read_var_array_json({}, {}.clone())?;"#,
                name,
                v_type,
                var,
                obj_fun(name)
            )
            .parse()
            .unwrap(),
            (name, fixed, 0, v_type) => format!(
                r#"let {}_result: {} = read_fixed_opaque_json({}, {}.clone())?;"#,
                name,
                v_type,
                fixed,
                obj_fun(name)
            )
            .parse()
            .unwrap(),
            (name, 0, var, v_type) => format!(
                r#"let {}_result: {} = read_var_opaque_json({}, {}.clone())?;"#,
                name,
                v_type,
                var,
                obj_fun(name)
            )
            .parse()
            .unwrap(),
            _ => "".to_string().parse().unwrap(),
        })
        .collect())
}

fn get_struct_build_in_xdr(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var) {
            (name, _, _) => format!("{}: {}_result.0,", name, name).parse().unwrap(),
        })
        .collect())
}

fn get_struct_build_in_json(data: &syn::DataStruct) -> Result<Vec<proc_macro2::TokenStream>, ()> {
    let members = get_members(data)?;
    Ok(members
        .iter()
        .map(|i| match (&i.name, i.fixed, i.var) {
            (name, _, _) => format!("{}: {}_result,", name, name).parse().unwrap(),
        })
        .collect())
}

fn impl_xdr_out_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let xdr_calls = get_calls_struct_out_xdr(data).unwrap();
            let json_calls = get_calls_struct_out_json(data).unwrap();
            quote! {
                impl XDROut for #name {
                    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
                        let mut written: u64 = 0;
                        #(#xdr_calls)*
                        Ok(written)
                    }

                    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
                        let mut written: u64 = 0;
                        #(#json_calls)*
                        Ok(written)
                    }
                }
            }
        }
        syn::Data::Enum(data) => {
            let xdr_matches = get_calls_enum_out_xdr(data).unwrap();
            let json_matches = get_calls_enum_out_json(data).unwrap();
            let names = std::iter::repeat(name);
            let names2 = std::iter::repeat(name);
            quote! {
                impl XDROut for #name {
                    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
                        match *self {
                            #(#names::#xdr_matches)*
                            _ => Err(Error::invalid_enum_value())
                        }
                    }

                    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
                        match *self {
                            #(#names2::#json_matches)*
                            _ => Err(Error::invalid_enum_value())
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
            let xdr_calls = get_calls_struct_in_xdr(data).unwrap();
            let json_calls = get_calls_struct_in_json(data).unwrap();
            let struct_build_xdr = get_struct_build_in_xdr(data).unwrap();
            let struct_build_json = get_struct_build_in_json(data).unwrap();
            quote! {
                impl XDRIn for #name {
                    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
                        let mut read: u64 = 0;
                        #(#xdr_calls)*
                        Ok((
                            #name {
                              #(#struct_build_xdr)*
                            },
                            read
                        ))
                    }

                    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
                        let obj = match &jval {
                            json::JsonValue::Object(o) => Some(o),
                            _ => None
                        };
                        #(#json_calls)*
                        Ok( #name {
                            #(#struct_build_json)*
                        })
                    }
                }

            }
        }
        syn::Data::Enum(data) => {
            let matches_xdr = get_calls_enum_in_xdr(data, name).unwrap();
            let matches_json1 = get_calls_enum_in_json(data, name).unwrap();
            let matches_json2 = get_calls_enum_in_json(data, name).unwrap();
            quote! {
                impl XDRIn for #name {
                    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
                        let enum_val = i32::read_xdr(buffer)?.0;
                        match enum_val {
                            #(#matches_xdr)*
                            _ => Err(Error::invalid_enum_value())
                        }
                    }

                    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
                        match jval {
                            json::JsonValue::Object(obj) =>  {
                                let enum_index = i32::read_json(obj.get("type").ok_or_else(|| Error::invalid_json())?.clone())?;
                                let enum_val = obj.get("data").ok_or_else(|| Error::invalid_json())?;
                                match enum_index {
                                    #(#matches_json1)*
                                    _ => Err(Error::invalid_enum_value())
                                }
                            },
                            json::JsonValue::Number(num) =>  {
                                let enum_index : i32 = f64::from(num) as i32;
                                let enum_val : json::JsonValue = json::JsonValue::new_object();
                                match enum_index {
                                    #(#matches_json2)*
                                    _ => Err(Error::invalid_enum_value())
                                }
                            },
                            _ => Err(Error::invalid_enum_value())
                        }
                    }
                }
            }
        }
        _ => panic!("XDRIn macro only works with enums and structs."),
    };

    gen.into()
}
