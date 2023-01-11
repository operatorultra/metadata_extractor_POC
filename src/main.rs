use std::collections::HashMap;

use infer::MatcherType;

use serde_json::Value;

use xml::reader::{EventReader, XmlEvent};

mod extractors;

fn main() {
    let path_to_file = "./test_files/test.ai";
    // let file: File = File::open(&path_to_file).expect("no file found");

    let file = std::fs::read(&path_to_file).unwrap();

    let filedata = infer::get_from_path(&path_to_file).expect("no file data");
    let mime_type = filedata.expect("unknown or corrupted data").mime_type();
    let matcher_type = filedata.expect("unknown or corrupted data").matcher_type();

    match matcher_type {
        MatcherType::Image => {
            let metadata =
                extractors::exif_extractor::Metadata::get_metadata(file, String::from(mime_type));
            println!("{metadata:#?} & {matcher_type:?}");
        }
        MatcherType::App => todo!(),
        MatcherType::Archive => {
            let metadata =
                extractors::exif_extractor::Metadata::get_metadata(file, String::from(mime_type));
            // Create a new EventReader
            let parser = EventReader::from_str(&metadata.xmp);

            // Create a Value object to store the JSON object
            let mut json = Value::Object(serde_json::Map::new());
            // let mut map = HashMap::new();

            // Iterate over the events in the XML string
            for event in parser {
                match event {
                    Ok(XmlEvent::StartElement {
                        name,
                        attributes,
                        namespace,
                    }) => {
                        // If the event is a start element, store the element name as a key in the JSON object
                        // let key = name.local_name;
                        // println!("{name:#?} {attributes:#?} {namespace:#?}");
                        // json[key] = Value::String(String::new());

                        println!("NAME {:#?}", name.local_name);
                        // map.insert(name.local_name, String::new());
                    }
                    Ok(XmlEvent::Characters(data)) => {
                        // If the event is characters, store the data as a value in the JSON object
                        // let _value = data.trim();
                        // let _key = json.as_object_mut().unwrap().keys().last().unwrap();
                        // let key = map.keys().last().unwrap();
                        // map.insert(key.clone(), data.trim().to_string());
                        println!("DATA {data:#?}");
                    }
                    // Ok(X)=> {},
                    _ => {}
                }
                // println!("{:#?}", map);
            }

            // println!("{:#?}", fields);
        }
        MatcherType::Audio => todo!(),
        MatcherType::Book => todo!(),
        MatcherType::Doc => todo!(),
        MatcherType::Font => todo!(),
        MatcherType::Text => todo!(),
        MatcherType::Video => todo!(),
        MatcherType::Custom => todo!(),
    }
}
