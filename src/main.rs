use std::io::{self, Write};
use rlox::parser::Parser;
use rlox::scanner::Scanner;
use rlox::Interpreter;
use rlox::LoxError;

fn main() -> Result<(), LoxError> {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => run_prompt()?,
        2 => run_file(args.get(1).unwrap())?,
        _ => {
            println!("Usage: rlox [script]");
            std::process::exit(64);
        }
    }

    Ok(())
}

fn run_prompt() -> Result<(), LoxError> {
    let stdin = io::stdin();
    let mut code = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut s = String::new();
        stdin.read_line(&mut s)?;
        code += s.trim();
        if s.trim().is_empty() && !code.trim().is_empty() {
            break;
        }
    }
    run(code)?;
    Ok(())
}

fn run_file(path: &str) -> Result<(), LoxError> {
    let input = std::fs::read_to_string(path)?;
    run(input)?;

    Ok(())
}

fn run(input: String) -> Result<(), LoxError> {
    let mut scn = Scanner::new(input);
    scn.scan_tokens()?;

    let tokens = scn.get_tokens();

    let mut parser = Parser::new(tokens);

    let statements = parser.parse()?;

    Interpreter::interpret(statements)?;

    Ok(())
}
