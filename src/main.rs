#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input_string = input.trim();
        let tokens = input_string.split_whitespace().collect::<Vec<&str>>();
        if tokens.is_empty() {
            continue;
        }
        let command = tokens[0];
        let args = &tokens[1..];
        match command {
            "exit" => {
                let exit_code = if !args.is_empty() {
                    args[0].parse::<i32>().unwrap_or(0)
                } else {
                    0
                };
                std::process::exit(exit_code);
            }
            "echo" => {
                println!("{}", args.join(" "));
            }
            _ => {
                println!("{}: command not found", command);
            }
        }
    }
}
