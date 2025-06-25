use std::collections::HashMap;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use serde_derive::{Deserialize, Serialize};
use syn::__private::quote;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Mappings{
    minecraft_version: String,
    classes: Vec<HashMap<String, String>>,
}

#[proc_macro]
pub fn get_mappings(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path:syn::LitStr = syn::parse(item).expect("Expected a path");
    let path = path.value();

    let path = {
        let mut new_path = std::path::PathBuf::new();
        new_path.push(std::env!("CARGO_MANIFEST_DIR"));
        new_path.push(path);
        new_path
    };

    let mut out_tokens = TokenStream::new();
    for mapping in std::fs::read_dir(path).expect("Could not read specified mapping dir") {
        let mut token_stream = proc_macro2::TokenStream::new();
        let mapping = mapping.expect("Could not read mapping inside mapping dir");
        let version = mapping.file_name().into_string().expect("folder inside mapping dir is not valid UTF-8");

        let mut json = mapping.path();
        json.push(format!("{version}.json"));
        let json = json;

        let json = std::fs::read(json).expect("Could not read version mapping");
        let json:Mappings = serde_json::from_slice(json.as_slice()).expect("Could not parse json version mappings");

        let mut transmuted_mappings = HashMap::new();
        for (i, mapping) in json.classes.iter().enumerate() {
            let mapping_array = mapping
                .iter()
                .map(|(key, value)|  {
                    quote::quote! {
                        Mapping{name: #key, value: #value}
                    }
                });
            let ident = quote::format_ident!("MAPPING_ARRAY_{i}");
            token_stream.append_all(quote::quote! {
                static #ident:&'static [Mapping] = &[#(#mapping_array),*];
            });

            for (name, value) in mapping {
                match transmuted_mappings.entry(name)
                    .or_insert_with(HashMap::new)
                    .insert(value, ident.clone()) {
                    None => {},
                    Some(prev_ident) => {
                        let err_ident = quote::format_ident!("warn_mapping_array_{i}_{name}");
                        let message = format!("In version '{version}' '{name}' mappings have a duplicated value of '{value}'. Previous value: {prev_ident}, New Value: {ident}");
                        token_stream.append_all(quote::quote! {
                            mod #err_ident {
                                #[must_use = #message]
                                struct compile_warning;
                                #[allow(dead_code)]
                                fn trigger_warning () { compile_warning; }
                            }
                        })
                    }
                }
            }
        }
        let hash_map = transmuted_mappings.into_iter().fold(TokenStream::new(), |mut init, (key, value)|{
            let value = value.into_iter().fold(
                TokenStream::new(),
                |mut init, (key, value)|{
                    init.append_all(quote::quote! {
                        #key => #value,
                    });
                    init
                }
            );
            init.append_all(quote::quote! {
                #key => phf_macros::phf_map! { #value },
            });
            init
        });
        token_stream.append_all(quote::quote! { phf_macros::phf_map! { #hash_map } });

        let version = &json.minecraft_version;
        out_tokens.append_all(quote::quote! { #version => { #token_stream }, });
    }

    quote::quote! {
        struct Mapping{
            name: &'static str,
            value: &'static str
        }
        static MAPPINGS:phf::Map<&'static str, phf::Map<&'static str, phf::Map<&'static str, &'static [Mapping]>>> = {
            phf_macros::phf_map!{ #out_tokens }
        };
    }.into()
}