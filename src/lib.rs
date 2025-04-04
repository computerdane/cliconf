use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericArgument, LitChar, LitStr,
    Meta, MetaList, PathArguments, Type, TypePath,
};

#[allow(dead_code)]
trait Parse {
    fn parse_env(&mut self, vars: HashMap<String, String>);
    fn parse_args(&mut self, args: Vec<String>) -> Vec<String>;
}

fn is_bool(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.first() {
            if let PathArguments::None = segment.arguments {
                return segment.ident == "bool";
            }
        }
    }
    false
}

fn is_vec(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            if is_bool(&inner_ty) {
                                panic!("CliConf does not support Vec<bool>!");
                            }
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[derive(Default)]
struct CliconfAttrs {
    shorthand: Option<char>,
    delimiter: Option<String>,
}

fn get_meta<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Meta> {
    for attr in attrs {
        if attr.meta.path().is_ident(name) {
            return Some(&attr.meta);
        }
    }
    None
}

fn get_meta_list<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a MetaList> {
    if let Some(Meta::List(meta_list)) = get_meta(attrs, name) {
        return Some(&meta_list);
    }
    None
}

fn get_cliconf_attrs(attrs: &[Attribute]) -> CliconfAttrs {
    let mut result = CliconfAttrs::default();
    if let Some(meta_list) = get_meta_list(attrs, "cliconf") {
        meta_list
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("shorthand") {
                    let value = meta.value()?;
                    let c: LitChar = value.parse()?;
                    result.shorthand = Some(c.value());
                }
                if meta.path.is_ident("delimiter") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    result.delimiter = Some(s.value());
                }
                Ok(())
            })
            .expect("Failed to parse cliconf attribute");
    }
    result
}

#[proc_macro_derive(Parse, attributes(cliconf))]
pub fn derive_flags(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut parse_env = vec![];
    let mut parse_arg = vec![];
    let mut need_arg = vec![];
    let mut need_arg_shorthand = vec![];
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            for f in fields_named.named.iter() {
                let field_name = &f.ident;
                let field_name_string = field_name.clone().unwrap().to_string();
                let var_name = field_name_string.to_uppercase();
                let arg_name = field_name_string.replace("_", "-");
                let field_is_vec = is_vec(&f.ty);

                let cliconf_attrs = get_cliconf_attrs(&f.attrs);

                let parse_env_value = quote! {
                    let value = value.parse().expect(&format!("Failed to parse environment variable {}", #var_name));
                };

                let parse_arg_value = quote! {
                    let value = arg.parse().expect(&format!("Failed to parse command-line argument {}", #arg_name));
                };

                let parse_env_op = if field_is_vec {
                    if let Some(delimiter) = cliconf_attrs.delimiter {
                        quote! {
                            self.#field_name.clear();
                            for value in value.split(&#delimiter) {
                                #parse_env_value
                                self.#field_name.push(value);
                            }
                        }
                    } else {
                        quote! {}
                    }
                } else {
                    quote! {
                        #parse_env_value
                        self.#field_name = value;
                    }
                };

                parse_env.push(quote! {
                    if let Some(value) = vars.get(#var_name) {
                        #parse_env_op
                    }
                });

                let parse_arg_op = if field_is_vec {
                    quote! {
                        if !cleared_vecs.contains(#field_name_string) {
                            self.#field_name.clear();
                            cleared_vecs.insert(#field_name_string);
                        }
                        #parse_arg_value
                        self.#field_name.push(value);
                    }
                } else {
                    quote! {
                        #parse_arg_value
                        self.#field_name = value;
                    }
                };

                parse_arg.push(quote! {
                    #field_name_string => {
                        #parse_arg_op
                    }
                });

                let need_arg_op = if is_bool(&f.ty) {
                    quote! {
                        self.#field_name = true
                    }
                } else {
                    quote! {
                        need_value_for_name = Some(#field_name_string)
                    }
                };

                need_arg.push(quote! {
                    #arg_name => #need_arg_op,
                });

                if let Some(shorthand) = cliconf_attrs.shorthand {
                    let shorthand = shorthand.to_string();
                    need_arg_shorthand.push(quote! {
                        #shorthand => #need_arg_op,
                    });
                }
            }
        } else {
            panic!("CliConf can only be derived for structs with named fields");
        }
    } else {
        panic!("CliConf can only be derived for structs");
    };

    let expanded = quote! {
        impl #name {
            pub fn parse_env(&mut self, vars: std::collections::HashMap<String, String>) {
                #(#parse_env)*
            }

            pub fn parse_args(&mut self, args: Vec<String>) -> Vec<String> {
                let mut positionals = vec![];
                let mut need_value_for_name: Option<&str> = None;
                let mut as_positionals = false;
                let mut cleared_vecs = std::collections::HashSet::new();

                for arg in args {
                    if as_positionals {
                        positionals.push(arg);
                    } else if let Some(name) = need_value_for_name {
                        match name {
                            #(#parse_arg)*
                            _ => panic!("Unknown flag: --{name}")
                        };
                        need_value_for_name = None;
                    } else if arg == "-" {
                        // Some programs use "-" to signify that data will be read from
                        // stdin, so we treat it as a positional argument
                        positionals.push(arg);
                    } else if arg == "--" {
                        // "--" is a special flag that treats all of the remaining
                        // arguments as positional arguments
                        as_positionals = true;
                    } else if arg.starts_with("--") {
                        let name = &arg[2..];
                        match name {
                            #(#need_arg)*
                            _ => panic!("Unknown flag: --{name}")
                        }
                    } else if arg.starts_with("-") {
                        let name = &arg[1..];
                        match name {
                            #(#need_arg_shorthand)*
                            _ => panic!("Unknown flag: -{name}")
                        }
                    } else {
                        positionals.push(arg);
                    }
                }

                positionals
            }
        }
    };

    TokenStream::from(expanded)
}
