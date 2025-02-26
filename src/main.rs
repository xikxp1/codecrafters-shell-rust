#[allow(unused_imports)]
use std::io::{self, Write};

enum BulitinCommand {
    Exit,
    Echo,
    Type,
}

impl BulitinCommand {
    fn from_str(command: &str) -> Option<Self> {
        match command {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            _ => None,
        }
    }

    fn to_impl(&self) -> fn(&[&str]) {
        match self {
            Self::Exit => exit_fn,
            Self::Echo => echo_fn,
            Self::Type => type_fn,
        }
    }
}

fn exit_fn(args: &[&str]) {
    let exit_code = if !args.is_empty() {
        args[0].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    std::process::exit(exit_code);
}

fn echo_fn(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn type_fn(args: &[&str]) {
    for arg in args {
        match BulitinCommand::from_str(arg) {
            Some(_) => println!("{} is a shell builtin", arg),
            None => println!("{}: not found", arg),
        }
    }
}

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
        let command_str = tokens[0];
        let args_str = &tokens[1..];
        let command = match BulitinCommand::from_str(command_str) {
            Some(command) => command,
            None => {
                println!("{}: command not found", command_str);
                continue;
            }
        };

        let command_fn = command.to_impl();
        command_fn(args_str);
    }
}
