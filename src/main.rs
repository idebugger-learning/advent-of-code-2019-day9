use crate::cpu::CPU;

mod cpu;

fn main() {
    let input = include_str!("../data/input.txt");
    let program: Vec<_> = input
        .split(',')
        .map(|n| n.parse::<isize>().unwrap())
        .collect();

    let mut cpu = CPU::new(&program);
    cpu.push_stdin(1);
    cpu.run();

    println!("Output: {:?}", cpu.get_stdout());
}
