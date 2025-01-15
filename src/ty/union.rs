use super::Ty;
use crate::utils::F64WithEq;
use oxc::{allocator::Allocator, semantic::SymbolId, span::Atom};
use rustc_hash::FxHashSet;
use std::{hash::Hash, mem};

#[derive(Debug, Default)]
pub enum UnionType<'a> {
  #[default]
  Never,
  Error,
  Any,
  Unknown,
  Compound(Box<CompoundUnion<'a>>),
  WithUnresolved(Box<UnionType<'a>>, Vec<Ty<'a>>),
}

impl<'a> UnionType<'a> {
  pub fn add(&mut self, ty: Ty<'a>) {
    match (self, ty) {
      (UnionType::Error | UnionType::Any | UnionType::Unknown, _) => {}
      (s, Ty::Error) => *s = UnionType::Error,
      (s, Ty::Any) => *s = UnionType::Any,
      (s, Ty::Unknown) => *s = UnionType::Unknown,
      (_, Ty::Never) => {}

      (UnionType::WithUnresolved(_, t), Ty::UnresolvedType(_) | Ty::UnresolvedVariable(_)) => {
        t.push(ty)
      }
      (UnionType::WithUnresolved(s, _), ty) => {
        s.add(ty);
      }
      (s, Ty::UnresolvedType(_) | Ty::UnresolvedVariable(_)) => {
        *s = UnionType::WithUnresolved(Box::new(mem::take(s)), vec![ty])
      }

      (s, Ty::Union(tys)) => {
        tys.for_each(|ty| s.add(ty));
      }

      // The rest should be added to compound
      (s @ UnionType::Never, compound) => {
        *s = UnionType::Compound(Box::new(CompoundUnion::default()));
        s.add(compound);
      }
      (UnionType::Compound(c), compound) => {
        c.add(compound);
      }
    }
  }

  pub fn for_each(&self, mut f: impl FnMut(Ty<'a>) -> ()) {
    match self {
      UnionType::Never => f(Ty::Never),
      UnionType::Error => f(Ty::Error),
      UnionType::Any => f(Ty::Any),
      UnionType::Unknown => f(Ty::Unknown),

      UnionType::Compound(c) => {
        c.for_each(f);
      }

      UnionType::WithUnresolved(s, t) => {
        s.for_each(&mut f);
        t.iter().copied().for_each(f);
      }
    }
  }
}

#[derive(Debug, Default)]
struct CompoundUnion<'a> {
  string: LiteralAble<&'a Atom<'a>>,
  number: LiteralAble<F64WithEq>,
  bigint: LiteralAble<&'a Atom<'a>>,
  symbol: LiteralAble<SymbolId>,

  object: bool,
  void: bool,
  null: bool,
  undefined: bool,
  /// (has_true, has_false)
  boolean: (bool, bool),

  complex: FxHashSet<Ty<'a>>,
}

impl<'a> CompoundUnion<'a> {
  pub fn add(&mut self, ty: Ty<'a>) {
    match ty {
      Ty::Error
      | Ty::Any
      | Ty::Unknown
      | Ty::Never
      | Ty::Union(_)
      | Ty::UnresolvedType(_)
      | Ty::UnresolvedVariable(_) => {
        unreachable!("Handled in UnionType")
      }

      Ty::Void => self.void = true,
      Ty::Null => self.null = true,
      Ty::Undefined => self.undefined = true,
      Ty::Object => self.object = true,

      Ty::String => self.string = LiteralAble::Any,
      Ty::Number => self.number = LiteralAble::Any,
      Ty::BigInt => self.bigint = LiteralAble::Any,
      Ty::Symbol => self.symbol = LiteralAble::Any,
      Ty::Boolean => self.boolean = (true, true),

      Ty::StringLiteral(s) => self.string.add(s),
      Ty::NumericLiteral(n) => self.number.add(n),
      Ty::BigIntLiteral(b) => self.bigint.add(b),
      Ty::UniqueSymbol(s) => self.symbol.add(s),
      Ty::BooleanLiteral(true) => self.boolean.0 = true,
      Ty::BooleanLiteral(false) => self.boolean.1 = true,

      Ty::Record(_)
      | Ty::Function(_)
      | Ty::Constructor(_)
      | Ty::Namespace(_)
      | Ty::Intersection(_) => {
        self.complex.insert(ty);
      }

      Ty::Generic(_) | Ty::Intrinsic(_) => unreachable!("Non-value"),
    }
  }

  pub fn for_each(&self, mut f: impl FnMut(Ty<'a>) -> ()) {
    self.string.for_each(Ty::String, Ty::StringLiteral, &mut f);
    self.number.for_each(Ty::Number, Ty::NumericLiteral, &mut f);
    self.bigint.for_each(Ty::BigInt, Ty::BigIntLiteral, &mut f);
    self.symbol.for_each(Ty::Symbol, Ty::UniqueSymbol, &mut f);

    if self.object {
      f(Ty::Object);
    }
    if self.void {
      f(Ty::Void);
    }
    if self.null {
      f(Ty::Null);
    }
    if self.undefined {
      f(Ty::Undefined);
    }
    match (self.boolean.0, self.boolean.1) {
      (true, true) => f(Ty::Boolean),
      (true, false) => f(Ty::BooleanLiteral(true)),
      (false, true) => f(Ty::BooleanLiteral(false)),
      (false, false) => {}
    }

    self.complex.iter().copied().for_each(f);
  }
}

#[derive(Debug, Default)]
enum LiteralAble<L> {
  #[default]
  Vacant,
  Any,
  Literals(FxHashSet<L>),
}

impl<'a, L> LiteralAble<L> {
  pub fn add(&mut self, literal: L)
  where
    L: Hash + Eq,
  {
    match self {
      LiteralAble::Vacant => {
        *self = LiteralAble::Literals({
          let mut set = FxHashSet::default();
          set.insert(literal);
          set
        })
      }
      LiteralAble::Any => {}
      LiteralAble::Literals(set) => {
        set.insert(literal);
      }
    }
  }

  pub fn for_each(&self, any: Ty<'a>, ctor: fn(L) -> Ty<'a>, mut f: impl FnMut(Ty<'a>) -> ())
  where
    L: Copy,
  {
    match self {
      LiteralAble::Vacant => {}
      LiteralAble::Any => f(any),
      LiteralAble::Literals(set) => set.iter().copied().map(ctor).for_each(f),
    }
  }
}

pub fn into_union<'a, Iter>(
  allocator: &'a Allocator,
  types: impl IntoIterator<Item = Ty<'a>, IntoIter = Iter>,
) -> Ty<'a>
where
  Iter: Iterator<Item = Ty<'a>> + ExactSizeIterator,
{
  let mut iter = types.into_iter();
  match iter.len() {
    // FIXME: Should be Ty::Never
    0 => Ty::Undefined,
    1 => iter.next().unwrap(),
    _ => Ty::Union({
      let union = allocator.alloc(UnionType::default());
      iter.for_each(|ty| union.add(ty));
      union
    }),
  }
}
