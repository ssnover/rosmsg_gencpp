use clap::Parser;
use rosmsg_gencpp::IncludedNamespace;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the input .msg file
    #[arg(long = "msg", short = 'm')]
    msg_path: PathBuf,
    /// The package namespace for the generated message
    #[arg(long, short)]
    package: String,
    /// Output directory for generated code
    #[arg(long, short)]
    output: PathBuf,
    /// Include namespaces for message dependencies
    #[arg(long, short = 'I', value_parser = include_namespace_parse)]
    include: Option<Vec<IncludedNamespace>>,
}

fn include_namespace_parse(s: &str) -> Result<IncludedNamespace, String> {
    let components = s.split(':').collect::<Vec<&str>>();
    if components.len() == 2 {
        let package = components[0].to_owned();
        let path = PathBuf::from(components[1]);
        Ok(IncludedNamespace { package, path })
    } else {
        Err(String::from("Expected format: 'PACKAGE:/some/path'"))
    }
}

fn main() {
    let args = Args::parse();
    let opts = rosmsg_gencpp::MessageGenOpts {
        package: args.package,
        includes: args.include.unwrap_or(vec![]),
    };
    let generated_source = rosmsg_gencpp::generate_message(&args.msg_path, &opts).unwrap();
    println!("{generated_source}");
}
