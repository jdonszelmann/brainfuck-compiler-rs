use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::str::FromStr;
use crate::desugared_brainfuck::{DesugaredBrainFuckInstruction, DesugaredBrainFuckProgram};
use crate::parser::Parser;

pub type Variable = usize;

pub enum LowLevelIntermediateExpr {
    Const(Variable, u8),
    Copy {
        dest: Variable,
        src: Variable,
    },
    AddAssign { dest: Variable, modifier: Variable },
    SubAssign { dest: Variable, modifier: Variable },
    Print(Variable),
    Input(Variable),
    WhileNotZero(Variable, Vec<LowLevelIntermediateExpr>),
}

pub struct LowLevelIntermediateProgram {
    program: Vec<LowLevelIntermediateExpr>,
}

struct CompileState {
    data_ptr: usize,
    used: HashSet<Variable>,
    free_temps: HashSet<Variable>,
    smallest_unused: usize,
}

impl CompileState {
    pub fn move_to(&mut self, to: Variable) -> DesugaredBrainFuckInstruction {
        if to > self.data_ptr {
            let lefts = to - self.data_ptr;
            self.data_ptr = to;
            DesugaredBrainFuckInstruction::Left(lefts)
        } else {
            let rights = self.data_ptr - to;
            self.data_ptr = to;
            DesugaredBrainFuckInstruction::Right(rights)
        }
    }

    pub fn mark_used(&mut self, variable: Variable) {
        if variable >= self.smallest_unused {
            self.smallest_unused = variable + 1;
        }

        self.used.insert(variable);
    }

    pub fn used(&self, variable: &Variable) -> bool {
        self.used.contains(variable)
    }

    pub fn allocate_temp(&mut self) -> Variable {
        if let Some(i) = self.free_temps.drain().next() {
            return i;
        }

        assert!(!self.used(&self.smallest_unused));
        let smallest_unused = self.smallest_unused;
        self.mark_used(smallest_unused);
        smallest_unused
    }

    pub fn free_temp(&mut self, var: Variable) {
        self.free_temps.insert(var);
    }

    pub fn create_loop(&mut self, f: impl FnOnce(&mut Self) -> Vec<DesugaredBrainFuckInstruction>) -> DesugaredBrainFuckInstruction {
        let start = self.data_ptr;
        let instrs = f(self);
        // make sure the loop doesn't actually leave the data pointer anywhere it wasn't before
        assert_eq!(start, self.data_ptr);

        DesugaredBrainFuckInstruction::Loop(instrs)
    }
}

