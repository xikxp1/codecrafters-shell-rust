#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

enum Command {
    BulitinCommand(BulitinCommand),
    ExecutableCommand(ExecutableCommand),
}

enum BulitinCommand {
    Exit,
    Echo,
    Type,
    Pwd,
}

impl BulitinCommand {
    fn from_str(command: &str) -> Option<Self> {
        match command {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            "pwd" => Some(Self::Pwd),
            _ => None,
        }
    }

    fn to_impl(&self) -> fn(&[&str]) {
        match self {
            Self::Exit => exit_fn,
            Self::Echo => echo_fn,
            Self::Type => type_fn,
            Self::Pwd => pwd_fn,
        }
    }
}

struct ExecutableCommand {
    path: String,
}

fn exit_fn(args: &[&str]) {
    if args.len() > 1 {
        println!("exit: too many arguments");
        return;
    }
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
    if args.is_empty() {
        println!("type: missing argument");
        return;
    }
    if args.len() > 1 {
        println!("type: too many arguments");
        return;
    }
    let command = search_command(args[0]);
    match command {
        Some(Command::BulitinCommand(_)) => {
            println!("{} is a shell builtin", args[0]);
        }
        Some(Command::ExecutableCommand(executable)) => {
            println!("{} is {}", args[0], executable.path);
        }
        None => {
            println!("{}: not found", args[0]);
        }
    }
}

fn pwd_fn(args: &[&str]) {
    if !args.is_empty() {
        println!("pwd: too many arguments");
        return;
    }
    let current_dir = std::env::current_dir();
    if current_dir.is_err() {
        println!("pwd: unable to get current directory");
        return;
    }
    println!("{}", current_dir.unwrap().display());
}

fn search_command(command: &str) -> Option<Command> {
    // First check if it's a builtin command
    if let Some(builtin) = BulitinCommand::from_str(command) {
        return Some(Command::BulitinCommand(builtin));
    }

    // Then check if it's an executable in PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for path in path_var.split(':') {
            let command_path = format!("{}/{}", path, command);
            let path = std::path::Path::new(&command_path);

            // Check if file exists, is a file, and is executable
            if path.exists() && path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    if metadata.permissions().mode() & 0o111 != 0 {
                        return Some(Command::ExecutableCommand(ExecutableCommand {
                            path: command_path,
                        }));
                    }
                }
            }
        }
    }

    None
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

        match search_command(command_str) {
            Some(Command::BulitinCommand(builtin)) => {
                let command_fn = builtin.to_impl();
                command_fn(args_str);
            }
            Some(Command::ExecutableCommand(_)) => {
                let command = std::process::Command::new(command_str)
                    .args(args_str)
                    .spawn();
                if command.is_err() {
                    println!("{}: command not found", command_str);
                    continue;
                }
                let _ = command.unwrap().wait();
            }
            None => {
                println!("{}: command not found", command_str);
            }
        }
    }
}
