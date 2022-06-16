#![feature(slice_flatten)]
use std::collections::HashMap;
use std::error::Error;

use clap::{App, Arg};

mod checker;
mod memory;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Malleable Checker runner")
        .version("0.1.0")
        .author("Elias")
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .value_name("FILE")
                .help("Sets a custom wasm file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ds_name")
                .short("n")
                .long("ds_name")
                .value_name("NAME")
                .help("Name of the dataset")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("dataset") // in the future we should allow several datasets and not only one
                .short("d")
                .long("dataset")
                .value_name("CSV CONTENT")
                .help("Uses it as dataset")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut dataset = HashMap::<String, String>::default();
    dataset.insert(
        matches.value_of("ds_name").unwrap().into(),
        matches.value_of("dataset").unwrap().into(),
    );

    run(matches.value_of("code").unwrap(), dataset)?;

    Ok(())
}

fn run(checker: &str, dataset: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    println!("=======");
    println!("running the checker");
    let store = checker::exec_checker_from_file(checker, "check", dataset)?;
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

