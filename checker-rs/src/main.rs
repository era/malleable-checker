use std::{error::Error};
use std::fmt;
use std::fmt::Debug;
use std::str;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, sync::WasiCtxBuilder};


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

fn main() -> Result<(), Box<dyn Error>> {
    println!("=======");
    println!("running the checker");
    let store = exec_checker_from_file("examples/read_from_stdin.wat", "check")?;
    println!("done");

    println!("=======");

    let checker = store.data();

    if checker.success.len() > 0 {
        println!("the following success messages were sent from the checker");
        for success_message in &checker.success {
            println!("{success_message}");
        }
        println!("=======");
    } else {
        println!("No succcess message found");
        println!("=======");
    }

   
    if checker.failures.len() > 0 {
        println!("the following failure messages were sent from the checker");
        for failure_message in &checker.failures {
            println!("{failure_message}");
        }
        println!("=======");
    } else {
        println!("No failure message found");
        println!("=======");
    }

   Ok(())
}


fn exec_checker_from_file(path: &str, func: &str) -> Result<Store<Checker>, Box<dyn Error>> {
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
    let linker = create_linker(&engine);

    // With a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = linker.instantiate(&mut store, &module)?;

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

    linker.func_wrap("checker", "fail", fail).unwrap();//TODO
    linker.func_wrap("checker", "succeed", succeed).unwrap();//TODO
    
    linker
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