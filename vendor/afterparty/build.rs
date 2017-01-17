extern crate case;
//extern crate hyper;
extern crate serde_codegen;
extern crate serde_json;
extern crate glob;

use std::collections::BTreeMap;
use case::CaseExt;
//use hyper::Client;
use std::env;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::path::Path;

/// generate an enum of Events
fn main() {
    for entry in glob::glob("data/**/*.json").expect("Failed to read glob pattern") {
        println!("cargo:rerun-if-changed={}", entry.unwrap().display());
    }

    let mut buf = String::new();
    let mut event_list = File::open("events.txt").unwrap();
    event_list.read_to_string(&mut buf).unwrap();
    let events = buf.lines().collect::<Vec<&str>>();
    println!("events {:#?}", events);
    if let Ok(_) = env::var("FETCH_PAYLOAD_DATA") {
        fetch_payload_data(&events).unwrap();
    }
    if let Ok(_) = env::var("SKIP_GENERATE") {
        return;
    }
    generate_enum(&events).unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let src = Path::new(&out_dir).join("events.rs.in");
    let dst = Path::new(&out_dir).join("events.rs");
    serde_codegen::expand(&src, &dst).unwrap();
}

fn fetch_payload_data(events: &Vec<&str>) -> Result<()> {
    println!("fetching payload data for events {:#?}", events);
    let data_dir = Path::new("data");
    /*let client = Client::new();
    for event in events {
        let src = format!("https://raw.githubusercontent.com/github/developer.github.\
                           com/master/lib/webhooks/{}.payload.json",
                          event);
        let mut res = client.get(&src)
                            .send()
                            .unwrap();
        let mut buf = String::new();
        try!(res.read_to_string(&mut buf));
        let outfile = data_dir.join(format!("{}.json", event));
        let mut f = try!(File::create(outfile));
        try!(f.write_all(buf.as_bytes()));
    }*/
    Ok(())
}

fn generate_enum(events: &Vec<&str>) -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("events.rs.in");
    let mut f = try!(File::create(&dest_path));

    // synthensize a type that can represent arbitrary json in deployment payloads
    // as well has a type which we may implement a default value
    try!(f.write_all(b"#[derive(Debug, Deserialize)]
pub struct Value { pub json: serde_json::Value }

"));

    try!(f.write_all(b"#[derive(Debug, Deserialize)]
pub enum Event {
"));
    let mut defs = BTreeMap::new();
    for event in events {
        let mut data = try!(File::open(format!("data/{}.json", event)));
        let mut buf = String::new();
        try!(data.read_to_string(&mut buf));
        let parsed: serde_json::Value = serde_json::from_str(&buf).unwrap();
        let enum_name = container_name(event);
        try!(f.write_all(format!("  {} ", enum_name).as_bytes()));
        try!(f.write_all(b"{"));

        match parsed {
            serde_json::Value::Object(obj) => {
                for (k, v) in obj {
                    try!(f.write_all(format!(r#"
      #[serde(rename="{}")]
      {}: {},"#, k,
                                             field_name(&k),
                                             value(&enum_name, &mut defs, &k, &v))
                                         .as_bytes()))
                }
            }
            _ => (),
        }
        try!(f.write_all(b"
  },
"));
    }
    try!(f.write_all(b"}

"));

    try!(print_structs(&mut f, defs, &mut vec![], 0));

    Ok(())
}

fn print_structs(f: &mut File,
                 defs: BTreeMap<String, serde_json::Value>,
                 generated: &mut Vec<String>,
                 depth: usize)
                 -> Result<()> {
    let mut aux = BTreeMap::new();
    for (struct_name, json) in defs.iter() {
        if generated.contains(&struct_name) {
            continue;
        }
        println!("struct {}", struct_name);
        try!(f.write_all(format!("
#[derive(Default, Debug, Deserialize)]
pub struct {} ",
                                 struct_name)
                             .as_bytes()));
        try!(f.write_all(b"{"));
        match json {
            &serde_json::Value::Object(ref obj) => {
                for (k, v) in obj {
                    // fields are renamed to enable deserialization of fields
                    // that are also reserved works in rust
                    try!(f.write_all(format!(r#"
    #[serde(rename="{}")]
    pub {}: {},"#, k,
                                             field_name(&k),
                                             value(&struct_name, &mut aux, &k, &v))
                                         .as_bytes()))
                }
            }
            _ => (),
        }

        try!(f.write_all(b"
}
"));
        generated.push(struct_name.clone());
    }
    if !aux.is_empty() {
        try!(print_structs(f, aux, generated, depth + 1));
    }
    Ok(())
}

fn value(container: &String, defs: &mut BTreeMap<String, serde_json::Value>, k: &str, j: &serde_json::Value) -> String {
    match j {
        &serde_json::Value::I64(_) => "i64".to_owned(),
        &serde_json::Value::U64(_) => "u64".to_owned(),
        &serde_json::Value::F64(_) => "f64".to_owned(),
        &serde_json::Value::String(_) => "String".to_owned(),
        &serde_json::Value::Bool(_) => "bool".to_owned(),
        &serde_json::Value::Array(ref jv) => {
            if jv.is_empty() {
                "Vec<String>".to_owned() // this is just a guess!
            } else {
                format!("Vec<{}>", value(&container, defs, k, &jv[0]))
            }
        }
        obj @ &serde_json::Value::Object(_) => {
            if "payload" == k {
                // payloads may contain any arbitrary json structure
                // it is unsafe to assume a fixed type of map, string, ect
                "Value".to_owned()
            } else {
                // avoid recusive types by compormising on an alternative name
                let struct_name = match container_name(k) {
                    ref recursive if recursive == container => format!("{}Inner", recursive),
                    ref exists if defs.contains_key(exists) && defs[exists] != *obj => {
                        format!("{}{}", container, exists)
                    }
                    valid => valid,
                };
                defs.insert(struct_name.clone(), obj.clone());
                struct_name
            }
        }
        &serde_json::Value::Null => "Option<String>".to_owned(),
    }
}

fn container_name(field: &str) -> String {
    if "self" == field {
        "SelfLink".to_owned()
    } else {
        field.to_camel()
    }
}

/// works around conflicts with reservied words
fn field_name(s: &str) -> String {
    // todo: this is only as robust as it needs to be
    // this list is not complete
    let reserved = vec!["ref", "self", "type"];
    if reserved.contains(&s) {
        format!("_{}", s)
    } else {
        s.to_owned()
    }
}
