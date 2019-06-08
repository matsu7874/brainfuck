#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    line: usize,
    col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation<T> {
    value: T,
    location: Location,
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
type Token = Annotation<TokenKind>;
impl Token {
    fn increment_pointer(location: Location) -> Self {
        Self::new(TokenKind::IncrementPointer, location)
    }
    fn decrement_pointer(location: Location) -> Self {
        Self::new(TokenKind::DecrementPointer, location)
    }
    fn increment(location: Location) -> Self {
        Self::new(TokenKind::Increment, location)
    }
    fn decrement(location: Location) -> Self {
        Self::new(TokenKind::Decrement, location)
    }
    fn output(location: Location) -> Self {
        Self::new(TokenKind::Output, location)
    }
    fn input(location: Location) -> Self {
        Self::new(TokenKind::Input, location)
    }
    fn jump_forward(location: Location) -> Self {
        Self::new(TokenKind::JumpForward, location)
    }
    fn jump_backward(location: Location) -> Self {
        Self::new(TokenKind::JumpBackward, location)
    }
}

type Program = Vec<Token>;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpreterErrorKind {
    UnmatchedJumpForwardError,
    UnmatchedJumpBackwardError,
    PointerError,
}
type InterpreterError = Annotation<InterpreterErrorKind>;

pub struct Interpreter {
    pointer: usize,
    program_cursor: usize,
    cells: Vec<u8>,
    program: Program,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            pointer: 0,
            program_cursor: 0,
            cells: vec![0 as u8],
            program: Vec::new(),
        }
    }
    fn eval_increment_pointer(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        self.pointer += 1;
        if self.cells.len() <= self.pointer {
            self.cells.push(0);
        }
        self.program_cursor += 1;
        Ok(self.pointer)
    }
    fn eval_decrement_pointer(&mut self, command: &Token) -> Result<usize, InterpreterError> {
        if self.pointer == 0 {
            return Err(InterpreterError {
                value: InterpreterErrorKind::PointerError,
                location: command.location,
            });
        }
        self.pointer -= 1;
        self.program_cursor += 1;
        Ok(self.pointer)
    }
    fn eval_increment(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        self.cells[self.pointer] = self.cells[self.pointer].wrapping_add(1);
        self.program_cursor += 1;
        Ok(self.cells[self.pointer] as usize)
    }

    fn eval_decrement(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        self.cells[self.pointer] = self.cells[self.pointer].wrapping_sub(1);
        self.program_cursor += 1;
        Ok(self.cells[self.pointer] as usize)
    }

    fn eval_output(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        print!("{}", self.cells[self.pointer] as char);
        self.program_cursor += 1;
        Ok(self.cells[self.pointer] as usize)
    }
    fn eval_input(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("read_line error");
        let value = buf.as_bytes()[0];
        self.cells[self.pointer] = value;
        self.program_cursor += 1;
        Ok(self.cells[self.pointer] as usize)
    }
    fn eval_jump_forward(&mut self, command: &Token) -> Result<usize, InterpreterError> {
        if self.cells[self.pointer] != 0 {
            self.program_cursor += 1;
            return Ok(self.cells[self.pointer] as usize);
        }
        let mut dep = 1;
        let mut pair_backward_pos = self.program_cursor;
        for i in (self.program_cursor + 1)..self.program.len() {
            match self.program[i].value {
                TokenKind::JumpForward => dep += 1,
                TokenKind::JumpBackward => {
                    if dep == 1 {
                        pair_backward_pos = i;
                        break;
                    } else {
                        dep -= 1;
                    }
                }
                _ => {}
            };
        }
        if self.program_cursor == pair_backward_pos {
            return Err(InterpreterError {
                value: InterpreterErrorKind::UnmatchedJumpBackwardError,
                location: command.location,
            });
        }
        self.program_cursor = pair_backward_pos;
        Ok(self.cells[self.pointer] as usize)
    }
    fn eval_jump_backward(&mut self, command: &Token) -> Result<usize, InterpreterError> {
        if self.cells[self.pointer] == 0 {
            self.program_cursor += 1;
            return Ok(self.cells[self.pointer] as usize);
        }
        let mut dep = 1;
        let mut pair_forward_pos = self.program_cursor;
        for i in (0..self.program_cursor).rev() {
            match self.program[i].value {
                TokenKind::JumpBackward => dep += 1,
                TokenKind::JumpForward => {
                    if dep == 1 {
                        pair_forward_pos = i;
                        break;
                    } else {
                        dep -= 1;
                    }
                }
                _ => {}
            };
        }
        if self.program_cursor == pair_forward_pos {
            return Err(InterpreterError {
                value: InterpreterErrorKind::UnmatchedJumpForwardError,
                location: command.location,
            });
        }
        self.program_cursor = pair_forward_pos;
        Ok(self.cells[self.pointer] as usize)
    }
    fn init(&mut self) {
        self.cells = vec![0 as u8];
    }
    pub fn eval(&mut self, program: &Program) -> Result<usize, InterpreterError> {
        self.program = (*program).clone();
        self.init();
        while self.program_cursor < self.program.len() {
            let command = &self.program[self.program_cursor].clone();
            let res = match command.value {
                TokenKind::IncrementPointer => self.eval_increment_pointer(command),
                TokenKind::DecrementPointer => self.eval_decrement_pointer(command),
                TokenKind::Increment => self.eval_increment(command),
                TokenKind::Decrement => self.eval_decrement(command),
                TokenKind::Output => self.eval_output(command),
                TokenKind::Input => self.eval_input(command),
                TokenKind::JumpForward => self.eval_jump_forward(command),
                TokenKind::JumpBackward => self.eval_jump_backward(command),
            };
            if res.is_err() {
                return Err(res.err().unwrap());
            }
        }
        Ok(0)
    }
}

#[test]
fn test_interpreter() {
    let mut interpreter = Interpreter::new();
    let program = vec![
        Token::increment(Location { line: 1, col: 1 }),
        Token::increment(Location { line: 1, col: 2 }),
        Token::increment_pointer(Location { line: 1, col: 3 }),
        Token::increment(Location { line: 1, col: 4 }),
        Token::increment(Location { line: 1, col: 5 }),
        Token::decrement(Location { line: 2, col: 1 }),
        Token::increment_pointer(Location { line: 2, col: 2 }),
        Token::decrement_pointer(Location { line: 2, col: 3 }),
    ];

    assert_eq!(interpreter.cells, vec![0]);
    assert_eq!(interpreter.eval(&program), Ok(0));
    assert_eq!(interpreter.cells, vec![2, 1, 0]);
    assert_eq!(interpreter.pointer, 1);
}
