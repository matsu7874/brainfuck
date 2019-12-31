#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation<T> {
    pub value: T,
    pub location: Location,
}
impl<T> Annotation<T> {
    fn new(value: T, location: Location) -> Self {
        Self { value, location }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    IncrementPointer,
    DecrementPointer,
    Increment,
    Decrement,
    Output,
    Input,
    JumpForward,
    JumpBackward,
}
pub type Token = Annotation<TokenKind>;
impl Token {
    pub fn increment_pointer(location: Location) -> Self {
        Self::new(TokenKind::IncrementPointer, location)
    }
    pub fn decrement_pointer(location: Location) -> Self {
        Self::new(TokenKind::DecrementPointer, location)
    }
    pub fn increment(location: Location) -> Self {
        Self::new(TokenKind::Increment, location)
    }
    pub fn decrement(location: Location) -> Self {
        Self::new(TokenKind::Decrement, location)
    }
    pub fn output(location: Location) -> Self {
        Self::new(TokenKind::Output, location)
    }
    pub fn input(location: Location) -> Self {
        Self::new(TokenKind::Input, location)
    }
    pub fn jump_forward(location: Location) -> Self {
        Self::new(TokenKind::JumpForward, location)
    }
    pub fn jump_backward(location: Location) -> Self {
        Self::new(TokenKind::JumpBackward, location)
    }
}

pub type Program = Vec<Token>;

pub struct Lexer;
impl Lexer {
    pub fn lex(input: &str) -> Program {
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut col = 1;

        for c in input.as_bytes() {
            if let Some(command) = match c {
                b'>' => Some(Token::increment_pointer(Location { line, col })),
                b'<' => Some(Token::decrement_pointer(Location { line, col })),
                b'+' => Some(Token::increment(Location { line, col })),
                b'-' => Some(Token::decrement(Location { line, col })),
                b'.' => Some(Token::output(Location { line, col })),
                b',' => Some(Token::input(Location { line, col })),
                b'[' => Some(Token::jump_forward(Location { line, col })),
                b']' => Some(Token::jump_backward(Location { line, col })),
                b'\n' => {
                    line += 1;
                    col = 0;
                    None
                }
                _ => None,
            } {
                tokens.push(command);
            }
            col += 1;
        }
        tokens
    }
}

#[test]
fn test_lexer() {
    let p = "+[.+]\n>,-.<";
    assert_eq!(
        Lexer::lex(p),
        vec![
            Token::increment(Location { line: 1, col: 1 }),
            Token::jump_forward(Location { line: 1, col: 2 }),
            Token::output(Location { line: 1, col: 3 }),
            Token::increment(Location { line: 1, col: 4 }),
            Token::jump_backward(Location { line: 1, col: 5 }),
            Token::increment_pointer(Location { line: 2, col: 1 }),
            Token::input(Location { line: 2, col: 2 }),
            Token::decrement(Location { line: 2, col: 3 }),
            Token::output(Location { line: 2, col: 4 }),
            Token::decrement_pointer(Location { line: 2, col: 5 }),
        ]
    );
}
