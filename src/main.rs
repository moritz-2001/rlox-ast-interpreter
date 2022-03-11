use std::{io::{self, Write}};
use Rlox::LoxError;
use Rlox::scanner::Scanner;
use Rlox::parser::Parser;

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
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut s = String::new();
        stdin.read_line(&mut s)?;
        if s.is_empty() { break }
        run(s)?;
    }
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

    let mut parser = Parser::new(scn.get_tokens());
    let expr = parser.parse()?;

    println!("{:#?}", expr);

    Ok(())
}
