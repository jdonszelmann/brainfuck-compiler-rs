use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::brainfuck::UnbalancedLoop::{OpenWithoutClose, TooManyClose};
use crate::desugared_brainfuck::{DesugaredBrainFuckInstruction, DesugaredBrainFuckProgram};

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum BrainFuckInstruction {
    Add,
    Sub,
    Left,
    Right,
    LoopStart,
    LoopEnd,
    Input,
    Output,
}

impl BrainFuckInstruction {
    pub fn set(n: u8) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(BrainFuckInstruction::LoopStart);
        res.push(BrainFuckInstruction::Sub);
        res.push(BrainFuckInstruction::LoopEnd);
        if n > 128 {
            res.push(BrainFuckInstruction::Sub);
            for _ in n..255 {
                res.push(BrainFuckInstruction::Sub);
            }
        } else {
            for _ in 0..n {
                res.push(BrainFuckInstruction::Add);
            }
        }

        res
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Add),
            '-' => Some(Self::Sub),
            '>' => Some(Self::Right),
            '<' => Some(Self::Left),
            '[' => Some(Self::LoopStart),
            ']' => Some(Self::LoopEnd),
            ',' => Some(Self::Input),
            '.' => Some(Self::Output),
            _ => None,
        }
    }
}

pub struct BrainFuckProgram(Vec<BrainFuckInstruction>);

pub enum UnbalancedLoop {
    TooManyClose,
    OpenWithoutClose,
}

impl BrainFuckProgram {
    pub fn from_instructions(v: impl AsRef<[BrainFuckInstruction]>) -> Self {
        Self(v.as_ref().to_vec())
    }

    fn desugar_iter<'a>(inp: impl Iterator<Item=&'a BrainFuckInstruction>) -> Result<Vec<DesugaredBrainFuckInstruction>, UnbalancedLoop> {
        let mut iter = inp.peekable();

        let mut res = Vec::new();
        while let Some(i) = iter.next() {
            match i {
                BrainFuckInstruction::Add => {
                    let mut num = 1;
                    while let Some(BrainFuckInstruction::Add) = iter.peek() {
                        let _ = iter.next();
                        num += 1;
                    }
                    res.push(DesugaredBrainFuckInstruction::Add(num))
                },
                BrainFuckInstruction::Sub => {
                    let mut num = 1;
                    while let Some(BrainFuckInstruction::Sub) = iter.peek() {
                        let _ = iter.next();
                        num += 1;
                    }
                    res.push(DesugaredBrainFuckInstruction::Sub(num))
                },
                BrainFuckInstruction::Left => {
                    let mut num = 1;
                    while let Some(BrainFuckInstruction::Left) = iter.peek() {
                        let _ = iter.next();
                        num += 1;
                    }
                    res.push(DesugaredBrainFuckInstruction::Left(num))
                },
                BrainFuckInstruction::Right => {
                    let mut num = 1;
                    while let Some(BrainFuckInstruction::Right) = iter.peek() {
                        let _ = iter.next();
                        num += 1;
                    }
                    res.push(DesugaredBrainFuckInstruction::Right(num))
                },
                BrainFuckInstruction::LoopStart => {
                    let mut loop_part = Vec::new();
                    let mut ctr = 0;
                    loop {
                        if let Some(nxt) = iter.next() {
                            if nxt == &BrainFuckInstruction::LoopEnd {
                                if ctr == 0 {
                                    break
                                } else {
                                    ctr -= 1;
                                    loop_part.push(nxt)
                                }
                            } else {
                                if nxt == &BrainFuckInstruction::LoopStart {
                                    ctr += 1;
                                }
                                loop_part.push(nxt)
                            }
                        } else {
                            return Err(OpenWithoutClose)
                        }
                    }

                    if loop_part == vec![&BrainFuckInstruction::Sub] || loop_part == vec![&BrainFuckInstruction::Add] {
                        if res.last() != Some(&DesugaredBrainFuckInstruction::Zero) {
                            res.push(DesugaredBrainFuckInstruction::Zero)
                        }
                    } else {
                        res.push(DesugaredBrainFuckInstruction::Loop(Self::desugar_iter(loop_part.into_iter())?))
                    }
                },
                BrainFuckInstruction::LoopEnd => {
                    return Err(TooManyClose);
                }
                BrainFuckInstruction::Input => res.push(DesugaredBrainFuckInstruction::Input),
                BrainFuckInstruction::Output => res.push(DesugaredBrainFuckInstruction::Output),
            }
        }

        Ok(res)
    }


    pub fn desugar(&self) -> Result<DesugaredBrainFuckProgram, UnbalancedLoop> {
        Ok(DesugaredBrainFuckProgram::from_instructions(Self::desugar_iter(self.0.iter())?))
    }
}

impl Display for BrainFuckProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in &self.0 {
            write!(f, "{}", match i {
                BrainFuckInstruction::Add => '+',
                BrainFuckInstruction::Sub => '-',
                BrainFuckInstruction::Left => '<',
                BrainFuckInstruction::Right => '>',
                BrainFuckInstruction::LoopStart => '[',
                BrainFuckInstruction::LoopEnd => ']',
                BrainFuckInstruction::Input => ',',
                BrainFuckInstruction::Output => '.',
            })?
        }
        writeln!(f)
    }
}

impl FromStr for BrainFuckProgram {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = Vec::new();
        for i in s.chars() {
            if let Some(i) = BrainFuckInstruction::from_char(i) {
                res.push(i);
            }
        }

        Ok(Self(res))
    }
}
