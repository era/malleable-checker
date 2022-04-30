use std::{error::Error};


mod checker;
mod memory;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=======");
    println!("running the checker");
    let store = checker::exec_checker_from_file("examples/read_from_stdin.wat", "check")?;
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
