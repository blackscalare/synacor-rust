use std::fs::File;
use std::io::Read;
use std::io::stdin;
use std::collections::VecDeque;
use std::collections::HashMap;

const RUNNING:i32 = 100;
const HALTED:i32 = 101;

struct SaveState {
    cpu: CPU,
    cursor: usize
}

#[derive(Clone)]
pub struct CPU {
    stack:          Vec<u16>,
    registers:      [u16; 8],
    memory:         Vec<u16>,
    state:          i32,
    input_queue:    VecDeque<u16>,
    opcode_map:     HashMap<u16, String>
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            stack: Vec::new(),
            registers: [0; 8],
            memory: Vec::new(),
            state: RUNNING,
            input_queue: VecDeque::new(),
            opcode_map: HashMap::new()
        }
    }

    pub fn read_binary(&mut self, filename: &String){
        let mut binary: Vec<u16> = Vec::new();
        self.opcode_map = HashMap::from([
            (0, String::from("HALT")), // 0
            (21, String::from("NOOP")), // 0,
            (2, String::from("PUSH")), // 1
            (3, String::from("POP")), // 1
            (6, String::from("JMP")), // 1
            (20, String::from("IN")), // 1
            (19, String::from("OUT")), // 1
            (17, String::from("CALL")), // 1
            (7, String::from("JT")), // 2
            (8, String::from("JF")), // 2
            (15, String::from("RMEM")), // 2
            (14, String::from("NOT")), // 2
            (16, String::from("WMEM")), // 2
            (1, String::from("SET")), // 2
            (9, String::from("ADD")), // 3
            (5, String::from("GT")), // 3
            (4, String::from("EQ")), // 3
            (10, String::from("MULT")), // 3
            (11, String::from("MOD")), // 3
            (12, String::from("AND")), // 3
            (13, String::from("OR")), // 3
            (18, String::from("RET")) // 1 from stack
        ]);
        let mut f =  File::open(&filename).expect("No file found");
        let metadata = f.metadata().expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("Buffer overflow");
        
        for x in (1..metadata.len() as usize).step_by(2) {
            let low: u8 = buffer[x - 1];
            let high: u8 = buffer[x];
            //let byte: u16 = ((high << 8) | (low & 0xff)) as u16;
            let byte: u16 = u16::from(high) << 8 | u16::from(low & 0xff);
            binary.push(byte);
        }

        self.memory = binary;
    }

    pub fn run(&mut self) {
        let mut cursor: usize = 0;
        let mut save_state: SaveState = SaveState {
            cpu: self.clone(),
            cursor: 0
        };
        let mut debugging: bool = false;
        let mut breakpoint: u16 = 0;
        let mut stepping: bool = true;
        while self.state == RUNNING {
            let opcode: u16 = self.memory[cursor];

            if debugging {
                print!("\x1B[2J\x1B[1;1H");
                println!("REGISTRY");
                for i in 0..8 {
                    let _r = self.registers[i];
                    println!("R{}: {}", i, _r,);
                }
                println!("STACK");
                let s_len = self.stack.len();
                let mut start = self.stack.len() as i32 - 5;
                if start < 0 {
                    start = 0;
                }
                for i in start..s_len as i32 {
                    let _s = self.stack[i as usize];
                    println!("{}", _s);
                }
                println!();

                for i in -3..7 {
                    if (cursor as i32) + i < 0 {
                        continue;
                    }
                    let mut p_str = String::new();
                    let cur = (cursor as i32 + i) as usize;
                    let _opcode = self.memory[cur];
                    let mut a = 0;
                    let mut b = 0;
                    let mut c = 0;
                    match _opcode {
                        0 => {// HALT
                            p_str = format!("{}     HALT", cur);
                        },
                        1 => {// SET
                            a = self.memory[cur + 1];
                            b = self.memory[cur + 2];
                            p_str = format!("{}    SET {} to {}", cur, a, b);
                        },
                        2 => {// PUSH
                            a = self.memory[cur + 1];
                            p_str = format!("{}    PUSH {}", cur, a);
                        },
                        3 => {// POP
                            a = self.memory[cur + 1];
                            p_str = format!("{}   POP {}", cur, a);
                        },
                        4 => {// EQ
                            a = self.memory[cur + 1];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   EQ ({} = {}) {}", cur, b, c, a);
                        },
                        5 => {// GT
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   GT ({} > {}) {}", cur, b, c, a);
                        },
                        6 => {// JMP
                            a = self.memory[(cur + 1)];
                            p_str = format!("{}   JMP {}", cur, a);
                        },
                        7 => {// JT
                            a = self.memory[cur + 1];
                            b = self.memory[(cur + 2)];
                            p_str = format!("{}   JT {} JMP to {}", cur, a, b);
                        },
                        8 => {// JF
                            a = self.memory[cur + 1];
                            b = self.memory[cur + 2];
                            p_str = format!("{}   JF {} JMP to {}", cur, a, b);
                        },
                        9 => {// ADD
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   ADD {} {} to {}", cur, b, c, a);
                        },
                        10 => {// MULT
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   MULT {} {} to {}", cur, b, c, a);
                        },
                        11 => {// MOD
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   MOD {} {} to {}", cur, b, c, a);
                        },
                        12 => {//AND
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   AND {} {} to {}", cur, b, c, a);
                        },
                        13 => {//OR
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            c = self.memory[(cur + 3)];
                            p_str = format!("{}   OR {} {} to {}", cur, b, c, a);
                        },
                        14 => {//NOT
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            p_str = format!("{}   NOT {} {} to {}", cur, b, c, a);
                        },
                        15 => {//RMEM
                            a = self.memory[cursor + 1];
                            b = self.memory[cursor + 2];
                            p_str = format!("{}   RMEM at {} to {}", cur, b, a);
                        },
                        16 => {//WMEM
                            a = self.memory[(cur + 1)];
                            b = self.memory[(cur + 2)];
                            p_str = format!("{}   WMEM {} to {}", cur, b, a);
                        }
                        17 => {//CALL
                            p_str = format!("{}   CALL {}", cur, cur + 2);
                        },
                        18 => {//RET
                            p_str = format!("{}   RET", cur);
                        }
                        19 => {// OUT
                            p_str = format!("{}   OUT", cur);
                        },
                        20 => {// IN
                            p_str = format!("{}   IN", cur);
                        },
                        21 => { // NOOP
                            p_str = format!("{}   NOOP", cur);
                        },
                        _ => {
                            p_str = format!("{}   {}", cur, self.memory[cur]);
                        }
                    }
                    if cur == cursor {
                        p_str.insert(0, '[');
                        p_str += "]";
                        //println!("[{}    {}]:", cur, self.opcode_map[&opcode]);
                    } /*else {
                        println!("{}    {}:", cur, self.opcode_map[&opcode]);
                    }*/
                    println!("{}", p_str);
                }
                println!();
                println!("s: step   b {}: breakpoint   c: continue", breakpoint);
                if breakpoint == cursor as u16 {
                    stepping = true;
                }
                if stepping {
                    let mut buffer = String::new();
                    let result = stdin().read_line(&mut buffer);
                    if buffer.trim()[..1] == String::from("b") {
                        breakpoint = buffer.trim()[2..buffer.trim().len()].parse::<u16>().unwrap();
                        stepping = false;
                    } else if buffer.trim() == "c" {
                        stepping = false;
                    }
                }
            }
            match opcode {
                0 => {// HALT
                    self.state = HALTED;
                },
                1 => {// SET
                    let b = self.read_value(cursor + 2);
                    self.set_register(cursor + 1, b);
                    cursor += 3;
                },
                2 => {// PUSH
                    let a = self.read_value(cursor + 1);
                    self.stack.push(a);
                    cursor += 2;
                },
                3 => {// POP
                    if self.stack.len() == 0 {
                        self.state = HALTED;
                    }

                    let top = self.stack.pop().unwrap();
                    self.set_register(cursor + 1, top);
                    cursor += 2;
                },
                4 => {// EQ
                    let b = self.read_value(cursor + 2);
                    let c = self.read_value(cursor + 3);
                    if b == c {
                        self.set_register(cursor + 1, 1);
                    } else {
                        self.set_register(cursor + 1, 0)
                    }
                    cursor += 4;
                },
                5 => {// GT
                    let b = self.read_value(cursor + 2);
                    let c = self.read_value(cursor + 3);
                    if b > c {
                        self.set_register(cursor + 1, 1);
                    } else {
                        self.set_register(cursor + 1, 0)
                    }
                    cursor += 4;
                },
                6 => {// JMP
                    cursor = self.read_value(cursor + 1) as usize;
                },
                7 => {// JT
                    if self.read_value(cursor + 1) != 0 {
                        cursor = self.read_value(cursor + 2) as usize;
                    }
                    else {
                        cursor += 3;
                    }
                },
                8 => {// JF
                    if self.read_value(cursor + 1) == 0 {
                        cursor = self.read_value(cursor + 2) as usize;
                    }
                    else {
                        cursor += 3;
                    }
                },
                9 => {// ADD
                    let b = self.read_value(cursor + 2) as i32;
                    let c = self.read_value(cursor + 3) as i32;
                    let sum = (b + c) % 32768;
                    self.set_register(cursor + 1, sum as u16);
                    cursor += 4;
                },
                10 => {// MULT
                    let b = self.read_value(cursor + 2) as i32;
                    let c = self.read_value(cursor + 3) as i32;
                    let sum = b * c;
                    self.set_register(cursor + 1, (sum % 32768) as u16);
                    cursor += 4;
                },
                11 => {// MOD
                    let b = self.read_value(cursor + 2) as i32;
                    let c = self.read_value(cursor + 3) as i32;
                    let sum = b % c;
                    self.set_register(cursor + 1, sum as u16);
                    cursor += 4;
                },
                12 => {//AND
                    let b = self.read_value(cursor + 2) as i32;
                    let c = self.read_value(cursor + 3) as i32;
                    self.set_register(cursor + 1, (b & c) as u16);
                    cursor += 4;
                },
                13 => {//OR
                    let b = self.read_value(cursor + 2) as i32;
                    let c = self.read_value(cursor + 3) as i32;
                    self.set_register(cursor + 1, (b | c) as u16);
                    cursor += 4;
                },
                14 => {//NOT
                    let b = self.read_value(cursor + 2) as i32;
                    self.set_register(cursor + 1, ((!b) & 0x7fff) as u16);
                    cursor += 3;
                },
                15 => {//RMEM
                    let b_addr = self.read_value(cursor + 2);
                    let b = self.memory[b_addr as usize];
                    self.set_register(cursor + 1, b);
                    cursor += 3;
                },
                16 => {//WMEM
                    let a = self.read_value(cursor + 1);
                    let b = self.read_value(cursor + 2);
                    self.memory[a as usize] = b;
                    cursor += 3;
                }
                17 => {//CALL
                    self.stack.push((cursor + 2) as u16);
                    cursor = self.read_value(cursor + 1) as usize;
                },
                18 => {//RET
                    cursor = self.stack.pop().unwrap() as usize;
                }
                19 => {// OUT
                    print!("{}", (self.read_value(cursor + 1) as u8) as char);
                    cursor += 2;
                },
                20 => {// IN
                    if self.input_queue.len() == 0 {
                        let mut buffer = String::new();
                        let result = stdin().read_line(&mut buffer);
                        
                        if buffer.trim() == "save" {
                            println!("saving state...");
                            save_state.cpu = self.clone();
                            save_state.cursor = cursor;
                            println!("saved state");
                        } else if buffer.trim() == "load" {
                            println!("loading state...");
                            *self = save_state.cpu.clone();
                            cursor = save_state.cursor;
                            println!("loaded state");
                        } else if buffer.trim() == "d" {
                            debugging = !debugging;
                        } else if buffer.trim().len() > 3 && buffer.trim()[..3] == String::from("set") {
                            let reg = buffer.trim()[4..5].parse::<usize>().unwrap();
                            let val = buffer.trim()[6..buffer.trim().len()].parse::<u16>().unwrap();
                            self.registers[reg] = val;
                            println!("set reg {} to {}", reg, val);
                        } else if buffer.trim() == "reg" {
                            println!("register");
                            for i in 0..8 {
                                println!("{}: {}", i, self.registers[i]);
                            }
                        } else if buffer.trim() == "q" {
                            std::process::exit(0);
                        } else if buffer.trim() == "s" {
                            stepping = true;
                        }

                        for c in buffer.chars() {
                            let _c = c as u16;
                            self.input_queue.push_back(_c);
                        }
                    } else {
                        let c = self.input_queue.pop_front().unwrap();
                        self.set_register(cursor + 1, c);
                        cursor += 2;
                    }
                },
                21 => { // NOOP
                    cursor += 1;
                },
                _ =>  {
                    println!("invalid operation {} at {}", self.memory[cursor], cursor);
                    self.state = HALTED;
                }
            }
        }
        println!("Program halted, now exiting");
    } 

    fn get_register(&mut self, cursor: usize) -> usize {
        return 32775 - self.memory[cursor] as usize;
    }

    fn set_register(&mut self, cursor: usize, value: u16) {
        let reg: usize = self.get_register(cursor);
        self.registers[reg] = value;
    }

    fn is_register(&mut self, cursor: usize) -> bool {
        return cursor >= 32768 && cursor <= 32775;
    }

    fn read_value(&mut self, cursor: usize) -> u16 {
        let value = self.memory[cursor] as usize;

        if self.is_register(value) {
            return self.read_register(cursor);
        }
        return value as u16;
    }

    fn read_register(&mut self, cursor: usize) -> u16 {
        let reg: usize = self.get_register(cursor);
        return self.registers[reg];
    }
}