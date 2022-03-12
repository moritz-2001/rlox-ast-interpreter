use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::object::Object;
use crate::parser::Parser;
use crate::scanner::Scanner;

fn run(input: String) -> Result<Object, LoxError> {
    let mut scn = Scanner::new(input);
    scn.scan_tokens()?;

    let tokens = scn.get_tokens();

    let mut parser = Parser::new(tokens);

    let statements = parser.parse()?;

    Interpreter::interpret(statements)
}

fn get_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}



#[test]
fn fun_1() {
    let code = get_file("./tests/6.lox");
    let res = run(code).unwrap();

    assert_eq!(res, Object::String("Hi, Dear Reader!".to_string()));
}