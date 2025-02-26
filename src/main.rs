#[allow(unused_imports)]
use std::io::{self, Write};

enum Command {
    BuiltinCommand(BuiltinCommand),
    ExecutableCommand(ExecutableCommand),
}

enum BuiltinCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

#[derive(Clone, Debug)]
struct OutputLine {
    line: String,
    is_err: bool,
}

#[derive(Debug)]
struct Output(Vec<OutputLine>);

impl Output {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, line: &str, is_err: bool) {
        self.0.push(OutputLine {
            line: line.to_string(),
            is_err,
        });
    }

    fn get(&self) -> Vec<OutputLine> {
        self.0.clone()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl BuiltinCommand {
    fn from_str(command: &str) -> Option<Self> {
        match command {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            "pwd" => Some(Self::Pwd),
            "cd" => Some(Self::Cd),
            _ => None,
        }
    }

    fn to_impl(&self) -> fn(&[&str], &mut Output) {
        match self {
            Self::Exit => exit_fn,
            Self::Echo => echo_fn,
            Self::Type => type_fn,
            Self::Pwd => pwd_fn,
            Self::Cd => cd_fn,
        }
    }
}

struct ExecutableCommand {
    path: String,
}

fn exit_fn(args: &[&str], output: &mut Output) {
    if args.len() > 1 {
        output.add("exit: too many arguments", true);
        return;
    }
    let exit_code = if !args.is_empty() {
        args[0].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    std::process::exit(exit_code);
}

fn echo_fn(args: &[&str], output: &mut Output) {
    output.add(&args.join(" "), false);
}

fn type_fn(args: &[&str], output: &mut Output) {
    if args.is_empty() {
        output.add("type: missing argument", true);
        return;
    }
    if args.len() > 1 {
        output.add("type: too many arguments", true);
        return;
    }
    let command = search_command(args[0]);
    match command {
        Some(Command::BuiltinCommand(_)) => {
            output.add(&format!("{} is a shell builtin", args[0]), false);
        }
        Some(Command::ExecutableCommand(executable)) => {
            output.add(&format!("{} is {}", args[0], executable.path), false);
        }
        None => {
            output.add(&format!("{}: not found", args[0]), true);
        }
    }
}

fn pwd_fn(args: &[&str], output: &mut Output) {
    if !args.is_empty() {
        output.add("pwd: too many arguments", true);
        return;
    }
    let current_dir = std::env::current_dir();
    if current_dir.is_err() {
        output.add("pwd: unable to get current directory", true);
        return;
    }
    output.add(&current_dir.unwrap().display().to_string(), false);
}

fn cd_fn(args: &[&str], output: &mut Output) {
    if args.is_empty() {
        // If no args provided, change to HOME directory
        if let Ok(home) = std::env::var("HOME") {
            if let Err(_) = std::env::set_current_dir(&home) {
                output.add(&format!("cd: {}: No such file or directory", home), true);
            }
        } else {
            output.add("cd: unable to get home directory", true);
        }
        return;
    }
    if args.len() > 1 {
        output.add("cd: too many arguments", true);
        return;
    }
    let new_dir = if args[0] == "~" {
        std::env::var("HOME")
    } else {
        Ok(args[0].to_string())
    };
    if new_dir.is_err() {
        output.add("cd: unable to get home directory", true);
        return;
    }
    let new_dir = new_dir.unwrap();
    let cd_result = std::env::set_current_dir(&new_dir);
    if cd_result.is_err() {
        output.add(&format!("cd: {}: No such file or directory", new_dir), true);
    }
}

fn search_command(command: &str) -> Option<Command> {
    // First check if it's a builtin command
    if let Some(builtin) = BuiltinCommand::from_str(command) {
        return Some(Command::BuiltinCommand(builtin));
    }

    // Then check if it's an executable in PATH
    if let Some(exec) = pathsearch::find_executable_in_path(command) {
        return Some(Command::ExecutableCommand(ExecutableCommand {
            path: exec.display().to_string(),
        }));
    }

    None
}

#[derive(Debug)]
struct TokenizerResult {
    command: String,
    args: Vec<String>,
    redirect_stdout: Option<String>,
    append_stdout: bool,
    redirect_stderr: Option<String>,
    append_stderr: bool,
}

fn handle_tokens(tokens: Vec<String>) -> Option<TokenizerResult> {
    if tokens.is_empty() {
        return None;
    }

    let command_str = tokens[0].as_str();
    let mut result = TokenizerResult {
        command: command_str.to_string(),
        args: Vec::new(),
        redirect_stdout: None,
        append_stdout: false,
        redirect_stderr: None,
        append_stderr: false,
    };

    let mut i = 1;
    while i < tokens.len() {
        match tokens[i].as_str() {
            ">" | "1>" => {
                if i + 1 >= tokens.len() {
                    eprintln!("syntax error: missing file name after redirection operator");
                    return None;
                }
                result.redirect_stdout = Some(tokens[i + 1].to_string());
                i += 2;
            }
            "2>" => {
                if i + 1 >= tokens.len() {
                    eprintln!("syntax error: missing file name after redirection operator");
                    return None;
                }
                result.redirect_stderr = Some(tokens[i + 1].to_string());
                i += 2;
            }
            ">>" | "1>>" => {
                if i + 1 >= tokens.len() {
                    eprintln!("syntax error: missing file name after redirection operator");
                    return None;
                }
                result.redirect_stdout = Some(tokens[i + 1].to_string());
                result.append_stdout = true;
                i += 2;
            }
            "2>>" => {
                if i + 1 >= tokens.len() {
                    eprintln!("syntax error: missing file name after redirection operator");
                    return None;
                }
                result.redirect_stderr = Some(tokens[i + 1].to_string());
                result.append_stderr = true;
                i += 2;
            }
            arg => {
                result.args.push(arg.to_string());
                i += 1;
            }
        }
    }

    Some(result)
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        let readl_line = stdin.read_line(&mut input);
        if readl_line.is_err() {
            eprintln!("Error reading input: {}", readl_line.err().unwrap());
            continue;
        }
        let input_string = input.trim();
        let tokenizer_result = shell_words::split(input_string);
        if tokenizer_result.is_err() {
            eprintln!("Error parsing input: {}", tokenizer_result.err().unwrap());
            continue;
        }
        let tokens = tokenizer_result.unwrap();
        if tokens.is_empty() {
            continue;
        }

        let tokenized = handle_tokens(tokens);
        if tokenized.is_none() {
            continue;
        }
        let tokenized = tokenized.unwrap();
        let command_str = tokenized.command.as_str();
        let args_str = tokenized
            .args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        let redirect_stdout = tokenized.redirect_stdout;
        let append_stdout = tokenized.append_stdout;
        let redirect_stderr = tokenized.redirect_stderr;
        let append_stderr = tokenized.append_stderr;

        // Create base OpenOptions for output and error files
        let mut base_out_options = std::fs::OpenOptions::new();
        base_out_options.write(true).create(true);

        let mut base_err_options = std::fs::OpenOptions::new();
        base_err_options.write(true).create(true);

        // Add mode-specific flags
        if append_stdout {
            base_out_options.append(true);
        } else {
            base_out_options.truncate(true);
        }

        if append_stderr {
            base_err_options.append(true);
        } else {
            base_err_options.truncate(true);
        }

        let out_file = redirect_stdout.as_ref().map(|path| {
            base_out_options.open(path).unwrap_or_else(|e| {
                eprintln!("Error opening output file {}: {}", path, e);
                std::process::exit(1);
            })
        });

        let err_file = redirect_stderr.as_ref().map(|path| {
            base_err_options.open(path).unwrap_or_else(|e| {
                eprintln!("Error opening error file {}: {}", path, e);
                std::process::exit(1);
            })
        });

        // Create writers from the file handles
        let mut out_writer: Box<dyn Write> = if let Some(file) = out_file {
            Box::new(file)
        } else {
            Box::new(io::stdout())
        };

        let mut err_writer: Box<dyn Write> = if let Some(file) = err_file {
            Box::new(file)
        } else {
            Box::new(io::stderr())
        };

        let mut output = Output::new();

        match search_command(command_str) {
            Some(Command::BuiltinCommand(builtin)) => {
                let command_fn = builtin.to_impl();
                command_fn(&args_str, &mut output);
                for line in output.get() {
                    if line.is_err {
                        writeln!(err_writer, "{}", line.line).unwrap();
                    } else {
                        writeln!(out_writer, "{}", line.line).unwrap();
                    }
                }
            }
            Some(Command::ExecutableCommand(_)) => {
                // Reuse the base options we created earlier
                if std::process::Command::new(command_str)
                    .args(args_str)
                    .stdout(if let Some(ref path) = redirect_stdout {
                        let file = base_out_options.open(path).unwrap_or_else(|e| {
                            eprintln!("Error opening output file {}: {}", path, e);
                            std::process::exit(1);
                        });
                        std::process::Stdio::from(file)
                    } else {
                        std::process::Stdio::inherit()
                    })
                    .stderr(if let Some(ref path) = redirect_stderr {
                        let file = base_err_options.open(path).unwrap_or_else(|e| {
                            eprintln!("Error opening error file {}: {}", path, e);
                            std::process::exit(1);
                        });
                        std::process::Stdio::from(file)
                    } else {
                        std::process::Stdio::inherit()
                    })
                    .spawn()
                    .and_then(|mut child| child.wait())
                    .is_err()
                {
                    eprintln!("{}: command not found", command_str);
                }
            }
            None => {
                eprintln!("{}: command not found", command_str);
            }
        }

        output.clear();
    }
}
