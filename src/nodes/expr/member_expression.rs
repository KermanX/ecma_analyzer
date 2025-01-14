use crate::{analyzer::Analyzer, r#type::Type};
use oxc::ast::ast::MemberExpression;

impl<'a> Analyzer<'a> {
  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read(
    &mut self,
    node: &'a MemberExpression<'a>,
  ) -> (Type<'a>, (Type<'a>, Type<'a>)) {
    let ((indeterminate, value), cache) = self.exec_member_expression_read_in_chain(node);

    if indeterminate {
      self.pop_cf_scope();
    }

    (value, cache)
  }

  /// Returns ((indeterminate, value), cache)
  pub fn exec_member_expression_read_in_chain(
    &mut self,
    node: &'a MemberExpression<'a>,
  ) -> ((bool, Type<'a>), (Type<'a>, Type<'a>)) {
    let (mut indeterminate, object) = self.exec_expression_in_chain(node.object());

    if !indeterminate && node.optional() {
      self.push_indeterminate_cf_scope();
      indeterminate = true;
    }

    let key = self.exec_key(node);

    let value = self.get_property(object, key);

    ((indeterminate, value), (object, key))
  }

  pub fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: Type<'a>,
    cache: Option<(Type<'a>, Type<'a>)>,
  ) {
    let (object, key) = cache.unwrap_or_else(|| {
      let object = self.exec_expression(node.object());

      let key = self.exec_key(node);

      (object, key)
    });

    self.set_property(object, key, value);
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> Type<'a> {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => self.exec_identifier_name(&node.property),
      MemberExpression::PrivateFieldExpression(node) => self.exec_private_identifier(&node.field),
    }
  }
}
