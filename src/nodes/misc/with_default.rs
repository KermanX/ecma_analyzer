use crate::{
  analyzer::Analyzer,
  ty::{union::into_union, Ty},
};
use oxc::ast::ast::Expression;

impl<'a> Analyzer<'a> {
  pub fn exec_with_default(&mut self, default: &'a Expression<'a>, value: Ty<'a>) -> Ty<'a> {
    self.push_indeterminate_cf_scope();
    let default_val = self.exec_expression(default);
    self.pop_cf_scope();

    into_union(self.allocator, [default_val, value])
  }
}
