use std::{error::Error, sync::Arc, sync::Mutex};
use wasmtime::*;

#[derive(Debug, Default)]
pub struct Checker {
    pub failures: Mutex<Vec<String>>,
    pub success: Mutex<Vec<String>>,
}

fn main() -> Result<(), Box<dyn Error>> {
   let checker = exec_checker_from_file("hello.wasm", "check")?;
   println!("checker state: {:?}", checker);
   Ok(())
}

fn exec_checker_from_file(path: &str, func: &str) -> Result<Arc<Checker>, Box<dyn Error>> {
    let checker = Arc::new(Checker::default());

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
    let mut store = Store::new(&engine, 0);

    let linker = create_linker(&engine, checker.clone());

    // With a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = linker.instantiate(&mut store, &module)?;

    // The `Instance` gives us access to various exported functions and items,
    // which we access here to pull out our `answer` exported function and
    // run it.
    let answer = instance
        .get_func(&mut store, func)
        .expect(format!("`{func}` was not an exported function").as_str());

        let answer = answer.typed::<(), _, _>(&store)?;

    answer.call(&mut store, ())?;

    println!("DONE");
    Ok(checker)
}


fn create_linker<T>(engine: &Engine, checker: Arc<Checker>) -> Linker<T> {
    let mut linker = Linker::new(&engine);
    // any param goes after caller
    let checker = checker.clone();
    linker.func_wrap("host", "hello", move |caller: Caller<'_, T>| {
        println!("this comes from host (rust)");
        checker.failures.lock().unwrap().push("it failed :C".to_string()); //TODO
    }).unwrap();//TODO
    linker
}