impl LowLevelIntermediateProgram {
    fn compile_iter<'a>(program: impl Iterator<Item=&'a LowLevelIntermediateExpr>, state: &mut CompileState) -> Vec<DesugaredBrainFuckInstruction> {
        let mut res = Vec::new();

        for expr in program {
            match expr {
                LowLevelIntermediateExpr::Const(var, val) => {
                    assert!(state.used(var));
                    res.push(state.move_to(*var));
                    res.push(DesugaredBrainFuckInstruction::Set(*val));
                }
                LowLevelIntermediateExpr::AddAssign { dest: source, modifier } => {
                    assert!(state.used(source));
                    assert!(state.used(modifier));

                    // x' = x + y
                    // temp0[-]
                    // y[x+temp0+y-]
                    // temp0[y+temp0-]
                    let temp0 = state.allocate_temp();

                    res.push(state.move_to(temp0));
                    res.push(DesugaredBrainFuckInstruction::Zero);
                    res.push(state.move_to(*modifier));

                    res.push(state.create_loop(|state| vec![
                        state.move_to(*source),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(*modifier),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    res.push(state.move_to(temp0));
                    res.push(state.create_loop(|state| vec![
                        state.move_to(*modifier),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    state.free_temp(temp0);
                }
                LowLevelIntermediateExpr::SubAssign { dest: source, modifier } => {
                    assert!(state.used(source));
                    assert!(state.used(modifier));

                    // x' = x + y
                    // temp0[-]
                    // y[x-temp0+y-]
                    // temp0[y+temp0-]
                    let temp0 = state.allocate_temp();

                    res.push(state.move_to(temp0));
                    res.push(DesugaredBrainFuckInstruction::Zero);
                    res.push(state.move_to(*modifier));

                    res.push(state.create_loop(|state| vec![
                        state.move_to(*source),
                        DesugaredBrainFuckInstruction::Sub(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(*modifier),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    res.push(state.move_to(temp0));
                    res.push(state.create_loop(|state| vec![
                        state.move_to(*modifier),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    state.free_temp(temp0);
                }
                LowLevelIntermediateExpr::Print(v) => {
                    res.push(state.move_to(*v));
                    res.push(DesugaredBrainFuckInstruction::Output);
                }
                LowLevelIntermediateExpr::Input(v) => {
                    res.push(state.move_to(*v));
                    res.push(DesugaredBrainFuckInstruction::Input);
                }
                LowLevelIntermediateExpr::WhileNotZero(v, code) => {
                    res.push(state.move_to(*v));
                    res.push(state.create_loop(|state| {
                        let mut inner = Vec::new();
                        inner.extend(Self::compile_iter(code.iter(), state));
                        inner.push(state.move_to(*v));
                        inner
                    }));
                }
                LowLevelIntermediateExpr::Copy { src, dest } => {
                    assert!(state.used(src));
                    assert!(state.used(dest));

                    // x = y
                    // temp0[-]
                    // x[-]
                    // y[x+temp0+y-]
                    // temp0[y+temp0-]
                    let temp0 = state.allocate_temp();

                    res.push(state.move_to(temp0));
                    res.push(DesugaredBrainFuckInstruction::Zero);
                    res.push(state.move_to(*dest));
                    res.push(DesugaredBrainFuckInstruction::Zero);

                    res.push(state.move_to(*src));
                    res.push(state.create_loop(|state| vec![
                        state.move_to(*dest),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(*src),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    res.push(state.move_to(temp0));
                    res.push(state.create_loop(|state| vec![
                        state.move_to(*src),
                        DesugaredBrainFuckInstruction::Add(1),
                        state.move_to(temp0),
                        DesugaredBrainFuckInstruction::Sub(1),
                    ]));

                    state.free_temp(temp0);
                }
            }
        }

        res
    }

    fn allocate_variables<'a>(program: impl Iterator<Item=&'a LowLevelIntermediateExpr>, state: &mut CompileState) {
        for i in program {
            match i {
                LowLevelIntermediateExpr::Const(v, _) => {
                    state.mark_used(*v);
                }
                LowLevelIntermediateExpr::Input(v) => {
                    state.mark_used(*v);
                }
                LowLevelIntermediateExpr::WhileNotZero(_, a) => {
                    Self::allocate_variables(a.iter(), state);
                }
                _ => {}
            }
        }
    }

    pub fn compile(&self) -> DesugaredBrainFuckProgram {
        let mut state = CompileState {
            data_ptr: 0,
            used: Default::default(),
            free_temps: Default::default(),
            smallest_unused: 0,
        };

        Self::allocate_variables(self.program.iter(), &mut state);
        DesugaredBrainFuckProgram::from_instructions(Self::compile_iter(
            self.program.iter(),
            &mut state,
        ))
    }

    pub fn parse_expr(s: &mut Parser, alloc: &mut VariableAllocator) -> LowLevelIntermediateExpr {
        if s.accept_str("print").is_some() {
            s.whitespace();
            let Some(name) = s.parse_ident() else {
                panic!("expected variable name after 'print'");
            };
            s.whitespace();
            assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
            s.whitespace();

            let var = alloc.variable(name);
            return LowLevelIntermediateExpr::Print(var)
        }

        if s.accept_str("input").is_some() {
            s.whitespace();
            let Some(name) = s.parse_ident() else {
                panic!("expected variable name after 'input'");
            };
            s.whitespace();
            assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
            s.whitespace();

            let var = alloc.variable(name);
            return LowLevelIntermediateExpr::Input(var)
        }

        if s.accept_str("while").is_some() {
            s.whitespace();
            let Some(name) = s.parse_ident() else {
                panic!("expected variable name after 'while' at {s}");
            };

            s.whitespace();
            if s.accept_str("!=").is_none() {
                panic!("expected '!=' name after 'while {}' at {s}", name);
            }
            s.whitespace();
            if s.accept_str("0").is_none() {
                panic!("expected '0' name after 'while {} !=' at {s}", name);
            }
            s.whitespace();

            if s.accept_str("{").is_none() {
                panic!("expected '{{' name after 'while {} != 0' at {s}", name);
            }
            s.whitespace();

            let mut res = Vec::new();
            loop {
                s.whitespace();
                if s.is_empty() {
                    panic!("expected '}}'");
                }
                if s.accept('}').is_some() {
                    break;
                }
                s.whitespace();

                res.push(Self::parse_expr(s, alloc));
                s.whitespace();
            }

            let var = alloc.variable(name);
            return LowLevelIntermediateExpr::WhileNotZero(var, res);
        }

        s.whitespace();
        let Some(dest) = s.parse_ident() else {
            panic!("expected variable name at {s}");
        };
        let dest = alloc.variable(dest);

        s.whitespace();
        if s.accept_str("+=").is_some() {
            s.whitespace();
            let Some(modifier) = s.parse_ident() else {
                panic!("expected variable name after '+=' at {s}");
            };
            s.whitespace();
            assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
            s.whitespace();

            let modifier = alloc.variable(modifier);
            return LowLevelIntermediateExpr::AddAssign {
                dest,
                modifier,
            }
        }
        if s.accept_str("-=").is_some() {
            s.whitespace();
            let Some(modifier) = s.parse_ident() else {
                panic!("expected variable name after '-='");
            };
            s.whitespace();
            assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
            s.whitespace();

            let modifier = alloc.variable(modifier);
            return LowLevelIntermediateExpr::SubAssign {
                dest,
                modifier,
            }
        }
        if s.accept_str("=").is_some() {
            s.whitespace();
            if let Some(value) = s.parse_num::<u8>() {
                s.whitespace();
                assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
                s.whitespace();
                return LowLevelIntermediateExpr::Const(dest, value)
            } else if let Some(i) = s.parse_ident() {
                s.whitespace();
                assert!(s.accept(';').is_some(), "expected semicolon at the end of the line at {s}");
                s.whitespace();

                let src = alloc.variable(i);
                return LowLevelIntermediateExpr::Copy {
                    dest,
                    src,
                }
            } else {
                panic!("expected numer (in 0..=255) or variable after '=' at {s}");
            }


        }

        panic!("expected '+=', '-=' or '=' after variable at {s}");
    }

    pub fn parse(s: &str) -> Self {
        let mut variable_allocator = VariableAllocator::new();

        let mut res = Vec::new();
        let mut stream = Parser::new(s);
        stream.whitespace();
        while !stream.is_empty() {
            stream.whitespace();
            res.push(Self::parse_expr(&mut stream, &mut variable_allocator));
            stream.whitespace();
        }
        Self {
            program: res,
        }
    }
}

pub struct VariableAllocator {
    vars: HashMap<String, usize>,
    max: usize,
}

impl VariableAllocator {
    pub fn new() -> Self {
        Self {
            vars: Default::default(),
            max: 0,
        }
    }

    pub fn variable(&mut self, v: String) -> usize {
        if let Some(i) = self.vars.get(&v) {
            *i
        } else {
            let i = self.max;
            self.max += 1;
            self.vars.insert(v, i);

            i
        }
    }
}

impl FromStr for LowLevelIntermediateProgram {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, stdin};
    use crate::interpreter::BrainFuckInterpreter;
    use crate::low_intermediate::LowLevelIntermediateProgram;

    macro_rules! bf_test {
        ($name: ident: $inp: literal, $output: expr) => {
            #[test]
            fn $name() {
                let program = $inp;

                let mut output_buf = Cursor::new(Vec::new());
                let interm = LowLevelIntermediateProgram::parse(program);
                let bf = interm.compile();
                println!("code: {}", bf);
                {
                    let mut interpreter = BrainFuckInterpreter::new(&mut output_buf, stdin());
                    interpreter.execute(bf);
                }
                let output = output_buf.into_inner();
                // println!("bytes: {:?}", output);

                assert_eq!(&$output, output.as_slice())
            }
        };
    }

    bf_test!(
        constant:
        r#"a = 3; print a;"#,
        [3]
    );
    bf_test!(
        add:
        r#"a = 3; b = 4; a += b; print a;"#,
        [7]
    );
    bf_test!(
        sub_to_zero:
        r#"
a = 4;
one = 1;
while a != 0 {
    a -= one;
}
print a;
"#,
        [0]
    );
    bf_test!(
        copy:
        r#"
a = 4;
one = 1;
res = 0;
while a != 0 {
    a -= one;
    res += one;
}
print res;
"#,
        [4]
    );
}

