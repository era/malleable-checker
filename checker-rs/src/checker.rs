use std::{error::Error};
use std::fmt;
use std::fmt::Debug;
use std::str;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, sync::WasiCtxBuilder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::memory::{MemoryManager, WASM_PAGE_SIZE};

#[derive(Debug, Default)]
pub struct Datasets {
    items: HashMap<String, Dataset>
}

#[derive(Debug, Clone, Copy)]
pub struct Dataset {
    offset: usize,
    size: usize,
}

impl Datasets {
    fn write_in_memory<T>(&mut self, mut store: Store<T>, instance: &Instance, buffer: &[u8], name: String, memory_manager: &mut MemoryManager) -> Store<T>  {
        store = memory_manager.write(store, &instance, buffer).expect("could not write dataset into wasm memory");

        let item = memory_manager.last_item().expect("dataset offset not pushed to memory manager");

        self.items.insert(name, Dataset{offset: item.offset, size: item.size});

        store
    }
}

pub struct Checker {
    pub failures: Vec<String>,
    pub success: Vec<String>,
    // https://docs.wasmtime.dev/examples-rust-wasi.html
    wasi: WasiCtx,
}

impl Default for Checker {
    fn default() -> Self { 
        let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args().expect("could not create wasi context")
        .build();

        Checker { failures: vec![], success: vec![], wasi: wasi}
    }
}

impl Debug for Checker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Checker")
         .field("failures", &self.failures)
         .field("success", &self.success)
         .finish()
    }
}

//TODO: alloc first page of memory to be used inside wasm freely
// second > pages are owned by Rust host and should not be used
// in an idea world with https://github.com/WebAssembly/multi-memory support
// we would have one memory for the communication between host <> guest
// and another for the datasets
pub fn exec_checker_from_file(path: &str, func: &str) -> Result<Store<Checker>, Box<dyn Error>> {

    let datasets = Arc::new(Mutex::new(Datasets::default()));

    //checker holds the state of the checks (failed/success)
    let checker = Checker::default();

    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::default();

    // We start off by creating a `Module` which represents a compiled form
    // of our input wasm module. In this case it'll be JIT-compiled after
    // we parse the text format.
    //could use from_binary as well
    let module = Module::from_file(&engine, path)?;

    // A `Store` is what will own instances, functions, globals, etc. All wasm
    // items are stored within a `Store`, and it's what we'll always be using to
    // interact with the wasm world. Custom data can be stored in stores but for
    // now we just use `()`.
    let mut store = Store::new(&engine, checker);

    // the linker will link our host functions to the wasm env
    let mut linker = create_linker(&engine);
    add_functions(&mut linker, datasets.clone());

    // With a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = linker.instantiate(&mut store, &module)?;

    let mut memory_manager = MemoryManager::new(WASM_PAGE_SIZE, "memory");

    //TODO receive as paramter
    let buffer = "1,cool;2,not_cool";
    //write dataset
    let mut store = datasets.lock().unwrap().write_in_memory(store, &instance, buffer.as_bytes(), "test".into(),&mut memory_manager);


    // The `Instance` gives us access to various exported functions and items,
    // which we access here to pull out our `func` exported function and
    // run it.
    let answer = instance
        .get_func(&mut store, func)
        .expect(format!("`{func}` was not an exported function").as_str());

        let answer = answer.typed::<(), _, _>(&store)?;

    answer.call(&mut store, ())?;

    Ok(store)
}

fn create_linker(engine: &Engine) -> Linker<Checker> {
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |state: &mut Checker| &mut state.wasi).unwrap();
    
    linker
}

fn add_functions(linker: &mut Linker<Checker>, datasets: Arc<Mutex<Datasets>>) ->() {
    
    linker.func_wrap("checker", "fail", fail).unwrap();//TODO
    linker.func_wrap("checker", "succeed", succeed).unwrap();//TODO
    linker.func_wrap("checker", "datasets", move |mut caller: Caller<'_, Checker>, ptr: i32, len: i32 |  {
        let key = match get_string(&mut caller, ptr, len) {
            Ok(e) => e,
            Err(e) => return Err::<(i32, i32), Trap>(e),
        };
        println!("{key}");
        println!("{:?}", datasets);
        
        match datasets.lock().unwrap().items.get(&key) {
            Some(result) => Ok::<(i32, i32), Trap>((result.offset as i32, result.size as i32)), //TODO
            None => Err(Trap::new("no dataset with that name")),
        }
    }).unwrap();
}

fn fail(mut caller: Caller<'_, Checker>, ptr: i32, len: i32) -> Result<(), Trap> {
    let string = get_string(&mut caller, ptr, len)?;
    caller.data_mut().failures.push(string);
    Ok(())
}

fn succeed(mut caller: Caller<'_, Checker>, ptr: i32, len: i32) -> Result<(), Trap> {
    let string = get_string(&mut caller, ptr, len)?;
    caller.data_mut().success.push(string);
    Ok(())
}

fn get_string(caller: &mut Caller<'_, Checker>, ptr: i32, len: i32) -> Result<String, Trap> {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Trap::new("failed to find host memory")),
    };
            
    let data = mem.data(caller)
        .get(ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));
    
        
    let string = match data {
        Some(data) => match str::from_utf8(data) {
            Ok(s) => s.to_owned(),
            Err(_) => return Err(Trap::new("invalid utf-8")),
        },
        None => return Err(Trap::new("pointer/length out of bounds")),
    };

    Ok(string)
}