use super::{
  consumed_object, Entity, EntityFactory, EntityTrait, EnumeratedProperties, IteratedElements,
  TypeofResult,
};
use crate::{analyzer::Analyzer, use_consumed_flag};
use std::cell::Cell;

#[derive(Debug)]
pub struct ReactElementEntity<'a> {
  consumed: Cell<bool>,
  tag: Entity<'a>,
  props: Entity<'a>,
}

impl<'a> EntityTrait<'a> for ReactElementEntity<'a> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    let tag = self.tag;
    let props = self.props;
    analyzer.exec_consumed_fn("React_blackbox", move |analyzer| {
      let copied_props = analyzer.new_empty_object(&analyzer.builtins.prototypes.object);
      copied_props.init_spread(analyzer, props);
      tag.jsx(analyzer, copied_props)
    });
  }

  fn get_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>) -> Entity<'a> {
    consumed_object::get_property(self, analyzer, key)
  }

  fn set_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>, value: Entity<'a>) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, key, value)
  }

  fn enumerate_properties(&'a self, analyzer: &mut Analyzer<'a>) -> EnumeratedProperties<'a> {
    consumed_object::enumerate_properties(self, analyzer)
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, key: Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, key)
  }

  fn call(&'a self, analyzer: &mut Analyzer<'a>, this: Entity<'a>, args: Entity<'a>) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::call(self, analyzer, this, args)
  }

  fn construct(&'a self, analyzer: &mut Analyzer<'a>, args: Entity<'a>) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::construct(self, analyzer, args)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::jsx(self, analyzer, props)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::r#await(analyzer)
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>) -> IteratedElements<'a> {
    self.consume(analyzer);
    consumed_object::iterate(analyzer)
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown_string
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown_string
  }

  fn get_to_numeric(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(val) => analyzer.factory.boolean(val),
      None => analyzer.factory.unknown_boolean,
    }
  }

  fn get_to_property_key(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn get_to_jsx_child(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::_Unknown
  }

  fn test_truthy(&self) -> Option<bool> {
    None
  }

  fn test_nullish(&self) -> Option<bool> {
    None
  }
}

impl<'a> EntityFactory<'a> {
  pub fn react_element(
    &self,
    tag: Entity<'a>,
    props: Entity<'a>,
  ) -> &'a mut ReactElementEntity<'a> {
    self.alloc(ReactElementEntity { consumed: Cell::new(false), tag, props })
  }
}
