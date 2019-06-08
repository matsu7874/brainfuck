use brainfuck::{Interpreter, Lexer};
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() <= 1 {
        println!(
            "Brainfuck interpreter written in Rust.
Usage:
    brainfuck source.bf"
        );
        return process::exit(64);
    }

    let mut p = String::new();
    let file_name = &args[1];
    let mut f = File::open(file_name).expect("file not found");
    f.read_to_string(&mut p)
        .expect("something went wrong reading the file");

    let mut interpreter = Interpreter::new();
    let program = Lexer::lex(&p);
    if let Err(e) = interpreter.eval(&program) {
        println!("Error: {:?}", e);
    }
}
