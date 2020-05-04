# lc3-rs
A `Little Computer 3` emulator in Rust, implementing (almost) the entire spec.

## The Little Computer 3 architecture (lc3)
The [Little Computer 3](https://en.wikipedia.org/wiki/Little_Computer_3) is a fictional cpu architecture ([ISA](https://en.wikipedia.org/wiki/Instruction_set_architecture)) used for educational purposes. The LC3 architecture was created by researchers at University of Texas at Austin and University of Illinois for use in teaching low level computer architecture courses. It features a relatively simple instruction set with 14 instructions, 6 traps, and 5 memory mapped registers for controlling the display and keyboard.

It is a popular architecture to write assembly programs for, since the ISA is relatively simple, but also to write virtual machines for such as this one. It is a step up in challenge compared to what I had done before (like [this](https://github.com/AxlLind/AdventOfCode2019/blob/master/src/intcoder.rs) or [this](https://github.com/AxlLind/synacor_challenge/blob/master/src/cpu.rs)) since it requires reading key-presses, condition registers, and a lot of bitwise operations.

The entire LC3 architecture is detailed in [this document](./spec.pdf). Note that like most implementations this does not implement the OS related functions like protection rings and the `RTI` privileged instruction.

## My implementation
The basic design (see [lc3.rs](./src/lc3.rs)) is very simple. A loop continuously executing instructions, based on the current program counter. The rest is just a matter of correctly implementing each instruction, being careful with doing correct bitwise operations.
```Rust
loop {
  let w = self.mem[self.pc as usize];
  let a = (w as usize >> 9) & 0x7;
  let b = (w as usize >> 6) & 0x7;
  self.pc += 1;
  match w >> 12 {
    NOT => self.cset(a, !self.reg[b]),
    ADD => self.cset(a, self.reg[b] + self.add_and_arg(w)),
    AND => self.cset(a, self.reg[b] & self.add_and_arg(w)),
    LD  => self.cset(a, self.rmem(self.pc + sext(w,9))),
    LDR => self.cset(a, self.rmem(self.reg[b] + sext(w,6))),
    LDI => self.cset(a, self.rmem(self.rmem(self.pc + sext(w,9)))),
    LEA => self.cset(a, self.pc + sext(w,9)),
    ST  => self.wmem(a, self.pc + sext(w,9)),
    STR => self.wmem(a, self.reg[b] + sext(w,6)),
    STI => self.wmem(a, self.rmem(self.pc + sext(w,9))),
    BR  => if w & self.regcc != 0 { self.pc += sext(w,9) },
    JMP => self.pc = self.reg[b],
    JSR => self.jsr(w,b),
    TRP => self.trap(w),
    _   => panic!("illegal opcode: {}", w >> 12),
  }
}
```
The complicated part of this architecture is dealing with key-presses. LC3 has a memory mapped register called `KBSR`. When you read from this memory address it should return `1 << 15` if the user pressed a key that the cpu has not dealt with and `0` otherwise. The `KBDR` address contains the pressed key. This means that to correctly simulate the LC3 cpu we cannot simply do a blocking read when reading `KBDR`. We need to know when there is a new pressed key available to read.

My solution to this problem was an asynchronous queue of pressed keys (see [KeyQueue here](./src/key_queue.rs)). On creation, this queue spawns a background thread which does blocking reads of key presses. When the user presses a key this thread inserts it into a queue. The LC3 can then simply check if this queue contains anything and pop from it accordingly. Some programs will not check the KBSR register before reading from KBDR however. To efficiently handle this, a condition variable is used to avoid unnecessary spinning in `pop_blocking` while waiting for a key to be pressed. If this is not added it will try to read from the queue over and over again as fast as possible, causing maximum cpu usage. This way it gets woken up as soon as there is a key available instead.

```Rust
match adr {
  KBSR => (!self.key_queue.is_empty() as u16) << 15,
  KBDR => self.key_queue.pop_blocking() as u8 as u16,
  _    => self.mem[adr as usize],
}
```
