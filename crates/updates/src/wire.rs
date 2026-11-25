use super::guest::Guest;
use garlic_state::StateError;
use wasmi::{Linker, Memory, Store, StoreLimits, StoreLimitsBuilder, TypedFunc};

pub const BLOCK: usize = 8192;
/// One bounded, stateful instance belongs exclusively to one TCP connection.
pub struct Wire {
    store: Store<StoreLimits>,
    memory: Memory,
    buffer: usize,
    step: TypedFunc<(i32, i32), i32>,
}
impl Wire {
    pub fn new(guest: &Guest) -> Result<Self, StateError> {
        let limits = StoreLimitsBuilder::new()
            .instances(1)
            .memories(1)
            .memory_size(131072)
            .tables(1)
            .table_elements(64)
            .build();
        let mut store = Store::new(&guest.engine, limits);
        store.limiter(|l| l);
        store.set_fuel(20000)?;
        let instance =
            Linker::new(&guest.engine).instantiate_and_start(&mut store, &guest.module)?;
        let memory = instance
            .get_memory(&store, "memory")
            .ok_or("missing wire memory")?;
        let buffer = usize::try_from(
            instance
                .get_typed_func::<(), i32>(&store, "net_buffer")?
                .call(&mut store, ())?,
        )?;
        if buffer
            .checked_add(BLOCK)
            .is_none_or(|end| end > memory.data_size(&store))
        {
            return Err("invalid wire buffer".into());
        }
        let step = instance.get_typed_func(&store, "net_step")?;
        Ok(Self {
            store,
            memory,
            buffer,
            step,
        })
    }

    pub fn event(&mut self, event: i32, input: &[u8]) -> Result<(bool, Vec<u8>), StateError> {
        if input.len() > BLOCK {
            return Err("wire input too large".into());
        }
        self.memory.write(&mut self.store, self.buffer, input)?;
        self.store.set_fuel(500000)?;
        let result = u32::try_from(
            self.step
                .call(&mut self.store, (event, input.len() as i32))?,
        )?;
        let len = (result & 0xffff) as usize;
        if result >> 16 > 1 || len > BLOCK {
            return Err("invalid wire response".into());
        }
        let mut out = vec![0; len];
        self.memory.read(&self.store, self.buffer, &mut out)?;
        Ok((result >> 16 == 1, out))
    }
}
