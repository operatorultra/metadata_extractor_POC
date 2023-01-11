use std::{collections::HashMap, io::Cursor, str};

use exif::{Tag, Value};
use pdf::{file::File, object::Resolve};

const XMP_TAG: u16 = 700;
const IPTC_TAG: u16 = 33723;

#[derive(Debug)]
pub struct Metadata {
    // tags --> which ones are currently in place?
    // createdISO8601: publication_date
    pub title: String,
    pub author: String,
    pub width: String,
    pub height: String,
    pub resolution: String, // dpi (if dots per inch is 0 set it to 72)
    pub make: String,
    pub model: String,
    pub flash_found: String,
    pub copyright: String,
    pub description: String,
    pub gps: String,
    pub xmp: String,
    pub iptc: String,
    pub subject_area: String,
    pub thumbnails: String,
    pub original_document_id: String,
}

impl Metadata {
    pub fn get_metadata(file: Vec<u8>, mime_type: String) -> Metadata {
        let mut title = String::new();
        let mut author = String::new();
        let mut width = String::new();
        let mut height = String::new();
        let mut resolution = HashMap::new();

        let mut copyright = String::new();
        let mut description = String::new();
        let mut make = String::new();
        let mut model = String::new();
        let mut flash_found = String::new();
        let mut gps = HashMap::new();

        let mut subject_area = HashMap::new();

        let mut thumbnails = Vec::new();

        let mut original_document_id: String = String::new();

        let mut xmp = String::new();
        let mut iptc = String::new();

        // log(&format!("Getting metadata from a {}", mime_type));
        if mime_type == "application/pdf" || mime_type == "application/postscript" {
            // PDF + AI (PDF based) can contain 'PDF Info' and XMP

            let pdf_file = File::from_data(file).unwrap();

            if let Some(ref info_dict) = pdf_file.trailer.info_dict {
                for (key, value) in info_dict {
                    if let Ok(pdf_string_value) = value.as_string() {
                        if let Ok(decoded_value) = pdf_string_value.as_str() {
                            // log(&format!("FOUND PDF INFO {}: {}", key, decoded_value));
                            match key.as_str() {
                                "Title" => {
                                    title = String::from(decoded_value);
                                }

                                "Author" => {
                                    author = String::from(decoded_value);
                                }

                                "Subject" => {
                                    description = String::from(decoded_value);
                                }

                                _ => {
                                    // log(&format!("Unknown PDF INFO {}: {}", key, decoded_value));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(pdf_metadata_stream_ref) = pdf_file.get_root().metadata {
                if let Ok(resolved_stream) = pdf_file.get(pdf_metadata_stream_ref) {
                    if let Ok(resolved_stream_data) = resolved_stream.data() {
                        if let Ok(metadata_as_str) = str::from_utf8(resolved_stream_data) {
                            // log(&format!("Found PDF Metadata: {}", metadata_as_str));
                            xmp = String::from(metadata_as_str);
                        }
                    }
                }
            }
        } else {
            let exifreader = exif::Reader::new();

            let exif = exifreader
                .read_from_container(&mut Cursor::new(file))
                .unwrap();

            for field in exif.fields() {
                let field_tag_number = field.tag.number();
                let tag = field.tag;

                match field_tag_number {
                    XMP_TAG => {
                        if let Value::Byte(value) = &field.value {
                            let value = std::str::from_utf8(&value).unwrap();
                            xmp = value.to_string().to_string();
                        }
                    }
                    IPTC_TAG => {
                        if let Value::Undefined(value, _) = &field.value {
                            let value = std::str::from_utf8(&value).unwrap();
                            iptc = value.to_string().to_string();
                        }
                    }
                    _ => match tag {
                        Tag::Copyright => {
                            copyright = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::ImageDescription => {
                            description = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::Make => {
                            make = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::Model => {
                            model = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::Flash => {
                            flash_found = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::ImageWidth | Tag::PixelXDimension => {
                            width = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::ImageLength | Tag::PixelYDimension => {
                            height = field
                                .display_value()
                                .with_unit(&exif)
                                .to_string()
                                .to_owned();
                        }
                        Tag::XResolution => {
                            resolution.insert(
                                "x".to_string(),
                                field.display_value().with_unit(&exif).to_string(),
                            );
                        }
                        Tag::YResolution => {
                            resolution.insert(
                                "y".to_string(),
                                field.display_value().with_unit(&exif).to_string(),
                            );
                        }
                        Tag::GPSLatitude => {
                            gps.insert(
                                "latitude".to_string(),
                                field.display_value().with_unit(&exif).to_string(),
                            );
                        }
                        Tag::GPSLongitude => {
                            gps.insert(
                                "longitude".to_string(),
                                field.display_value().with_unit(&exif).to_string(),
                            );
                        }
                        Tag::SubjectArea | Tag::SubjectLocation => {
                            let value = &field.value;
                            subject_area.insert("x", value.get_uint(0).unwrap());
                            subject_area.insert("y", value.get_uint(1).unwrap());
                            if let Some(width_or_diameter) = value.get_uint(2) {
                                if let Some(height) = value.get_uint(3) {
                                    subject_area.insert("width", width_or_diameter);
                                    subject_area.insert("height", height);
                                } else {
                                    subject_area.insert("diameter", width_or_diameter);
                                }
                            }
                        }

                        _ => println!(""),
                    },
                }

                // println!(
                //     "{:?} {:?} {}",
                //     field.tag,
                //     field.ifd_num,
                //     field.display_value().with_unit(&exif),
                // );
            }
        }

        //::::::::XMP METADATA HANDLING::::::::
        if xmp.len() > 0 {
            if let Ok(root) = xmp.parse::<minidom::Element>() {
                if let Some(rdf_root) =
                    root.get_child("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                {
                    for rdf_bag_element in rdf_root.children() {
                        //Dublin Core
                        if let Some(title_element) =
                            rdf_bag_element.get_child("title", "http://purl.org/dc/elements/1.1/")
                        {
                            title = title_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            // log(&format!("Set title from XMP-dc-title to {}", title));
                        }

                        if let Some(rights_element) =
                            rdf_bag_element.get_child("rights", "http://purl.org/dc/elements/1.1/")
                        {
                            copyright = rights_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            // log(&format!(
                            //     "Set copyright from XMP-dc-copyright to {}",
                            //     copyright
                            // ));
                        }

                        if let Some(creator_element) =
                            rdf_bag_element.get_child("creator", "http://purl.org/dc/elements/1.1/")
                        {
                            author = creator_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            // log(&format!("Set author from XMP-dc-creator to {}", author));
                        }

                        //XMPGImg thumbnail
                        if let Some(thumbnails_element) =
                            rdf_bag_element.get_child("Thumbnails", "http://ns.adobe.com/xap/1.0/")
                        {
                            // log("Found thumbnails element");
                            for thumb_element in
                                thumbnails_element.children().next().unwrap().children()
                            {
                                thumbnails.push(HashMap::from([
                                    (
                                        "format",
                                        thumb_element
                                            .get_child(
                                                "format",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "width",
                                        thumb_element
                                            .get_child(
                                                "width",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "height",
                                        thumb_element
                                            .get_child(
                                                "height",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "image",
                                        thumb_element
                                            .get_child(
                                                "image",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                ]));
                                // log("Pushed a thumbnail");
                            }
                        }

                        //XMPMM
                        if let Some(derived_from_element) = rdf_bag_element
                            .get_child("DerivedFrom", "http://ns.adobe.com/xap/1.0/mm/")
                        {
                            if let Some(original_document_id_element) = derived_from_element
                                .get_child(
                                    "originalDocumentID",
                                    "http://ns.adobe.com/xap/1.0/sType/ResourceRef#",
                                )
                            {
                                original_document_id = original_document_id_element.text();
                            }
                        }
                    }
                }
            }
        }

        let mut resolution_vector: Vec<HashMap<String, String>> = Vec::new();
        resolution_vector.push(resolution);
        let resolution_to_json = serde_json::to_string(&resolution_vector).unwrap();

        let mut gps_vector: Vec<HashMap<String, String>> = Vec::new();
        if gps.len() > 0 {
            gps_vector.push(gps);
        }
        let gps_to_json = serde_json::to_string(&gps_vector).unwrap();
        let subject_area_to_json = serde_json::to_string(&subject_area).unwrap();
        let thumbnails_to_json = serde_json::to_string(&thumbnails).unwrap();

        let result = Metadata {
            title,
            author,
            width,
            height,
            resolution: resolution_to_json,
            make,
            model,
            flash_found,
            copyright,
            description,
            gps: gps_to_json,
            xmp,
            iptc,
            subject_area: subject_area_to_json,
            thumbnails: thumbnails_to_json,
            original_document_id,
        };

        return result;
    }
}
