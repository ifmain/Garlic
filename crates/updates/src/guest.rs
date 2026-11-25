use base64::{engine::general_purpose::STANDARD, Engine as _};
use garlic_state::StateError;
use wasmi::{Config, Engine, Linker, Module, Store, StoreLimitsBuilder};

/// One ABI for external modules. Guests see three integers and return one integer.
/// Each invocation gets a fresh instance. No WASI or other host imports are linked.
pub struct Guest {
    pub(crate) engine: Engine,
    pub(crate) module: Module,
}

impl Guest {
    pub fn load(encoded: &str) -> Result<Self, StateError> {
        if encoded.len() > 8192 {
            return Err("guest exceeds size limit".into());
        }
        let bytes = STANDARD.decode(encoded)?;
        let mut config = Config::default();
        config
            .consume_fuel(true)
            .allow_start_fn(false)
            .enforced_limits(wasmi::EnforcedLimits::strict());
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &bytes[..])?;
        if module.imports().next().is_some() {
            return Err("guest imports are disabled".into());
        }
        Ok(Self { engine, module })
    }

    pub fn call(&self, entry: &str, args: (i32, i32, i32)) -> Result<i32, StateError> {
        let limits = StoreLimitsBuilder::new()
            .instances(1)
            .memories(1)
            .memory_size(131072)
            .tables(1)
            .table_elements(64)
            .build();
        let mut store = Store::new(&self.engine, limits);
        store.limiter(|limits| limits);
        store.set_fuel(20_000)?;
        let instance = Linker::new(&self.engine).instantiate_and_start(&mut store, &self.module)?;
        Ok(instance
            .get_typed_func::<(i32, i32, i32), i32>(&store, entry)?
            .call(&mut store, args)?)
    }
}
