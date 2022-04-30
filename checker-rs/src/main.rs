use std::{error::Error};
use std::env;

mod checker;
mod memory;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(file) = args.get(1) {
        run(file)?;
    } else {
        panic!("expecting path to checker file as first argument");
    }

   Ok(())
}

fn run(checker: &str) -> Result<(), Box<dyn Error>> {
    println!("=======");
    println!("running the checker");
    let store = checker::exec_checker_from_file(checker, "check")?;
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