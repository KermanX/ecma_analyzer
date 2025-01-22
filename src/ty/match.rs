use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;

use super::{
  callable::CallableType, intersection::IntersectionTypeBuilder, unresolved::UnresolvedType, Ty,
};
use crate::Analyzer;

pub enum MatchResult<'a> {
  Error,
  Unmatched,
  Matched,
  Inferred(FxHashMap<SymbolId, Ty<'a>>),
  Multiple(Vec<MatchResult<'a>>),
}

impl<'a> Analyzer<'a> {
  /// `Target extends Pattern`
  /// Returns
  /// - `None` if the types do not match
  /// - `Some` with a map of `infer T` in pattern
  pub fn match_types(&mut self, target: Ty<'a>, pattern: Ty<'a>) -> MatchResult<'a> {
    match (target, pattern) {
      (target, pattern) if target == pattern => MatchResult::Matched,

      (matched, Ty::Unresolved(UnresolvedType::InferType(s)))
      | (Ty::Unresolved(UnresolvedType::InferType(s)), matched) => MatchResult::Inferred({
        let mut map = FxHashMap::default();
        map.insert(s, matched);
        map
      }),

      (Ty::Any | Ty::Error, _) => {
        MatchResult::Multiple(vec![MatchResult::Unmatched, MatchResult::Matched])
      }
      (_, Ty::Any | Ty::Error | Ty::Unknown) => MatchResult::Matched,

      (Ty::Undefined, Ty::Void) => MatchResult::Matched,

      (Ty::StringLiteral(_), Ty::String) => MatchResult::Matched,
      (Ty::NumericLiteral(_), Ty::Number) => MatchResult::Matched,
      (Ty::BigIntLiteral(_), Ty::BigInt) => MatchResult::Matched,
      (Ty::BooleanLiteral(_), Ty::Boolean) => MatchResult::Matched,
      (Ty::UniqueSymbol(_), Ty::Symbol) => MatchResult::Matched,

      (Ty::Record(target), Ty::Record(pattern)) => todo!(),
      (Ty::Function(target), Ty::Function(pattern)) => self.match_callable_types(target, pattern),
      (Ty::Constructor(target), Ty::Constructor(pattern)) => {
        self.match_callable_types(target, pattern)
      }
      (Ty::Namespace(target), Ty::Namespace(pattern)) => todo!(),

      (Ty::Union(target), Ty::Union(pattern)) => {
        todo!()
      }
      (Ty::Union(target), pattern) => {
        let mut results = Vec::new();
        target.for_each(|ty| results.push(self.match_types(ty, pattern)));
        MatchResult::Multiple(results)
      }
      (Ty::Intersection(target), Ty::Intersection(pattern)) => {
        todo!()
      }
      (Ty::Intersection(target), pattern) => {
        let mut error = false;
        let mut matched: Option<Option<FxHashMap<SymbolId, IntersectionTypeBuilder<'a>>>> = None;
        target.for_each(|ty| {
          let result = self.match_types(ty, pattern);
          match result {
            MatchResult::Unmatched => {}
            MatchResult::Matched => {
              matched.get_or_insert_with(Default::default);
            }
            MatchResult::Inferred(map) => {
              let inferred =
                matched.get_or_insert_with(Default::default).get_or_insert_with(Default::default);
              for (s, t) in map {
                let builder = inferred.entry(s).or_insert_with(Default::default);
                builder.add(self, t);
              }
            }
            MatchResult::Multiple(results) => todo!(),
            MatchResult::Error => error = true,
          }
        });
        if error {
          MatchResult::Error
        } else if let Some(matched) = matched {
          if let Some(inferred) = matched {
            let mut map = FxHashMap::default();
            for (s, builder) in inferred {
              map.insert(s, builder.build(self));
            }
            MatchResult::Inferred(map)
          } else {
            MatchResult::Matched
          }
        } else {
          MatchResult::Unmatched
        }
      }

      (Ty::Generic(_) | Ty::Intrinsic(_), _) => MatchResult::Error,
      (_, Ty::Generic(_) | Ty::Intrinsic(_)) => MatchResult::Error,

      _ => MatchResult::Unmatched,
    }
  }

  fn match_callable_types<const CTOR: bool>(
    &mut self,
    target: &'a CallableType<'a, CTOR>,
    pattern: &'a CallableType<'a, CTOR>,
  ) -> MatchResult<'a> {
    // Step1: Match type parameters
    if target.type_params.len() != pattern.type_params.len() {
      return MatchResult::Unmatched;
    }
    for (target, pattern) in target.type_params.iter().zip(pattern.type_params.iter()) {
      if target.constraint == pattern.constraint {
        continue;
      }
      // Note: Contravariance - `pattern.constraint extends target.constraint`
      match self.match_types(
        pattern.constraint.unwrap_or(Ty::Unknown),
        target.constraint.unwrap_or(Ty::Unknown),
      ) {
        MatchResult::Error => return MatchResult::Error,
        MatchResult::Unmatched => return MatchResult::Unmatched,
        MatchResult::Matched => {}
        MatchResult::Inferred(_) => {
          // Somehow in TypeScript this is ignored
        }
        MatchResult::Multiple(_) => unreachable!("Unmatch or single match"),
      }
    }

    let mut inferred = FxHashMap::default();

    // Step2: Match this type
    if let (Some(target), Some(pattern)) = (target.this_param, pattern.this_param) {
      match self.match_types(pattern, target) {
        MatchResult::Error => return MatchResult::Error,
        MatchResult::Unmatched => return MatchResult::Unmatched,
        MatchResult::Matched => {}
        MatchResult::Inferred(map) => {
          inferred.extend(map);
        }
        MatchResult::Multiple(_) => unreachable!("Unmatch or single match"),
      }
    }

    // Step3: Match parameters
    for (index, (target_optional, mut target)) in target.params.iter().enumerate() {
      if let Some((pattern_optional, mut pattern)) = pattern.params.get(index) {
        if target_optional != pattern_optional {
          target = self.get_optional_type(*target_optional, target);
          pattern = self.get_optional_type(*pattern_optional, pattern);
        }
        match self.match_types(pattern, target) {
          MatchResult::Error => return MatchResult::Error,
          MatchResult::Unmatched => return MatchResult::Unmatched,
          MatchResult::Matched => {}
          MatchResult::Inferred(map) => {
            inferred.extend(map);
          }
          MatchResult::Multiple(_) => unreachable!("Unmatch or single match"),
        }
      } else if *target_optional {
        // Optional parameter, matched
      } else {
        // TODO: Check rest parameter
        return MatchResult::Unmatched;
      }
    }

    // Step4: Match rest parameter
    // TODO: Check rest parameter

    // Step5: Match return type
    match self.match_types(target.return_type, pattern.return_type) {
      MatchResult::Error => MatchResult::Error,
      MatchResult::Unmatched => MatchResult::Unmatched,
      MatchResult::Matched => MatchResult::Matched,
      MatchResult::Inferred(map) => {
        inferred.extend(map);
        MatchResult::Inferred(inferred)
      }
      MatchResult::Multiple(_) => unreachable!("Unmatch or single match"),
    }
  }
}
