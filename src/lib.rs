use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput, Fields, PathArguments, Type, TypePath};

#[allow(dead_code)]
trait CliConf {
    fn parse_env(&mut self, vars: HashMap<String, String>);
    fn parse_args(&mut self, args: Vec<String>) -> Vec<String>;
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == 1 {
            let segment = &path.segments[0];
            if let PathArguments::None = segment.arguments {
                return segment.ident == "bool";
            }
        }
    }
    return false;
}

#[proc_macro_derive(CliConf)]
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

                parse_env.push(quote! {
                    if let Some(value) = vars.get(#var_name) {
                        self.#field_name = value.parse().expect(&format!("Failed to parse environment variable {}", #var_name));
                    }
                });

                parse_arg.push(quote! (
                    #field_name_string => self.#field_name = arg.parse().expect(&format!("Failed to parse arg {}", #arg_name)),
                ));

                let set_or_need = if is_bool_type(&f.ty) {
                    quote! (self.#field_name = true)
                } else {
                    quote! (need_value_for_name = Some(#field_name_string))
                };

                need_arg.push(quote! (
                    #arg_name => #set_or_need,
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

                for arg in args {
                    if as_positionals {
                        positionals.push(arg);
                    } else if let Some(name) = need_value_for_name {
                        match name {
                            #(#parse_arg)*
                            _ => {}
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
                            _ => {}
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
