use crate::memory::{MemoryManager, WASM_PAGE_SIZE};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::str;
use std::sync::{Arc, Mutex};
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

#[derive(Debug, Default)]
pub struct Datasets {
    items: HashMap<String, Dataset>,
}

#[derive(Debug, Clone, Copy)]
pub struct Dataset {
    offset: usize,
    size: usize,
}

pub enum Var {
    Str(String, Vec<u8>),
}

impl Datasets {
    fn write_in_memory<T>(&mut self, var: Var, memory_manager: &mut MemoryManager<T>) {
        let (name, buffer) = match var {
            Var::Str(name, buffer) => (name, buffer),
            _ => panic!("not supported yet"),
        };

        memory_manager
            .write(&buffer)
            .expect("could not write dataset into wasm memory");

        let item = memory_manager
            .last_item()
            .expect("dataset offset not pushed to memory manager");

        self.items.insert(
            name,
            Dataset {
                offset: item.offset,
                size: item.size,
            },
        );
    }

    fn from_hash_map<T>(
        &mut self,
        ds: HashMap<String, String>,
        memory_manager: &mut MemoryManager<T>,
    ) {
        for (var_name, content) in ds {
            // assuming everything is a a string (right now a csv)
            let dataset = Var::Str(var_name, content.as_bytes().to_owned());
            //write dataset
            self.write_in_memory(dataset, memory_manager);
        }
    }
}

pub struct Checker {
    pub failures: Vec<String>,
    pub success: Vec<String>,
    // https://docs.wasmtime.dev/examples-rust-wasi.html
    wasi: WasiCtx,
}

impl Checker {
    fn new(
        stdin: wasmtime_wasi::sync::file::File,
        stdout: wasmtime_wasi::sync::file::File,
    ) -> Self {
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(stdin))
            .stdout(Box::new(stdout))
            .build();
        Checker {
            failures: vec![],
            success: vec![],
            wasi: wasi,
        }
    }
}

impl Default for Checker {
    fn default() -> Self {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .expect("could not create wasi context")
            .build();

        Checker {
            failures: vec![],
            success: vec![],
            wasi: wasi,
        }
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
// ds is a Hashmap of DatasetName and Dataset content (as CSV)
pub fn exec_checker_from_file(
    path: &str,
    func: &str,
    user_variables: HashMap<String, String>,
) -> Result<Store<Checker>, Box<dyn Error>> {
    //checker holds the state of the checks (failed/success)
    exec_from_file(path, func, user_variables, Checker::default())
}

pub fn exec_from_file(
    path: &str,
    func: &str,
    user_variables: HashMap<String, String>,
    checker: Checker,
) -> Result<Store<Checker>, Box<dyn Error>> {
    let datasets = Arc::new(Mutex::new(Datasets::default()));

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

    let mut memory_manager = MemoryManager::new(WASM_PAGE_SIZE, "memory", store, instance);

    //write dataset
    datasets
        .lock()
        .unwrap()
        .from_hash_map(user_variables, &mut memory_manager);

    // The `Instance` gives us access to various exported functions and items,
    // which we access here to pull out our `func` exported function and
    // run it.
    memory_manager.exec_func::<(), i32>(func, ()).unwrap();

    Ok(memory_manager.store)
}

fn create_linker(engine: &Engine) -> Linker<Checker> {
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |state: &mut Checker| &mut state.wasi).unwrap();

    linker
}

fn add_functions(linker: &mut Linker<Checker>, datasets: Arc<Mutex<Datasets>>) -> () {
    linker.func_wrap("checker", "fail", fail).unwrap(); //TODO
    linker.func_wrap("checker", "succeed", succeed).unwrap(); //TODO
    linker
        .func_wrap(
            "checker",
            "datasets",
            move |mut caller: Caller<'_, Checker>, ptr: i32, len: i32| {
                let key = match get_string(&mut caller, ptr, len) {
                    Ok(e) => e,
                    Err(e) => return Err::<(i32, i32), Trap>(e),
                };
                println!("{key}");
                println!("{:?}", datasets);

                match datasets.lock().unwrap().items.get(&key) {
                    Some(result) => {
                        Ok::<(i32, i32), Trap>((result.offset as i32, result.size as i32))
                    } //TODO
                    None => Err(Trap::new("no dataset with that name")),
                }
            },
        )
        .unwrap();
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

    let data = mem
        .data(caller)
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

#[cfg(test)]
mod test_checker {
    use super::*;

    #[test]
    fn test_executing_fail() {
        let store = exec_checker_from_file(
            "examples/this_checker_always_fail.wat",
            "check",
            HashMap::<String, String>::default(),
        )
        .unwrap();

        let checker = store.data();
        assert_eq!("This checker always fail", checker.failures.get(0).unwrap());
    }

    #[test]
    fn test_executing_succeeds() {
        let store = exec_checker_from_file(
            "examples/this_checker_always_succeeds.wat",
            "check",
            HashMap::<String, String>::default(),
        )
        .unwrap();

        let checker = store.data();
        assert_eq!(
            "This checker always succeed",
            checker.success.get(0).unwrap()
        );
    }

    #[test]
    fn test_reads_dataset_and_outputs_to_stdout() {
        let ds = "123,456,678,10,12,12";
        let stdin = wasmtime_wasi::sync::file::File::from_cap_std(cap_std::fs::File::from_std(
            File::create("stdin").unwrap(),
        ));
        let stdout = wasmtime_wasi::sync::file::File::from_cap_std(cap_std::fs::File::from_std(
            File::create("stdout").unwrap(),
        ));

        let mut dataset = HashMap::<String, String>::default();
        dataset.insert("test".to_string(), ds.to_string());
        let checker = Checker::new(stdin, stdout);

        let store = exec_from_file(
            "examples/write_to_stdout_dataset_directly.wat",
            "check",
            dataset,
            checker,
        )
        .unwrap();
        let mut stdout = File::open("stdout").unwrap();
        let mut contents = String::new();
        stdout.read_to_string(&mut contents).unwrap();
        let checker = store.data();
        assert_eq!(ds, contents);

        std::fs::remove_file("stdin").expect("File delete failed");
        std::fs::remove_file("stdout").expect("File delete failed");
    }
}
