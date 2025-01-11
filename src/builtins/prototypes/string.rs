use super::{object::create_object_prototype, Prototype};
use crate::{init_prototype, r#type::EntityFactory};

pub fn create_string_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  init_prototype!("String", create_object_prototype(factory), {
    "anchor" => factory.pure_fn_returns_string,
    "at" => factory.pure_fn_returns_unknown,
    "big" => factory.pure_fn_returns_string,
    "blink" => factory.pure_fn_returns_string,
    "bold" => factory.pure_fn_returns_string,
    "charAt" => factory.pure_fn_returns_string,
    "charCodeAt" => factory.pure_fn_returns_unknown,
    "codePointAt" => factory.pure_fn_returns_unknown,
    "concat" => factory.pure_fn_returns_string,
    "endsWith" => factory.pure_fn_returns_unknown,
    "fixed" => factory.pure_fn_returns_string,
    "fontcolor" => factory.pure_fn_returns_string,
    "fontsize" => factory.pure_fn_returns_string,
    "includes" => factory.pure_fn_returns_unknown,
    "indexOf" => factory.pure_fn_returns_unknown,
    "italics" => factory.pure_fn_returns_string,
    "lastIndexOf" => factory.pure_fn_returns_unknown,
    "link" => factory.pure_fn_returns_string,
    "localeCompare" => factory.pure_fn_returns_unknown,
    "match" => factory.pure_fn_returns_unknown,
    "matchAll" => factory.pure_fn_returns_unknown,
    "normalize" => factory.pure_fn_returns_string,
    "padEnd" => factory.pure_fn_returns_string,
    "padStart" => factory.pure_fn_returns_string,
    "repeat" => factory.pure_fn_returns_string,
    "replace" => factory.pure_fn_returns_unknown,
    "replaceAll" => factory.pure_fn_returns_unknown,
    "search" => factory.pure_fn_returns_unknown,
    "slice" => factory.pure_fn_returns_string,
    "small" => factory.pure_fn_returns_string,
    "split" => factory.pure_fn_returns_unknown,
    "startsWith" => factory.pure_fn_returns_unknown,
    "strike" => factory.pure_fn_returns_string,
    "sub" => factory.pure_fn_returns_string,
    "substr" => factory.pure_fn_returns_string,
    "substring" => factory.pure_fn_returns_string,
    "sup" => factory.pure_fn_returns_string,
    "toLocaleLowerCase" => factory.pure_fn_returns_string,
    "toLocaleUpperCase" => factory.pure_fn_returns_string,
    "toLowerCase" => factory.pure_fn_returns_string,
    "toString" => factory.pure_fn_returns_string,
    "toUpperCase" => factory.pure_fn_returns_string,
    "trim" => factory.pure_fn_returns_string,
    "trimEnd" => factory.pure_fn_returns_string,
    "trimLeft" => factory.pure_fn_returns_string,
    "trimRight" => factory.pure_fn_returns_string,
    "trimStart" => factory.pure_fn_returns_string,
    "valueOf" => factory.pure_fn_returns_string,
  })
}
