use std::fmt::{Display, Formatter};
use std::iter;
use crate::brainfuck::{BrainFuckInstruction, BrainFuckProgram};

#[derive(Clone, PartialEq)]
pub enum DesugaredBrainFuckInstruction {
    Add(usize),
    Sub(usize),
    Left(usize),
    Right(usize),
    Loop(Vec<DesugaredBrainFuckInstruction>),
    Zero,
    Set(u8),
    Input,
    Output,
}

impl DesugaredBrainFuckInstruction {
    pub fn resugar(&self) -> Vec<BrainFuckInstruction> {
        match self {
            DesugaredBrainFuckInstruction::Add(n) => iter::repeat(BrainFuckInstruction::Add).take(*n).collect(),
            DesugaredBrainFuckInstruction::Sub(n) => iter::repeat(BrainFuckInstruction::Sub).take(*n).collect(),
            DesugaredBrainFuckInstruction::Left(n) => iter::repeat(BrainFuckInstruction::Left).take(*n).collect(),
            DesugaredBrainFuckInstruction::Right(n) => iter::repeat(BrainFuckInstruction::Right).take(*n).collect(),
            DesugaredBrainFuckInstruction::Loop(v) => {
                let mut res = Vec::new();
                res.push(BrainFuckInstruction::LoopStart);
                res.extend(v.iter().flat_map(|i| i.resugar()));
                res.push(BrainFuckInstruction::LoopEnd);
                res
            }
            DesugaredBrainFuckInstruction::Zero => vec![BrainFuckInstruction::LoopStart, BrainFuckInstruction::Sub, BrainFuckInstruction::LoopEnd],
            DesugaredBrainFuckInstruction::Set(n) => BrainFuckInstruction::set(*n),
            DesugaredBrainFuckInstruction::Input => vec![BrainFuckInstruction::Input],
            DesugaredBrainFuckInstruction::Output => vec![BrainFuckInstruction::Output],
        }
    }
}

pub struct DesugaredBrainFuckProgram(Vec<DesugaredBrainFuckInstruction>);

impl DesugaredBrainFuckProgram {
    pub fn from_instructions(v: impl AsRef<[DesugaredBrainFuckInstruction]>) -> Self {
        Self(v.as_ref().to_vec())
    }

    pub fn as_slice(&self) -> &[DesugaredBrainFuckInstruction] {
        &self.0
    }

    pub fn resugar(&self) -> BrainFuckProgram {
        let mut res = Vec::new();
        for i in &self.0 {
            res.extend(i.resugar());
        }

        BrainFuckProgram::from_instructions(res)
    }
}

impl Display for DesugaredBrainFuckProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.resugar().fmt(f)
    }
}