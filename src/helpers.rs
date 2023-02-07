use std::borrow::Cow;
use minijinja::State;
use minijinja::value::Value;

pub fn is_header(value: Value) -> bool {
    false
}

pub fn append(v: Cow<'_, str>, append: Cow<'_, str>) -> String {
    let mut v = v.to_string();
    v.push_str(&append as &str);
    v
}