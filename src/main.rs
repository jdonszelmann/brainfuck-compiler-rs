use std::convert::Infallible;
use std::io::{stdin, stdout};
use crate::brainfuck::BrainFuckProgram;
use crate::interpreter::BrainFuckInterpreter;

pub mod brainfuck;
pub mod desugared_brainfuck;
pub mod interpreter;
pub mod low_intermediate;
pub mod parser;

fn main() {
    let mut interpreter = BrainFuckInterpreter::new(stdout(), stdin());
    let Ok(program): Result<BrainFuckProgram, Infallible> = r#"

    "#.parse() else {
        unreachable!()
    };

    let Ok(desugared) = program.desugar() else {
        panic!("unbalanced parens")
    };

    interpreter.execute(desugared);
}
