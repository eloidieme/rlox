use std::error::Error;
use std::io::{Stdin, Write};
use std::path::Path;
use std::process::exit;
use std::{env, io};
use scanner::Scanner;
use token_type::Token;

mod scanner;
mod token_type;

struct ErrorReporter {
    had_error: bool,
}

impl ErrorReporter {
    fn new() -> Self {
        Self { had_error: false }
    }

    fn set_error(&mut self) {
        self.had_error = true;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut error_reporter = ErrorReporter::new();

    if args.len() > 2 {
        eprintln!("Usage: rlox [script]");
        exit(64);
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1], &mut error_reporter) {
            eprintln!("Error: {}", e);
            error_reporter.set_error();
        }
    } else {
        if let Err(e) = run_prompt(&mut error_reporter) {
            eprintln!("Error: {}", e);
            error_reporter.set_error();
        }
    }

    if error_reporter.had_error {
        exit(65);
    }

    Ok(())
}

fn run_file(path: &str, error_reporter: &mut ErrorReporter) -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(Path::new(path))?;
    let source = String::from_utf8(bytes)?;
    run(&source, error_reporter);

    Ok(())
}

fn run_prompt(error_reporter: &mut ErrorReporter) -> Result<(), Box<dyn Error>> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let stdin: Stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        if input.is_empty() {
            break;
        }

        run(&input, error_reporter);
    }

    Ok(())
}

fn run(source: &str, error_reporter: &mut ErrorReporter) {
    let mut scanner: Scanner = Scanner::new(source.to_string(), error_reporter);
    let tokens: &[Token] = scanner.scan_tokens();

    for token in tokens {
       println!("{:?}", token);
    }
}

fn error(error_reporter: &mut ErrorReporter, line_no: usize, message: &str) {
    report(error_reporter, line_no, "", message);
}

fn report(error_reporter: &mut ErrorReporter, line_no: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line_no, location, message);
    error_reporter.set_error();
}
