use wasmtime::WasmResults;
use wasmtime::{Instance, Memory, MemoryType, Store};

pub const WASM_PAGE_SIZE: usize = 65536;
#[derive(Debug)]
pub struct Error {
    pub message: String,
}

// super naive memory manager that just keeps adding in to the end of the
// array without never looking back to reclaim memory
pub struct MemoryManager<T> {
    last_alloc_ptr: usize,
    // this is the memory we are importing from wasm
    // we probably want to keep two, one for communicating back and forth with host <> guest
    // the other one to give the datasets to the guest (wasm env)
    // today this is not supported, waiting for https://github.com/WebAssembly/multi-memory
    pub memory_name: String,
    pub allocations: Vec<Item>,
    pub store: Store<T>,
    pub instance: Instance,
}

pub struct Item {
    pub offset: usize,
    pub size: usize,
}

impl<T> MemoryManager<T> {
    pub fn new(start_offset: usize, name: &str, store: Store<T>, instance: Instance) -> Self {
        Self {
            last_alloc_ptr: start_offset,
            allocations: vec![],
            memory_name: name.to_string(),
            store: store,
            instance: instance,
        }
    }
    pub fn new_memory<A>(&self, store: &mut Store<A>) -> Result<Memory, Error> {
        let memory_type = MemoryType::new(1, None);

        match Memory::new(store, memory_type) {
            Ok(m) => Ok(m),
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }
    // Memory should not be hold for long, the memory can be expanded or changed inside the wasm and our pointers
    // and reference to it will be invalid
    pub fn get(&mut self) -> Result<Memory, Error> {
        let memory = self
            .instance
            .get_memory(&mut self.store, &self.memory_name)
            .ok_or(Error {
                message: "failed to find `memory` export".to_owned(),
            })?;

        Ok(memory)
    }

    //TODO tests
    // write into the store memory the buffer
    // it also pushes the item to self.allocations. So if you want to know where your data was written
    // you can check self.last_item()
    // the layout is always | usize | <buffer content> |
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // for how to properly get offset read https://radu-matei.com/blog/practical-guide-to-wasm-memory/#passing-arrays-to-modules-using-wasmtime
        // going to do something bad here, every time we are asked to copy the buffer into the wasm memory
        // we are going to allocate more memory and put it there.
        // we are not keeping note of that data, which will lead to memory leak
        // since the plan is to only pass the dataset the user needs for writing down alarm rules
        // and it should stay in memory for the whole execution time
        // this is not so bad. But it should not be used in other contexts where we don't want
        // 'static' data.

        let memory = self.get()?;

        [[1, 2, 3], [4, 5, 6]].flatten();

        let item = Item {
            offset: self.last_alloc_ptr,
            size: buffer.len() + 1,
        };

        let buffer_size = buffer.len().to_string();
        let size = buffer_size.as_bytes();

        let buffer: &[u8] = &[size, buffer].concat();

        match memory.write(&mut self.store, item.offset, buffer.into()) {
            Err(_) => {
                // MemoryAccessError
                memory.grow(&mut self.store, 1).or_else(|err| {
                    Err(Error {
                        message: err.to_string(),
                    })
                })?; //TODO very naive to assume we only need one more page

                memory
                    .write(&mut self.store, item.offset, buffer)
                    .or_else(|err| {
                        Err(Error {
                            message: err.to_string(),
                        })
                    })?;
                self.last_alloc_ptr += item.size + 1;
            }
            _ => self.last_alloc_ptr += item.size + 1,
        };

        self.allocations.push(item);

        Ok(())
    }

    // last_item returns the last allocated item in this memory manager
    // it clones the Item struct
    pub fn last_item(&self) -> Option<Item> {
        if let Some(e) = self.allocations.last() {
            Some(Item {
                offset: e.offset,
                size: e.size,
            })
        } else {
            None
        }
    }
    //TODO we should be able to properly use the result type,
    // but Rust is not happy with () as R
    pub fn exec_func<P: WasmResults, R>(
        &mut self,
        func_name: &str,
        params: P,
    ) -> Result<(), Error> {
        // The `Instance` gives us access to various exported functions and items,
        // which we access here to pull out our `func` exported function and
        // run it.
        let func = self
            .instance
            .get_func(&mut self.store, func_name)
            .expect(format!("`{func_name}` function was not exported or does not exist").as_str());

        let func = match func.typed::<P, (), _>(&mut self.store) {
            Ok(func) => func,
            Err(e) => {
                return Err(Error {
                    message: e.to_string(),
                })
            }
        };

        match func.call(&mut self.store, params) {
            Ok(result) => Ok(result),
            Err(e) => Err(Error {
                message: e.to_string(),
            }),
        }
    }
}
