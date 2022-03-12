use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::object::Object;
use crate::parser::Parser;
use crate::scanner::Scanner;

fn run(input: String) -> Result<Vec<Object>, LoxError> {
    let mut scn = Scanner::new(input);
    scn.scan_tokens()?;

    let tokens = scn.get_tokens();

    let mut parser = Parser::new(tokens);

    let statements = parser.parse()?;

    let log = Interpreter::interpret(statements)?;

    Ok(log)
}

fn get_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn block() {
    let code = get_file("./tests/1.lox");
    let res = run(code).unwrap();

    assert_eq!(
        res,
        vec![
            Object::String("inner a".to_string()),
            Object::String("outer b".to_string()),
            Object::String("global c".to_string()),
            Object::String("outer a".to_string()),
            Object::String("outer b".to_string()),
            Object::String("global c".to_string()),
            Object::String("global a".to_string()),
            Object::String("global b".to_string()),
            Object::String("global c".to_string()),
        ]
    );
}

#[test]
fn if_then_else() {
    let code = get_file("./tests/2.lox");
    let res = run(code).unwrap();

    assert_eq!(
        res,
        vec![
            Object::String("i >= 2".to_string()),
            Object::String("i < 2".to_string()),
            Object::String("i < 1".to_string()),
        ]
    );
}

#[test]
fn or() {
    let code = get_file("./tests/3.lox");
    let res = run(code).unwrap();

    assert_eq!(
        res,
        vec![
            Object::String("hi".to_string()),
            Object::String("yes".to_string()),
        ]
    );
}

#[test]
fn while_fac() {
    let code = get_file("./tests/4.lox");
    let res = run(code).unwrap();

    assert_eq!(res, vec![Object::Number(120.0),]);
}

// for loop test
