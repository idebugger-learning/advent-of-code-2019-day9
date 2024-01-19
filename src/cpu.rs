enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl ParameterMode {
    fn parse(code: isize) -> Self {
        match code {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            2 => ParameterMode::Relative,
            x => panic!("Unknown parameter code {}", x),
        }
    }
}

enum Opcode {
    Add,
    Mul,
    Read,
    Write,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    SetRelBase,
    Halt,
}

impl Opcode {
    fn parse(code: isize) -> Self {
        match code {
            1 => Opcode::Add,
            2 => Opcode::Mul,
            3 => Opcode::Read,
            4 => Opcode::Write,
            5 => Opcode::JumpIfTrue,
            6 => Opcode::JumpIfFalse,
            7 => Opcode::LessThan,
            8 => Opcode::Equals,
            9 => Opcode::SetRelBase,
            99 => Opcode::Halt,
            x => panic!("Unknown opcode {}", x),
        }
    }
}

type Instruction = (Opcode, ParameterMode, ParameterMode, ParameterMode);

pub struct CPU {
    memory: Vec<isize>,
    ip: usize,
    halted: bool,
    waiting_for_stdin: bool,
    stdin: Vec<isize>,
    stdin_position: usize,
    stdout: Vec<isize>,
    relative_base: isize,
}

impl CPU {
    pub fn new(program: &Vec<isize>) -> Self {
        let mut memory = program.clone();
        memory.append(&mut vec![0; 10240]);

        CPU {
            memory,
            ip: 0,
            halted: false,
            waiting_for_stdin: false,
            stdin: vec![],
            stdin_position: 0,
            stdout: vec![],
            relative_base: 0,
        }
    }

    pub fn push_stdin(&mut self, input: isize) {
        self.waiting_for_stdin = false;
        self.stdin.push(input);
    }

    pub fn get_stdout(&self) -> &Vec<isize> {
        &self.stdout
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn run(&mut self) {
        while !self.halted && !self.waiting_for_stdin {
            self.step();
        }
    }

    fn parse_instruction(code: isize) -> Instruction {
        let opcode = code % 100;
        let opcode = Opcode::parse(opcode);

        let pmode_1 = (code / 100) % 10;
        let pmode_1 = ParameterMode::parse(pmode_1);

        let pmode_2 = (code / 1000) % 10;
        let pmode_2 = ParameterMode::parse(pmode_2);

        let pmode_3 = (code / 10000) % 10;
        let pmode_3 = ParameterMode::parse(pmode_3);

        (opcode, pmode_1, pmode_2, pmode_3)
    }

    fn get_operand_addr(&self, addr: usize, mode: ParameterMode) -> usize {
        match mode {
            ParameterMode::Immediate => addr,
            ParameterMode::Position => self.memory[addr] as usize,
            ParameterMode::Relative => (self.relative_base + self.memory[addr]) as usize,
        }
    }

    fn step(&mut self) {
        let (opcode, pmode_1, pmode_2, pmode_3) = Self::parse_instruction(self.memory[self.ip]);
        match opcode {
            Opcode::Add => self.opcode_add((pmode_1, pmode_2, pmode_3)),
            Opcode::Mul => self.opcode_mul((pmode_1, pmode_2, pmode_3)),
            Opcode::JumpIfTrue => self.opcode_jump_if_true((pmode_1, pmode_2)),
            Opcode::JumpIfFalse => self.opcode_jump_if_false((pmode_1, pmode_2)),
            Opcode::LessThan => self.opcode_less_than((pmode_1, pmode_2, pmode_3)),
            Opcode::Equals => self.opcode_equals((pmode_1, pmode_2, pmode_3)),
            Opcode::Read => self.opcode_read(pmode_1),
            Opcode::Write => self.opcode_write(pmode_1),
            Opcode::SetRelBase => self.opcode_set_rel_base(pmode_1),
            Opcode::Halt => self.step_halt(),
        }
    }

    fn opcode_add(&mut self, pmode: (ParameterMode, ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        let target_pos = self.get_operand_addr(self.ip + 3, pmode.2);
        self.memory[target_pos] = operand1 + operand2;

        self.ip += 4;
    }

    fn opcode_mul(&mut self, pmode: (ParameterMode, ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        let target_pos = self.get_operand_addr(self.ip + 3, pmode.2);
        self.memory[target_pos] = operand1 * operand2;

        self.ip += 4;
    }

    fn opcode_jump_if_true(&mut self, pmode: (ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        if operand1 != 0 {
            self.ip = operand2 as usize
        } else {
            self.ip += 3
        }
    }

    fn opcode_jump_if_false(&mut self, pmode: (ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        if operand1 == 0 {
            self.ip = operand2 as usize
        } else {
            self.ip += 3
        }
    }

    fn opcode_less_than(&mut self, pmode: (ParameterMode, ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        let target_pos = self.get_operand_addr(self.ip + 3, pmode.2);
        if operand1 < operand2 {
            self.memory[target_pos] = 1;
        } else {
            self.memory[target_pos] = 0;
        }

        self.ip += 4;
    }

    fn opcode_equals(&mut self, pmode: (ParameterMode, ParameterMode, ParameterMode)) {
        let operand1 = self.memory[self.get_operand_addr(self.ip + 1, pmode.0)];
        let operand2 = self.memory[self.get_operand_addr(self.ip + 2, pmode.1)];

        let target_pos = self.get_operand_addr(self.ip + 3, pmode.2);
        if operand1 == operand2 {
            self.memory[target_pos] = 1;
        } else {
            self.memory[target_pos] = 0;
        }

        self.ip += 4;
    }

    fn opcode_read(&mut self, pmode: ParameterMode) {
        let target_pos = self.get_operand_addr(self.ip + 1, pmode);

        if self.stdin.len() > self.stdin_position {
            let integer = self.stdin[self.stdin_position];
            self.stdin_position += 1;

            self.memory[target_pos] = integer;
            self.ip += 2;
        } else {
            self.waiting_for_stdin = true;
        }
    }

    fn opcode_write(&mut self, pmode: ParameterMode) {
        let operand = self.memory[self.get_operand_addr(self.ip + 1, pmode)];

        self.stdout.push(operand);

        self.ip += 2;
    }

    fn opcode_set_rel_base(&mut self, pmode: ParameterMode) {
        let operand = self.memory[self.get_operand_addr(self.ip + 1, pmode)];

        self.relative_base += operand;

        self.ip += 2;
    }

    fn step_halt(&mut self) {
        self.halted = true;
    }
}
