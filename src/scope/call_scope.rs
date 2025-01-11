use super::try_scope::TryScope;
use crate::{analyzer::Analyzer, r#type::Type, utils::CalleeInfo};
use oxc::semantic::ScopeId;
use std::mem;

pub struct CallScope<'a> {
  pub call_id: usize,
  pub callee: CalleeInfo<'a>,
  pub old_variable_scope_stack: Vec<ScopeId>,
  pub cf_scope_depth: usize,
  pub body_variable_scope: ScopeId,
  pub returned_values: Vec<Type<'a>>,
  pub is_async: bool,
  pub is_generator: bool,
  pub try_scopes: Vec<TryScope<'a>>,
  pub need_consume_arguments: bool,

  #[cfg(feature = "flame")]
  pub scope_guard: flame::SpanGuard,
}

impl<'a> CallScope<'a> {
  pub fn new(
    call_id: usize,
    callee: CalleeInfo<'a>,
    old_variable_scope_stack: Vec<ScopeId>,
    cf_scope_depth: usize,
    body_variable_scope: ScopeId,
    is_async: bool,
    is_generator: bool,
  ) -> Self {
    CallScope {
      call_id,
      callee,
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      returned_values: Vec::new(),
      is_async,
      is_generator,
      try_scopes: vec![TryScope::new(cf_scope_depth)],
      need_consume_arguments: false,

      #[cfg(feature = "flame")]
      scope_guard: flame::start_guard(callee.debug_name.to_string()),
    }
  }

  pub fn finalize(self, analyzer: &mut Analyzer<'a>) -> (Vec<ScopeId>, Type<'a>) {
    assert_eq!(self.try_scopes.len(), 1);

    // Forwards the thrown value to the parent try scope
    let try_scope = self.try_scopes.into_iter().next().unwrap();
    if try_scope.may_throw {
      if self.is_generator {
        let unknown = analyzer.factory.unknown;
        let parent_try_scope = analyzer.try_scope_mut();
        parent_try_scope.may_throw = true;
        if !try_scope.thrown_values.is_empty() {
          parent_try_scope.thrown_values.push(unknown);
        }
        for value in try_scope.thrown_values {
          value.unknown_mutation(analyzer);
        }
      } else if self.is_async {
        for value in try_scope.thrown_values {
          value.unknown_mutation(analyzer);
        }
      } else {
        analyzer.forward_throw(try_scope.thrown_values);
      }
    }

    let value = if self.returned_values.is_empty() {
      analyzer.factory.undefined
    } else {
      analyzer.factory.union(self.returned_values)
    };

    let value = if self.is_async { analyzer.factory.unknown } else { value };

    #[cfg(feature = "flame")]
    self.scope_guard.end();

    (self.old_variable_scope_stack, value)
  }
}

impl<'a> Analyzer<'a> {
  pub fn return_value(&mut self, value: Type<'a>) {
    let call_scope = self.call_scope_mut();
    call_scope.returned_values.push(value);

    let target_depth = call_scope.cf_scope_depth;
    self.exit_to(target_depth);
  }

  pub fn consume_arguments(&mut self) -> bool {
    let scope = self.call_scope().body_variable_scope;
    self.consume_arguments_on_scope(scope)
  }

  pub fn consume_return_values(&mut self) {
    let call_scope = self.call_scope_mut();
    let values = mem::take(&mut call_scope.returned_values);
    for value in values {
      value.unknown_mutation(self);
    }
  }
}
