use oxc::ast::ast::JSXFragment;

use crate::{analyzer::Analyzer, ty::Ty};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_fragment(&mut self, node: &'a JSXFragment<'a>, _sat: Option<Ty<'a>>) -> Ty<'a> {
    // already computed unknown
    self.exec_jsx_children(&node.children)
  }
}
