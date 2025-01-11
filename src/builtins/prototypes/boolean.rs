use super::{object::create_object_prototype, Prototype};
use crate::{init_prototype, r#type::EntityFactory};

pub fn create_boolean_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  init_prototype!("Boolean", create_object_prototype(factory), {
    "valueOf" => factory.pure_fn_returns_boolean,
  })
}
