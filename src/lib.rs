use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, GenericArgument, Lit, Meta,
    PathArguments, Type, TypePath,
};

#[allow(dead_code)]
trait CliConf {
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

fn get_delimiter_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if let Meta::NameValue(nv) = &attr.meta {
            if nv.path.is_ident("delimiter") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(CliConf, attributes(delimiter))]
pub fn derive_flags(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut parse_env = vec![];
    let mut parse_arg = vec![];
    let mut need_arg = vec![];
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            for f in fields_named.named.iter() {
                let field_name = &f.ident;
                let field_name_string = field_name.clone().unwrap().to_string();
                let var_name = field_name_string.to_uppercase();
                let arg_name = field_name_string.replace("_", "-");
                let field_is_vec = is_vec(&f.ty);

                let delimiter = get_delimiter_attribute(&f.attrs);

                let parse_env_value = quote! {
                    let value = value.parse().expect(&format!("Failed to parse environment variable {}", #var_name));
                };

                let parse_arg_value = quote! {
                    let value = arg.parse().expect(&format!("Failed to parse command-line argument {}", #arg_name));
                };

                let parse_env_op = if field_is_vec {
                    if let Some(delimiter) = delimiter {
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

                parse_arg.push(quote! (
                    #field_name_string => {
                        #parse_arg_op
                    }
                ));

                let need_arg_op = if is_bool(&f.ty) {
                    quote! (self.#field_name = true)
                } else {
                    quote! (need_value_for_name = Some(#field_name_string))
                };

                need_arg.push(quote! (
                    #arg_name => #need_arg_op,
                ))
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
