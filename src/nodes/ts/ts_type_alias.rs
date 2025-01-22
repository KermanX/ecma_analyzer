use oxc::ast::ast::TSTypeAliasDeclaration;

use crate::{
  ty::{generic::GenericType, unresolved::UnresolvedType, Ty},
  Analyzer,
};

impl<'a> Analyzer<'a> {
  pub fn declare_ts_type_alias(&mut self, node: &'a TSTypeAliasDeclaration<'a>) {
    let symbol_id = node.id.symbol_id();
    let body = self.resolve_type(&node.type_annotation);
    let ty = if let Some(type_parameters) = &node.type_parameters {
      let params = self.resolve_type_parameter_declaration(type_parameters);
      Ty::Generic(self.allocator.alloc(GenericType { name: &node.id.name, params, body }))
    } else {
      body
    };
    self.types.insert(symbol_id, ty);
  }

  pub fn init_ts_type_alias(&mut self, node: &'a TSTypeAliasDeclaration<'a>) {
    let symbol_id = node.id.symbol_id();
    let unresolved = *self.types.get(&symbol_id).unwrap();
    if let Some(resolved) = self.try_resolve_unresolved(unresolved) {
      self.types.insert(symbol_id, resolved);
    }
  }
}
