use oxc::ast::ast::TSType;

use crate::analyzer::Analyzer;

#[derive(Debug, Clone, Copy)]
pub struct IntrinsicType {
  name: &'static str,
  handler: fn(&str) -> String,
}

impl<'a> Analyzer<'a> {
  pub fn print_intrinsic_type(&self, intrinsic: &IntrinsicType) -> TSType<'a> {
    todo!()
  }
}
