use minijinja::{context, Environment, Template};
use roslibrust_codegen::parse::{FieldInfo, MessageFile};
use std::path::{Path, PathBuf};

mod helpers;

const MESSAGE_HEADER_TMPL: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/msg.h.j2"));

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

pub fn generate_message(
    msg_path: &Path,
    opts: &MessageGenOpts,
) -> Result<String, minijinja::Error> {
    let search_paths = opts.includes.iter().map(|inc| inc.path.clone()).collect();
    let msgs = roslibrust_codegen::find_and_parse_ros_messages(search_paths);

    let mut env = Environment::new();
    env.add_filter("append", helpers::append);
    env.add_function("is_header", helpers::is_header);
    env.add_template("msg.h", MESSAGE_HEADER_TMPL).unwrap();

    let message_name = msg_path.file_stem().unwrap().to_str().unwrap().to_owned();
    match msgs {
        Ok((msgs, _)) => {
            match msgs.iter().find(|parsed_msg| {
                parsed_msg.name == message_name && parsed_msg.package == opts.package
            }) {
                Some(parsed_msg) => {
                    fill_message_template(&env.get_template("msg.h").unwrap(), parsed_msg)
                }
                None => Err(minijinja::Error::new(
                    minijinja::ErrorKind::UndefinedError,
                    "Message not found in search paths",
                )),
            }
        }
        _ => Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "No messages found",
        )),
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
    pub md5sum_first: String,
    pub md5sum_second: String,
    pub description: String,
}

impl MessageSpecification {
    pub fn from_data(data: &MessageFile) -> Self {
        let mut spec = Self {
            short_name: data.name.clone(),
            package: data.package.clone(),
            fields: data
                .fields
                .iter()
                .map(|field| Field::from_roslibrust_field(field))
                .collect(),
            md5sum_first: String::from("ac26ce75a41384fa"),
            md5sum_second: String::from("8bb4dc10f491ab90"),
            description: String::from("Not a real description"),
        };
        spec.fields.iter_mut().for_each(|field| {
            if field.package.is_none() {
                field.package = Some(data.package.clone());
            }
        });
        spec
    }
}

fn fill_message_template(
    template: &Template,
    msg_data: &MessageFile,
) -> Result<String, minijinja::Error> {
    let context = context! {
        spec => MessageSpecification::from_data(msg_data),
    };
    template.render(&context)
}
