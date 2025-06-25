use std::collections::HashMap;
use std::io::Write;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Mappings{
    minecraft_version: String,
    classes: Vec<HashMap<String, String>>,
}

pub fn get_mappings(inpath: &std::path::Path) -> String {
    let mut out_tokens = String::new();
    let mut out_map = phf_codegen::Map::new();
    for (i, mapping) in std::fs::read_dir(inpath).expect(&format!("Could not read specified mapping dir at path '{}'", inpath.display())).enumerate() {
        out_tokens.push_str(&format!("mod map_{i} {{"));
        let mapping = mapping.expect("Could not read mapping inside mapping dir: ");
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
                .fold(String::new(), |mut init, (key, value)|  {
                    init.push_str(&format!(r#"Mapping{{name: "{key}", value: "{value}",}},
                    "#));
                    init
                });
            let ident = format!("MAPPING_ARRAY_{i}");
            out_tokens.push_str(&format!(r#"
static {ident}:&'static [Mapping] = &[
{mapping_array}
];
            "#));

            for (name, value) in mapping {
                match transmuted_mappings.entry(name)
                    .or_insert_with(HashMap::new)
                    .insert(value, ident.clone()) {
                    None => {},
                    Some(prev_ident) => {
                        let err_ident = format!("warn_mapping_array_{i}_{name}");
                        let message = format!("In version '{version}' '{name}' mappings have a duplicated value of '{value}'. Previous value: {prev_ident}, New Value: {ident}");
                        out_tokens.push_str(&format!(r#"
                            mod {err_ident} {{
                                #[must_use = "{message}"]
                                struct CompileWarning;
                                #[allow(dead_code)]
                                fn trigger_warning () {{ CompileWarning; }}
                            }}
                        "#));
                    }
                }
            }
        }
        let hash_map = transmuted_mappings.into_iter().fold(phf_codegen::Map::new(), |mut init, (key, value)|{
            let map = value.into_iter().fold(
                phf_codegen::Map::new(),
                |mut init, (key, value)|{
                    init.entry(key, value);
                    init
                }
            );
            let map = map.build().to_string();
            out_tokens.push_str(&format!(r#"mod {key} {{
    pub static MAPPING:phf::Map<&'static str, &'static [Mapping]> = {map};
}}
            "#));
            init.entry(key, format!("{key}::MAPPING"));
            init
        });
        let hash_map = hash_map.build();

        out_tokens.push_str(&format!(r#"
    pub static MAPPING:phf::Map<&'static str, phf::Map<&'static str, &'static [Mapping]>> = {hash_map};
}}
        "#));
        out_map.entry(json.minecraft_version, format!("map_{i}::MAPPING"));
    }

    let out_map = out_map.build();
    out_tokens.push_str(&format!(r#"
        pub struct Mapping{{
            name: &'static str,
            value: &'static str
        }}
        pub static MAPPINGS:phf::Map<&'static str, phf::Map<&'static str, phf::Map<&'static str, &'static [Mapping]>>> = {out_map};
    "#));

    out_tokens
}
fn main() {
    let path = {
        let mut new_path = std::path::PathBuf::new();
        new_path.push(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        new_path.push("../mappings/mappings");
        new_path
    };

    let mappings = get_mappings(path.as_path());

    let path = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());

    file.write_all(mappings.as_bytes()).unwrap();
}