use wasmtime::{Memory, Store, MemoryType, Instance};

#[derive(Debug)]
pub struct Error{
    pub message: String,
}

pub fn new<T>(store: &mut Store<T>) -> Result<Memory, Error> {
    let memory_type = MemoryType::new(1, None);

    match Memory::new(store, memory_type) {
        Ok(m) => Ok(m),
        Err(error) => Err(Error{message: error.to_string()})
    }
}
// Memory should not be hold for long, the memory can be expanded or changed inside the wasm and our pointers
// and reference to it will be invalid
pub fn get<T>(store: &mut Store<T>, instance: Instance) -> Result<Memory, Error> {
    let memory = instance
        .get_memory(store, "memory")
        .ok_or(Error{message: "failed to find `memory` export".to_owned()})?;
        
        Ok(memory)
}

pub fn write<T>(store: &mut Store<T>, instance: Instance, offset: usize, buffer: &[u8]) -> Result<(), Error> {
    //TODO check if the size is enough
    get(store, instance)?
        .write(store, offset, buffer)
        .or_else(|err| Err(Error{message: err.to_string()}))?;

    Ok(())
}