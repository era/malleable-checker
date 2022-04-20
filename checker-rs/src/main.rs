use std::error::Error;
use wasmtime::*;

fn main() -> Result<(), Box<dyn Error>> {
   exec_checker_from_file("hello.wasm", "check")
}

fn exec_checker_from_file(path: &str, func: &str) -> Result<(), Box<dyn Error>> {
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

    let linker = create_linker(&engine);

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
    Ok(())
}


fn create_linker<T>(engine: &Engine) -> Linker<T> {
    let mut linker = Linker::new(&engine);
    // any param goes after caller
    linker.func_wrap("host", "hello", |caller: Caller<'_, T>| {
        println!("this comes from host (rust)");
    }).unwrap();//TODO
    linker
    // (Instance::new(&mut store, module, &[host_hello.into()]).unwrap(), store) //TODO remove
}