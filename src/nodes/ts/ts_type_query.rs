use oxc::ast::ast::{TSTypeQuery, TSTypeQueryExprName};

use crate::{
  analyzer::Analyzer,
  ty::{generic::GenericInstanceType, Ty},
};

impl<'a> Analyzer<'a> {
  pub fn resolve_type_query(&mut self, node: &'a TSTypeQuery<'a>) -> Ty<'a> {
    let base = match &node.expr_name {
      TSTypeQueryExprName::IdentifierReference(node) => {
        self.exec_identifier_reference_read(node, None)
      }
      TSTypeQueryExprName::TSImportType(_node) => todo!(),
      TSTypeQueryExprName::QualifiedName(_node) => todo!(),
    };

    if let Some(type_parameters) = &node.type_parameters {
      let type_parameters = self.resolve_type_parameter_instantiation(type_parameters);
      Ty::Instance(self.allocator.alloc(GenericInstanceType::new(base, type_parameters)))
    } else {
      base
    }
  }
}
