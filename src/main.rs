#![allow(unused_variables)]
mod ast;
mod error;
mod faestd;
mod lexer;
mod runtime;

use std::{env::args, fs, process::exit};

use ast::*;
use error::*;
use lexer::*;
use runtime::*;

fn main() {
    let Some(arg) = args().nth(1) else {
        println!("USAGE: fae <file>.fae");
        exit(1);
    };
    if !fs::exists(&arg).map_or(false, |b| b) {
        println!("File doesn't exist");
        exit(1);
    }
    let input = fs::read_to_string(arg).unwrap();
    let errs = FErrorManager::new(input.clone());
    let mut lexer = Lexer::new(input);
    lexer.try_all();
    debug!("== LEXER ==");
    let mut toks = Vec::new();
    for token in &lexer.tokens {
        if let TokenKind::Error(s) = &token.kind {
            errs.print_error(&token.pos, format!("{s}"));
        } else {
            toks.push(token.clone());
            debug!("{:?}", token);
        }
    }
    if toks.len() != lexer.tokens.len() {
        exit(1);
    }
    let mut parser = Parser::new(toks);
    parser.try_all();
    debug!("== PARSER ==");
    let mut nodes = Vec::new();
    for node in &parser.nodes {
        if let NodeKind::Error(s) = &node.kind {
            errs.print_error(&node.pos, format!("{s}"));
        } else {
            nodes.push(node.clone());
            debug!("{:#?}", node);
        }
    }
    if nodes.len() != parser.nodes.len() {
        exit(1);
    }
    let mut scope = Scope::new(errs);
    scope.run_block(&nodes);
    if let Some(_) = scope.funcs.get("main").cloned() {
        scope.stack.push(Value::Stack(Stack {
            values: args().map(|s| Value::String(s)).collect(),
        }));
        scope.run_block(&vec![Node {
            kind: NodeKind::Instruction("main".to_string()),
            pos: Span { start: 0, end: 0 },
        }]);
    }
}
