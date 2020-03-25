extern crate clap;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Sea Lantern Minecraft Runtime")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Runs a Java program, redirecting its input and output to a socket while preserving writing to real STDOUT.")
        .arg(Arg::with_name("java")
                 .long("java-executable")
                 .required(true)
                 .takes_value(true)
                 .help("Path to the Java executable"))
        .arg(Arg::with_name("server-jar")
                 .long("server-jar")
                 .takes_value(true)
                 .required(true)
                 .help("Path to the Minecraft server JAR"))
        .arg(Arg::with_name("workdir")
                 .long("working-directory")
                 .takes_value(true)
                 .help("Working directory when executing Java"))
        .arg(Arg::with_name("socket")
                 .long("socket")
                 .required(true)
                 .takes_value(true)
                 .help("URL of Socket to read/write from"))
        .get_matches();

    let myfile = matches.value_of("java").unwrap_or("input.txt");
    println!("The file passed is: {}", myfile);

    let num_str = matches.value_of("num");
    match num_str {
        None => println!("No idea what your favorite number is."),
        Some(s) => match s.parse::<i32>() {
            Ok(n) => println!("Your favorite number must be {}.", n + 5),
            Err(_) => println!("That's not a number! {}", s),
        },
    }
}
