use crate::lexer::{Annotation, Program, Token, TokenKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpreterErrorKind {
    UnmatchedJumpForwardError,
    UnmatchedJumpBackwardError,
    PointerError,
}
type InterpreterError = Annotation<InterpreterErrorKind>;

pub struct SimpleInterpreter {
    pointer: usize,
    program_cursor: usize,
    cells: Vec<u8>,
    program: Program,
    input_stream: Option<String>,
    jump_table: HashMap<usize, usize>,
}

impl SimpleInterpreter {
    pub fn new() -> Self {
        Self {
            pointer: 0,
            program_cursor: 0,
            cells: vec![0 as u8],
            program: Vec::new(),
            input_stream: None,
            jump_table: HashMap::new(),
        }
    }
    pub fn setInputStream(&mut self, input_stream: String) {
        self.input_stream = Some(input_stream);
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
    fn eval_jump_forward(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        if self.cells[self.pointer] != 0 {
            self.program_cursor += 1;
            return Ok(self.cells[self.pointer] as usize);
        }

        self.program_cursor = *self.jump_table.get(&self.program_cursor).unwrap();
        Ok(self.cells[self.pointer] as usize)
    }
    fn eval_jump_backward(&mut self, _command: &Token) -> Result<usize, InterpreterError> {
        if self.cells[self.pointer] == 0 {
            self.program_cursor += 1;
            return Ok(self.cells[self.pointer] as usize);
        }
        self.program_cursor = *self.jump_table.get(&self.program_cursor).unwrap();
        Ok(self.cells[self.pointer] as usize)
    }
    fn init(&mut self) -> Result<usize, InterpreterError> {
        self.cells = vec![0 as u8];
        let mut forward_brackets = vec![];
        let mut dep = 1;
        for i in 0..self.program.len() {
            match self.program[i].value {
                TokenKind::JumpForward => forward_brackets.push(i),
                TokenKind::JumpBackward => {
                    if forward_brackets.len() > 0 {
                        let forward = forward_brackets.pop().unwrap();
                        self.jump_table.insert(i, forward);
                        self.jump_table.insert(forward, i);
                    } else {
                        return Err(InterpreterError {
                            value: InterpreterErrorKind::UnmatchedJumpForwardError,
                            location: self.program[i].location,
                        });
                    }
                }
                _ => {}
            };
        }
        if forward_brackets.len() > 0 {
            return Err(InterpreterError {
                value: InterpreterErrorKind::UnmatchedJumpForwardError,
                location: self.program[forward_brackets[0]].location,
            });
        }
        Ok(0)
    }

    pub fn eval(&mut self, program: &Program) -> Result<usize, InterpreterError> {
        self.program = (*program).clone();
        if let Err(e) = self.init() {
            return Err(e);
        };
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
    use crate::lexer::{Location, Token};
    let mut interpreter = SimpleInterpreter::new();
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
