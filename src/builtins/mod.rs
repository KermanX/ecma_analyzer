mod constants;
mod globals;
mod import_meta;
mod known_modules;
mod prototypes;
mod utils;

use crate::{
  config::Config,
  r#type::{EntityFactory, Type},
};
use known_modules::KnownModule;
use prototypes::BuiltinPrototypes;
pub use prototypes::Prototype;
use rustc_hash::FxHashMap;

pub struct Builtins<'a> {
  pub config: &'a Config,
  pub factory: &'a EntityFactory<'a>,

  pub prototypes: &'a BuiltinPrototypes<'a>,
  pub globals: FxHashMap<&'static str, Type<'a>>,
  pub import_meta: Type<'a>,
  pub known_modules: FxHashMap<&'static str, KnownModule<'a>>,
}

impl<'a> Builtins<'a> {
  pub fn new(config: &'a Config, factory: &'a EntityFactory<'a>) -> Self {
    let prototypes = Self::create_builtin_prototypes(factory);
    let mut builtins = Self {
      config,
      factory,

      prototypes,
      import_meta: Self::create_import_meta(factory, prototypes),
      globals: Default::default(),       // Initialize later
      known_modules: Default::default(), // Initialize later
    };
    builtins.init_globals();
    builtins.init_known_modules();
    builtins
  }
}
