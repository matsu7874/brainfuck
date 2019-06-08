use brainfuck::{Interpreter, Lexer};

fn main() {
    let mut interpreter = Interpreter::new();
    let p = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    let program = Lexer::lex(p);
    if let Err(e) = interpreter.eval(&program) {
        println!("Error: {:?}", e);
    }
}
