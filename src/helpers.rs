use handlebars::handlebars_helper;

handlebars_helper!(upper: |s: String| s.to_uppercase());

handlebars_helper!(is_header: |arg: (String, String)| arg.0 == "std_msgs" && arg.1 == "Header");
