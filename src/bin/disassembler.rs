use std::io::Result;
use lc3_image::read_image;

// opcodes
const ADD: u16 = 0b0001; const AND: u16 = 0b0101; const BR:  u16 = 0b0000;
const JMP: u16 = 0b1100; const JSR: u16 = 0b0100; const LD:  u16 = 0b0010;
const LDI: u16 = 0b1010; const LDR: u16 = 0b0110; const LEA: u16 = 0b1110;
const NOT: u16 = 0b1001; const ST:  u16 = 0b0011; const STI: u16 = 0b1011;
const STR: u16 = 0b0111; const TRP: u16 = 0b1111;
// trap codes
const OUT: u16 = 0x21; const PUTS:  u16 = 0x22; const GETC: u16 = 0x20;
const IN:  u16 = 0x23; const PUTSP: u16 = 0x24; const HALT: u16 = 0x25;

const DEFAULT_PROGRAM: &str = "./programs/obj/2048.obj";

fn sext(w: u16, b: u8) -> u16 {
  let m = 1 << (b - 1);
  let x = w & ((1 << b) - 1);
  (x ^ m) - m
}

fn print_add_and(op: &str, w: u16) {
  let r  = (w >> 9) & 0x7;
  let r2 = (w >> 6) & 0x7;
  if w & 0x20 == 0 {
    println!("{} ${} ${} ${}", op, r, r2, w as usize & 0x7);
  } else {
    println!("{} ${} ${} {}", op, r, r2, sext(w,5));
  }
}


fn main() -> Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let default_program = DEFAULT_PROGRAM.to_string();
  let path = args.get(1).unwrap_or(&default_program);
  let (program, pc_start) = read_image(path)?;
  println!("origin {:#06x}", pc_start);
  for (i,&w) in program.iter().enumerate() {
    let pc = pc_start + i as u16 + 1;
    print!("{:#06x}: ", pc - 1);
    let r  = (w >> 9) & 0x7;
    let r2 = (w >> 6) & 0x7;
    match w >> 12 {
      ADD => print_add_and("add", w),
      AND => print_add_and("and", w),
      NOT => println!("not ${} ${}", r, r2),
      LD  => println!("ld  ${} {:#06x}", r, pc + sext(w,9)),
      LDR => println!("ldr ${} ${} {:#06x}", r, r2, sext(w,6)),
      LDI => println!("ldi ${} {:#06x}", r, pc + sext(w,9)),
      LEA => println!("lea ${} {:#06x}", r, pc + sext(w,9)),
      ST  => println!("st  ${} {:#06x}", r, pc + sext(w,9)),
      STR => println!("str ${} ${} {}", r, r2, sext(w,6)),
      STI => println!("sti ${} {:#06x}", r, pc + sext(w,9)),
      JMP => println!("jmp ${}", r2),
      BR  => {
        let n = if w & 0x800 != 0 {"n"} else {""};
        let z = if w & 0x400 != 0 {"z"} else {""};
        let p = if w & 0x200 != 0 {"p"} else {""};
        println!("br{}{}{} {:#06x}", n, z, p, pc + sext(w,9));
      }
      JSR => {
        if w & 0x800 == 0 {
          println!("jsr ${}", r2);
        } else {
          println!("jsr {:#06x}", pc + sext(w,11));
        }
      }
      TRP => match w & 0xff {
        GETC  => println!("getc"),
        OUT   => println!("out"),
        IN    => println!("in"),
        PUTS  => println!("puts"),
        PUTSP => println!("putsp"),
        HALT  => println!("halt"),
        _     => println!("illegal trap {}", w & 0xff),
      },
      _ => println!("illegal op {}", w >> 12),
    }
  }
  Ok(())
}
