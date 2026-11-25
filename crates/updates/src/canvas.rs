use garlic_state::StateError;
use wasmi::{Instance, Linker, Store, StoreLimits, StoreLimitsBuilder, TypedFunc};
use super::guest::Guest;

/// Stateful UI guest. The only host data is pointer position, pressed flag and elapsed time.
/// Output is a bounded list of rectangles in a 320 by 240 logical viewport.
pub struct Canvas {
    store: Store<StoreLimits>,
    instance: Instance,
    draw: TypedFunc<(i32,i32,i32,i32),i32>,
}
impl Canvas {
    pub fn new(encoded: &str) -> Result<Self,StateError> {
        let guest=Guest::load(encoded)?;
        let mut store=Store::new(&guest.engine,StoreLimitsBuilder::new().instances(1)
            .memories(1).memory_size(131072).tables(1).table_elements(64).build());
        store.limiter(|limits|limits);
        store.set_fuel(20000)?;
        let instance=Linker::new(&guest.engine).instantiate_and_start(&mut store,&guest.module)?;
        let draw=instance.get_typed_func(&store,"ui_frame")?;
        Ok(Self {store,instance,draw})
    }
    pub fn frame(&mut self,x:i32,y:i32,down:i32,ms:i32)->Result<Vec<i32>,StateError>{
        self.store.set_fuel(20000)?;
        let address=self.draw.call(&mut self.store,(x.clamp(0,319),y.clamp(0,239),i32::from(down!=0),ms.clamp(0,100)))?;
        let mem=self.instance.get_memory(&self.store,"memory").ok_or("missing guest memory")?;
        let start=usize::try_from(address)?;
        let mut header=[0;4];mem.read(&self.store,start,&mut header)?;
        let count=u32::from_le_bytes(header);
        if count>128{return Err("too many draw commands".into())}
        let mut bytes=vec![0;count as usize*20];
        mem.read(&self.store,start.checked_add(4).ok_or("invalid address")?,&mut bytes)?;
        Ok(bytes.chunks_exact(4).map(|v|i32::from_le_bytes(v.try_into().unwrap())).collect())
    }
}
