use handlebars::Handlebars;
use roslibrust_codegen::parse::{MessageFile, FieldInfo};
use std::path::{Path, PathBuf};

mod helpers;

const MESSAGE_HEADER_TMPL: &'static str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/msg.h.hbs"
));

#[derive(Clone, Debug)]
pub struct IncludedNamespace {
    /// The package namespace being included
    pub package: String,
    /// The path to the package
    pub path: PathBuf,
}

pub struct MessageGenOpts {
    /// The package namespace for the generated message
    pub package: String,
    /// Include packages for resolving message dependencies
    pub includes: Vec<IncludedNamespace>,
}

pub fn generate_message(msg_path: &Path, opts: &MessageGenOpts) -> Result<String, ()> {
    let search_paths = opts.includes.iter().map(|inc| inc.path.clone()).collect();
    let msgs = roslibrust_codegen::find_and_parse_ros_messages(search_paths);

    let mut handlebars = Handlebars::new();
    handlebars.register_helper("upper", Box::new(helpers::upper));
    handlebars.register_helper("is_header", Box::new(helpers::is_header));
    handlebars.register_template_string("msg.h", MESSAGE_HEADER_TMPL).unwrap();

    let message_name = msg_path.file_stem().unwrap().to_str().unwrap().to_owned();
    match msgs {
        Ok((msgs, _)) => {
            match msgs.iter().find(|parsed_msg| {
                parsed_msg.name == message_name && parsed_msg.package == opts.package
            }) {
                Some(parsed_msg) => fill_message_template(&handlebars, parsed_msg),
                None => Err(()),
            }
        }
        _ => Err(()),
    }
}

#[derive(serde::Serialize)]
struct Field {
    pub name: String,
    pub field_type: String,
    pub package: Option<String>,
}

impl Field {
    pub fn from_roslibrust_field(field: &FieldInfo) -> Self {
        Self {
            name: field.field_name.clone(),
            field_type: field.field_type.field_type.clone(),
            package: field.field_type.package_name.clone(),
        }
    }
}

#[derive(serde::Serialize)]
struct MessageSpecification {
    pub short_name: String,
    pub package: String,
    pub fields: Vec<Field>,
}

impl MessageSpecification {
    pub fn from_data(data: &MessageFile) -> Self {
        let mut spec = Self {
            short_name: data.name.clone(),
            package: data.package.clone(),
            fields: data.fields.iter().map(|field| Field::from_roslibrust_field(field)).collect(),
        };
        spec.fields.iter_mut().for_each(|field| {
            if field.package.is_none() {
                field.package = Some(data.package.clone());
            }
        });
        spec
    }
}

fn fill_message_template(registry: &Handlebars, msg_data: &MessageFile) -> Result<String, ()> {
    let data = serde_json::json!({
        "spec": MessageSpecification::from_data(msg_data),
    });
    Ok(registry.render("msg.h", &data).unwrap())
}
