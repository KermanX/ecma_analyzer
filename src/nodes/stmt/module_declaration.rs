use oxc::ast::ast::{ExportDefaultDeclarationKind, ModuleDeclaration};

use crate::{analyzer::Analyzer, ty::Ty};

impl<'a> Analyzer<'a> {
  pub fn declare_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(node) => {
        if let Some(specifiers) = &node.specifiers {
          let name = node.source.value.as_str();
          let known = self.resolve_module(name);

          for specifier in specifiers {
            let value = if let Some(known) = known {
              // match specifier {
              //   ImportDeclarationSpecifier::ImportDefaultSpecifier(_node) => known.default,
              //   ImportDeclarationSpecifier::ImportNamespaceSpecifier(_node) => known.namespace,
              //   ImportDeclarationSpecifier::ImportSpecifier(node) => {
              //     let key = Ty::StringLiteral(match &node.imported {
              //       ModuleExportName::IdentifierName(identifier) => &identifier.name,
              //       ModuleExportName::IdentifierReference(identifier) => &identifier.name,
              //       ModuleExportName::StringLiteral(literal) => &literal.value,
              //     });
              //     self.get_property(known.namespace, key)
              //   }
              // }
              todo!()
            } else {
              Ty::Unknown
            };

            let local = specifier.local();
            self.declare_binding_identifier(local, true);
            self.init_binding_identifier(local, Some(value));
          }
        }
      }
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return;
        }
        if let Some(declaration) = &node.declaration {
          self.declare_declaration(declaration);
        }
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        match &node.declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default function(){}`
              return;
            }
            self.declare_function(node);
          }
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default class{}`
              return;
            }
            self.declare_class(node);
          }
          ExportDefaultDeclarationKind::TSInterfaceDeclaration(node) => {
            self.declare_ts_interface(node);
          }
          _expr => {}
        };
      }
      ModuleDeclaration::ExportAllDeclaration(_)
      | ModuleDeclaration::TSExportAssignment(_)
      | ModuleDeclaration::TSNamespaceExportDeclaration(_) => {
        // Nothing to do
      }
    }
  }

  pub fn init_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(_node) => {}
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return;
        }
        if let Some(declaration) = &node.declaration {
          self.init_declaration(declaration);
        }
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        match &node.declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => self.exec_function(node, None),
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default class{}`
              self.exec_class(node, None)
            } else {
              self.init_class(node)
            }
          }
          ExportDefaultDeclarationKind::TSInterfaceDeclaration(node) => {
            self.init_ts_interface(node)
          }
          node => self.exec_expression(node.to_expression(), None),
        };
      }
      ModuleDeclaration::ExportAllDeclaration(_node) => {
        // Nothing to do
      }
      ModuleDeclaration::TSExportAssignment(node) => {
        self.exec_expression(&node.expression, None);
      }
      ModuleDeclaration::TSNamespaceExportDeclaration(node) => {
        todo!()
      }
    }
  }
}
