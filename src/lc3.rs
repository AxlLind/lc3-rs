use std::io::Write;
use std::fmt::Display;
use std::iter::once;
use console::Term;
use crate::key_queue::KeyQueue;

// opcodes
const ADD: u16 = 0b0001; const AND: u16 = 0b0101; const BR:  u16 = 0b0000;
const JMP: u16 = 0b1100; const JSR: u16 = 0b0100; const LD:  u16 = 0b0010;
const LDI: u16 = 0b1010; const LDR: u16 = 0b0110; const LEA: u16 = 0b1110;
const NOT: u16 = 0b1001; const ST:  u16 = 0b0011; const STI: u16 = 0b1011;
const STR: u16 = 0b0111; const TRP: u16 = 0b1111;
// trap codes
const OUT: u16 = 0x21; const PUTS:  u16 = 0x22; const GETC: u16 = 0x20;
const IN:  u16 = 0x23; const PUTSP: u16 = 0x24; const HALT: u16 = 0x25;
// mem-mapped io
const KBSR: u16 = 0xFE00; const DSR: u16 = 0xFE04; const MCR: u16 = 0xFFFE;
const KBDR: u16 = 0xFE02; const DDR: u16 = 0xFE06;

#[inline(always)]
fn sext(w: u16, b: u8) -> u16 {
  let m = 1 << (b - 1);
  let x = w & ((1 << b) - 1);
  (x ^ m) - m
}

fn to_char(v: u16) -> char { (v & 0x7f) as u8 as char }

pub struct LC3 {
  pc: u16,
  reg: [u16;8],
  regcc: u16,
  key_queue: KeyQueue,
  term: Term,
  mem: [u16;0x10000],
}

impl LC3 {
  pub fn new(code: &[u16], pc: u16) -> Self {
    let (start, end) = (pc as usize, pc as usize + code.len());
    let mut mem = [0;0x10000];
    mem[start..end].clone_from_slice(code);
    mem[DSR as usize] = 1 << 15;
    mem[MCR as usize] = 1 << 15;
    Self {
      pc,
      reg: [0;8],
      regcc: 0,
      key_queue: KeyQueue::spawn(),
      term: Term::buffered_stdout(),
      mem,
    }
  }

  pub fn execute(&mut self) {
    loop {
      let w = self.mem[self.pc as usize];
      let r  = (w as usize >> 9) & 0x7;
      let r2 = (w as usize >> 6) & 0x7;
      self.pc += 1;
      match w >> 12 {
        ADD => self.set_cc(r, self.reg[r2] + self.add_and_arg(w)),
        AND => self.set_cc(r, self.reg[r2] & self.add_and_arg(w)),
        NOT => self.set_cc(r, !self.reg[r2]),
        LD  => self.set_cc(r, self.rmem(self.pc + sext(w,9))),
        LDR => self.set_cc(r, self.rmem(self.reg[r2] + sext(w,6))),
        LDI => self.set_cc(r, self.rmem(self.rmem(self.pc + sext(w,9)))),
        LEA => self.set_cc(r, self.pc + sext(w,9)),
        ST  => self.wmem(self.reg[r], self.pc + sext(w,9)),
        STR => self.wmem(self.reg[r], self.reg[r2] + sext(w,6)),
        STI => self.wmem(self.reg[r], self.rmem(self.pc + sext(w,9))),
        BR  => if w & self.regcc != 0 { self.pc += sext(w,9) },
        JMP => self.pc = self.reg[r2],
        JSR => self.jsr(w,r2),
        TRP => self.trap(w),
        _ => panic!("illegal opcode exception: {}", w >> 12),
      }
    }
  }

  fn jsr(&mut self, w: u16, r2: usize) {
    self.reg[7] = self.pc;
    if w & 0x800 == 0 {
      self.pc = self.reg[r2];
    } else {
      self.pc += sext(w,11);
    }
  }

  fn trap(&mut self, w: u16) {
    match w & 0xff {
      GETC => self.reg[0] = self.key_queue.pop_blocking() as u8 as u16,
      OUT  => self.write(to_char(self.reg[0])),
      IN   => {
        self.write("> ");
        let c = self.key_queue.pop_blocking();
        self.write(c);
        self.reg[0] = c as u8 as u16;
      }
      PUTS => {
        let adr = self.reg[0] as usize;
        let s = self.mem[adr..].iter()
          .map(|&m| to_char(m))
          .take_while(|&m| m != '\0')
          .collect::<String>();
        self.write(s);
      }
      PUTSP => {
        let adr = self.reg[0] as usize;
        let s = self.mem[adr..].iter()
          .flat_map(|&m| once(m).chain(once(m >> 8)))
          .map(|m| to_char(m))
          .take_while(|&m| m != '\0')
          .collect::<String>();
        self.write(s);
      }
      HALT => panic!("lc3 halted"),
      _ => panic!("illegal trap: {}", w & 0xff),
    }
  }

  fn add_and_arg(&self, w: u16) -> u16 {
    if w & 0x20 != 0 { return sext(w,5); }
    self.reg[w as usize & 0x7]
  }

  fn rmem(&self, adr: u16) -> u16 {
    match adr {
      KBSR => (!self.key_queue.is_empty() as u16) << 15,
      KBDR => self.key_queue.pop_blocking() as u8 as u16,
      _    => self.mem[adr as usize],
    }
  }

  fn wmem(&mut self, v: u16, adr: u16) {
    match adr {
      DSR => return,
      DDR => self.write(to_char(v)),
      MCR => if v >> 15 == 0 { self.trap(HALT) },
      _   => {},
    }
    self.mem[adr as usize] = v;
  }

  fn set_cc(&mut self, r: usize, v: u16) {
    let sign = 10 - (v as i16).signum();
    self.regcc = 1 << sign;
    self.reg[r] = v;
  }

  fn write<T: Display>(&mut self, t: T) {
    // Ridiculous hack until this issue is fixed:
    // https://github.com/mitsuhiko/console/issues/36
    let s = format!("{}", t);
    let mut lines = s.split("\n");
    write!(self.term, "{}", lines.next().unwrap()).unwrap();
    for line in lines {
      self.term.clear_line().unwrap();
      writeln!(self.term, "{}", line).unwrap();
    }
    self.term.flush().unwrap();
  }
}
