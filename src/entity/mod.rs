mod arguments;
mod array;
mod builtin_fn;
mod class;
mod consumed_object;
mod factory;
mod function;
mod literal;
mod logical_result;
mod object;
mod operations;
mod primitive;
mod react_element;
mod symbol;
mod typeof_result;
mod union;
mod unknown;
mod utils;

pub use builtin_fn::PureBuiltinFnEntity;
pub use class::ClassEntity;
pub use factory::EntityFactory;
pub use literal::LiteralEntity;
pub use object::{ObjectEntity, ObjectProperty, ObjectPropertyValue};
pub use operations::EntityOpHost;
pub use primitive::PrimitiveEntity;
pub use typeof_result::TypeofResult;
pub use unknown::UnknownEntity;

use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::{cmp::Ordering, fmt::Debug};

/// (vec![(definite, key, value)])
pub type EnumeratedProperties<'a> = Vec<(bool, Entity<'a>, Entity<'a>)>;

/// (vec![known_elements], rest)
pub type IteratedElements<'a> = (Vec<Entity<'a>>, Option<Entity<'a>>);

pub trait EntityTrait<'a>: Debug {
  fn unknown_mutation(&'a self, analyzer: &mut Analyzer<'a>);

  fn get_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>) -> Entity<'a>;
  fn set_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>, value: Entity<'a>);
  fn enumerate_properties(&'a self, analyzer: &mut Analyzer<'a>) -> EnumeratedProperties<'a>;
  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>);
  fn call(&'a self, analyzer: &mut Analyzer<'a>, this: Entity<'a>, args: Entity<'a>) -> Entity<'a>;
  fn construct(&'a self, analyzer: &mut Analyzer<'a>, args: Entity<'a>) -> Entity<'a>;
  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a>;
  fn r#await(&'a self, analyzer: &mut Analyzer<'a>) -> Entity<'a>;
  fn iterate(&'a self, analyzer: &mut Analyzer<'a>) -> IteratedElements<'a>;

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_jsx_child(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_literals(&'a self, _analyzer: &Analyzer<'a>) -> Option<FxHashSet<LiteralEntity<'a>>> {
    None
  }
  fn get_literal(&'a self, analyzer: &Analyzer<'a>) -> Option<LiteralEntity<'a>> {
    self.get_to_literals(analyzer).and_then(|set| {
      if set.len() == 1 {
        set.into_iter().next()
      } else {
        None
      }
    })
  }

  fn test_typeof(&self) -> TypeofResult;
  fn test_truthy(&self) -> Option<bool>;
  fn test_nullish(&self) -> Option<bool>;
  fn test_is_undefined(&self) -> Option<bool> {
    let t = self.test_typeof();
    match (t == TypeofResult::Undefined, t.contains(TypeofResult::Undefined)) {
      (true, _) => Some(true),
      (false, true) => None,
      (false, false) => Some(false),
    }
  }

  fn destruct_as_array(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    length: usize,
    need_rest: bool,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let (mut elements, rest) = self.iterate(analyzer);
    let iterated_len = elements.len();
    let extras = match iterated_len.cmp(&length) {
      Ordering::Equal => Vec::new(),
      Ordering::Greater => elements.split_off(length),
      Ordering::Less => {
        elements.resize(length, rest.unwrap_or(analyzer.factory.undefined));
        Vec::new()
      }
    };

    let rest_arr = need_rest.then(|| {
      let rest_arr = analyzer.new_empty_array();
      rest_arr.elements.borrow_mut().extend(extras);
      if let Some(rest) = rest {
        rest_arr.init_rest(rest);
      }
      rest_arr as Entity<'a>
    });

    (elements, rest_arr)
  }

  fn iterate_result_union(&'a self, analyzer: &mut Analyzer<'a>) -> Option<Entity<'a>> {
    let (elements, rest) = self.iterate(analyzer);
    if let Some(rest) = rest {
      let mut result = elements;
      result.push(rest);
      Some(analyzer.factory.union(result))
    } else if !elements.is_empty() {
      Some(analyzer.factory.union(elements))
    } else {
      None
    }
  }

  fn call_as_getter(&'a self, analyzer: &mut Analyzer<'a>, this: Entity<'a>) -> Entity<'a> {
    self.call(analyzer, this, analyzer.factory.empty_arguments)
  }

  fn call_as_setter(
    &'a self,
    analyzer: &mut Analyzer<'a>,

    this: Entity<'a>,
    value: Entity<'a>,
  ) -> Entity<'a> {
    self.call(analyzer, this, analyzer.factory.arguments(vec![(false, value)]))
  }
}

pub type Entity<'a> = &'a (dyn EntityTrait<'a> + 'a);
