use std::io::{BufRead, BufReader, BufWriter, Read, stdout, Write};
use crate::desugared_brainfuck::{DesugaredBrainFuckInstruction, DesugaredBrainFuckProgram};

const MEMORY_SIZE: usize = 30_000;

pub struct BrainFuckInterpreter<W: Write, R> {
    memory: [u8; MEMORY_SIZE],
    ptr: usize,
    output: BufWriter<W>,
    input: BufReader<R>,
    read_buf: std::vec::IntoIter<u8>,
}

impl<W: Write, R: Read> BrainFuckInterpreter<W, R> {
    pub fn new(output: W, input: R) -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            ptr: 0,
            output: BufWriter::new(output),
            input: BufReader::new(input),
            read_buf: vec![].into_iter(),
        }
    }

    fn execute_internal<'a>(&mut self, program: impl Iterator<Item=&'a DesugaredBrainFuckInstruction>) {
        for i in program {
            match i {
                DesugaredBrainFuckInstruction::Add(i) => self.memory[self.ptr] = (self.memory[self.ptr] as usize).wrapping_add(*i) as u8,
                DesugaredBrainFuckInstruction::Sub(i) => self.memory[self.ptr] = (self.memory[self.ptr] as usize).wrapping_sub(*i) as u8,
                DesugaredBrainFuckInstruction::Left(l) => self.ptr = (self.ptr as isize - *l as isize).rem_euclid(MEMORY_SIZE as isize) as usize,
                DesugaredBrainFuckInstruction::Right(r) => self.ptr = (self.ptr as isize + *r as isize).rem_euclid(MEMORY_SIZE as isize) as usize,
                DesugaredBrainFuckInstruction::Loop(l) => {
                    loop {
                        if self.memory[self.ptr] == 0 {
                            break;
                        }
                        self.execute_internal(l.iter());
                    }
                }
                DesugaredBrainFuckInstruction::Zero => {
                    self.memory[self.ptr] = 0;
                }
                DesugaredBrainFuckInstruction::Set(v) => {
                    self.memory[self.ptr] = *v;
                }
                DesugaredBrainFuckInstruction::Input => {
                    if let Some(i) = self.read_buf.next() {
                        self.memory[self.ptr] = i;
                    } else {
                        let mut buf = String::new();
                        match self.input.read_line(&mut buf) {
                            Ok(0) => {
                                self.read_buf = vec![0].into_iter();
                            }
                            Ok(_) => {
                                let mut res = Vec::new();
                                res.extend(buf.bytes());
                                res.push(10);
                                self.read_buf = res.into_iter();
                            }
                            Err(e) => panic!("{e}")
                        }
                    }
                }
                DesugaredBrainFuckInstruction::Output => {
                    let byte = self.memory[self.ptr];
                    if let Err(e) = self.output.write(&[byte]) {
                        panic!("{e}")
                    }
                }
            }
        }
    }

    pub fn execute(&mut self, program: DesugaredBrainFuckProgram) {
        self.execute_internal(program.as_slice().iter())
    }
}