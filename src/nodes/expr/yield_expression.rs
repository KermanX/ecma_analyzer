use oxc::ast::ast::YieldExpression;

use crate::{analyzer::Analyzer, ty::Ty};

impl<'a> Analyzer<'a> {
  pub fn exec_yield_expression(
    &mut self,
    node: &'a YieldExpression<'a>,
    sat: Option<Ty<'a>>,
  ) -> Ty<'a> {
    todo!()
  }
}
