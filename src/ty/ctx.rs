use oxc::{
  allocator,
  ast::ast::{TSType, TSTypeAnnotation},
};

use crate::{scope::r#type::TypeScopeId, Analyzer};

use super::Ty;

#[derive(Debug, Clone, Copy)]
pub enum CtxTy<'a> {
  Static(Ty<'a>),
  WithCtx(TypeScopeId, &'a TSType<'a>),
}

impl<'a> PartialEq for CtxTy<'a> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (CtxTy::Static(a), CtxTy::Static(b)) => a == b,
      (CtxTy::WithCtx(a, an), CtxTy::WithCtx(b, bn)) => {
        a == b && (an as *const _) == (bn as *const _)
      }
      _ => false,
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn ctx_ty_from_ts_type(&self, node: &'a TSType<'a>) -> CtxTy<'a> {
    CtxTy::WithCtx(self.type_scopes.top(), node)
  }

  pub fn ctx_ty_from_annotation(
    &mut self,
    node: &'a Option<allocator::Box<'a, TSTypeAnnotation<'a>>>,
    inferred: Option<Ty<'a>>,
  ) -> CtxTy<'a> {
    match (node, inferred) {
      (Some(node), _) => self.ctx_ty_from_ts_type(&node.type_annotation),
      (None, Some(ty)) => {
        // TODO: perf
        let node = self.allocator.alloc(self.serialize_type(ty));
        self.ctx_ty_from_ts_type(node)
      }
      (None, None) => CtxTy::Static(Ty::Any),
    }
  }

  pub fn refresh_ctx_ty(&mut self, ty: CtxTy<'a>) -> CtxTy<'a> {
    match ty {
      CtxTy::Static(ty) => CtxTy::Static(ty),
      CtxTy::WithCtx(_, node) => self.ctx_ty_from_ts_type(node),
    }
  }

  pub fn resolve_ctx_ty(&mut self, ty: CtxTy<'a>) -> Ty<'a> {
    match ty {
      CtxTy::Static(ty) => ty,
      CtxTy::WithCtx(scope, node) => {
        let old_top = self.type_scopes.replace_top(scope);
        let ty = self.resolve_type(node);
        self.type_scopes.replace_top(old_top);
        ty
      }
    }
  }

  pub fn serialize_ctx_ty(&mut self, ty: CtxTy<'a>) -> TSType<'a> {
    let ty = self.resolve_ctx_ty(ty);
    self.serialize_type(ty)
  }
}
