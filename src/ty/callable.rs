use super::{generic::GenericParam, union::UnionType, Ty};
use crate::analyzer::Analyzer;
use oxc::{
  allocator,
  ast::{
    ast::{Argument, FormalParameterKind, TSType, TSTypeParameterInstantiation},
    NONE,
  },
  semantic::SymbolId,
  span::SPAN,
};

#[derive(Debug, Clone)]
pub enum ReturnType<'a> {
  /// function f(): number;
  Simple(Ty<'a>),

  /// function f(a): asserts a is number;
  Assertion(SymbolId, Ty<'a>),

  /// function f(a): a is number;
  Predicate(SymbolId, Ty<'a>),
}

#[derive(Debug, Clone)]
pub struct CallableType<'a, const CTOR: bool> {
  pub type_params: Vec<GenericParam<'a>>,
  pub this_type: Option<Ty<'a>>,
  /// (optional, type)
  pub params: Vec<(bool, Ty<'a>)>,
  pub rest_param: Option<Ty<'a>>,
  pub return_type: Ty<'a>,
}

pub type FunctionType<'a> = CallableType<'a, false>;
pub type ConstructorType<'a> = CallableType<'a, true>;

impl<'a> Analyzer<'a> {
  pub fn print_callable_type<const CTOR: bool>(
    &self,
    callable: &CallableType<'a, CTOR>,
  ) -> TSType<'a> {
    self.ast_builder.ts_type_function_type(
      SPAN,
      /* TODO: */ NONE,
      callable.this_type.map(|ty| {
        self.ast_builder.ts_this_parameter(
          SPAN,
          SPAN,
          Some(self.ast_builder.ts_type_annotation(SPAN, self.print_type(ty))),
        )
      }),
      self.ast_builder.formal_parameters(
        SPAN,
        FormalParameterKind::Signature,
        {
          let mut items = self.ast_builder.vec();
          for (i, (optional, param)) in callable.params.iter().enumerate() {
            items.push(self.ast_builder.formal_parameter(
              SPAN,
              self.ast_builder.vec(),
              self.ast_builder.binding_pattern(
                self.ast_builder.binding_pattern_kind_binding_identifier(
                  SPAN,
                  &*self.allocator.alloc(format!("a{i}")),
                ),
                Some(self.ast_builder.ts_type_annotation(SPAN, self.print_type(*param))),
                *optional,
              ),
              None,
              false,
              false,
            ))
          }
          if let Some(rest) = callable.rest_param {}
          items
        },
        callable.rest_param.map(|ty| {
          self.ast_builder.binding_rest_element(
            SPAN,
            self.ast_builder.binding_pattern(
              self.ast_builder.binding_pattern_kind_binding_identifier(SPAN, "rest"),
              Some(self.ast_builder.ts_type_annotation(SPAN, self.print_type(ty))),
              false,
            ),
          )
        }),
      ),
      self.ast_builder.ts_type_annotation(SPAN, self.print_type(callable.return_type)),
    )
  }
}

#[derive(Debug)]
pub enum ExtractedCallable<'a, const CTOR: bool> {
  Single(&'a CallableType<'a, CTOR>),
  Overloaded(Vec<ExtractedCallable<'a, CTOR>>),
  Union(Vec<ExtractedCallable<'a, CTOR>>),
}

macro_rules! impl_extract_callable {
  ($name: ident, $ctor: expr, $member: ident) => {
    impl<'a> Analyzer<'a> {
      pub fn $name(&mut self, ty: Ty<'a>) -> Option<ExtractedCallable<'a, $ctor>> {
        match ty {
          Ty::$member(f) => Some(ExtractedCallable::Single(f)),
          Ty::Union(u) => {
            let mut res = Some(vec![]);
            u.for_each(|ty| {
              if let (Some(res), Some(extracted)) = (&mut res, self.$name(ty)) {
                res.push(extracted);
              } else {
                res = None;
              }
            });
            res.map(ExtractedCallable::Union)
          }
          Ty::Intersection(i) => {
            let mut res = vec![];
            for ty in &i.types {
              if let Some(extracted) = self.$name(*ty) {
                res.push(extracted);
              }
            }
            match res.len() {
              0 => None,
              1 => Some(res.into_iter().next().unwrap()),
              _ => Some(ExtractedCallable::Overloaded(res)),
            }
          }
          _ => None,
        }
      }
    }
  };
}

impl_extract_callable!(extract_callable_function, false, Function);
impl_extract_callable!(extract_callable_constructor, true, Constructor);

impl<'a> Analyzer<'a> {
  pub fn get_callable_parameter_types<const CTOR: bool>(
    &mut self,
    callable: &ExtractedCallable<'a, CTOR>,
  ) -> Vec<Ty<'a>> {
    match callable {
      ExtractedCallable::Single(callable) => callable
        .params
        .iter()
        .copied()
        .map(|(optional, ty)| self.get_optional_type(optional, ty))
        .collect(),
      ExtractedCallable::Overloaded(callables) => {
        let allocator = self.allocator;
        let callables = callables.iter().map(|c| self.get_callable_parameter_types(c));
        let mut res = Vec::new();
        for callable in callables {
          for _ in res.len()..callable.len() {
            res.push(allocator.alloc(UnionType::default()));
          }
          for (i, item) in callable.into_iter().enumerate() {
            res[i].add(item);
          }
        }
        res.into_iter().map(|u| Ty::Union(u)).collect()
      }
      ExtractedCallable::Union(callables) => {
        todo!()
      }
    }
  }

  pub fn exec_call<const CTOR: bool>(
    &mut self,
    callable: Option<ExtractedCallable<'a, CTOR>>,
    type_parameters: &'a Option<allocator::Box<'a, TSTypeParameterInstantiation<'a>>>,
    arguments: &'a allocator::Vec<'a, Argument<'a>>,
  ) -> Ty<'a> {
    if let Some(callable) = callable {
      match callable {
        ExtractedCallable::Single(callable) => {
          if let Some(type_parameters) = type_parameters {
            let type_args = self.resolve_type_parameter_instantiation_or_defer(type_parameters);
            let old_generics = self.take_generics();
            self.instantiate_generic_param(&callable.type_params, type_args);
            let params = self.get_callable_parameter_types(&ExtractedCallable::Single(callable));
            let mut in_order = true;
            for (i, arg) in arguments.iter().enumerate() {
              match arg {
                Argument::SpreadElement(node) => {
                  self.exec_expression(&node.argument, None);
                  in_order = false;
                }
                node => {
                  self.exec_expression(
                    node.to_expression(),
                    in_order.then(|| params.get(i).copied().unwrap_or(Ty::Error)),
                  );
                }
              }
            }
            let ret = self.precise_type(callable.return_type);
            self.restore_generics(old_generics);
            ret
          } else {
            todo!()
          }
        }
        ExtractedCallable::Overloaded(callables) => {
          let ret = Ty::Error;
          for callable in callables {
            todo!();
            // If matches, set ret_val and break
          }
          ret
        }
        ExtractedCallable::Union(callables) => {
          todo!()
        }
      }
    } else {
      for arg in arguments {
        match arg {
          Argument::SpreadElement(node) => {
            self.exec_expression(&node.argument, None);
          }
          node => {
            self.exec_expression(node.to_expression(), None);
          }
        }
      }
      Ty::Error
    }
  }
}
