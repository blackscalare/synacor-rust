mod cpu;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;


fn main() {
    let mut cpu = cpu::CPU::new();
    //disassemble(&String::from("/Users/adriansjohag/Documents/Synacor/challenge.bin"));
    cpu.read_binary(&String::from("/Users/adriansjohag/Documents/Synacor/challenge.bin"));
    cpu.run();
}
#[allow(dead_code)]
fn disassemble(filename: &String) {
    let mut binary: Vec<u16> = Vec::new();
    let mut f =  File::open(&filename).expect("No file found");
    let metadata = f.metadata().expect("Unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("Buffer overflow");
    
    for x in (1..metadata.len() as usize).step_by(2) {
        let low: u8 = buffer[x - 1];
        let high: u8 = buffer[x];
        let byte: u16 = u16::from(high) << 8 | u16::from(low & 0xff);
        binary.push(byte);
    }
    dump_binary(binary);
}

fn dump_binary(bin: Vec<u16>) {
    let opcode_map = HashMap::from([
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

    let reg_map  = HashMap::from([
        (32768, String::from("R0")),
        (32769, String::from("R1")),
        (32770, String::from("R2")),
        (32771, String::from("R3")),
        (32772, String::from("R4")),
        (32773, String::from("R5")),
        (32774, String::from("R6")),
        (32775, String::from("R7"))
    ]);
    let mut dumb_data = "".to_owned();
    let mut index = 0;
    for byte in bin {
        dumb_data.push_str(&index.to_string());
        dumb_data.push_str(" ");
        if reg_map.contains_key(&byte) {
            dumb_data.push_str(&reg_map[&byte]);
        } else if opcode_map.contains_key(&byte) {
            dumb_data.push_str(&opcode_map[&byte]);
        } else {
            dumb_data.push_str(&byte.to_string());
        }
        dumb_data.push_str("\n");
        index += 1;
    }
    std::fs::write("dump.txt", dumb_data).expect("Failed to dump file");
}