use crate::{
  analyzer::Analyzer,
  ty::{property_key::PropertyKeyType, Ty},
};
use oxc::ast::ast::JSXMemberExpression;

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_member_expression(
    &mut self,
    node: &'a JSXMemberExpression<'a>,
    _sat: Option<Ty<'a>>,
  ) -> Ty<'a> {
    let object = self.exec_jsx_member_expression_object(&node.object, None);
    let key = PropertyKeyType::StringLiteral(&node.property.name);
    self.get_property(object, key)
  }
}
