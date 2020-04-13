use crate::bus::Bus;
use crate::cpu6507::CPU6507;

#[derive(Debug)]
pub enum AddressingMode {
    None,
    Immediate,
    Absolute,
    Implied,
    Accumulator,
    AbsoluteX,
    AbsoluteY,
    ZeroPageIndexed,
    ZeroPageX,
    ZeroPageY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
    Relative,
}

fn pages_differ(addr_a: u16, addr_b: u16) -> bool {
    (addr_a & 0xff00) != (addr_b & 0xff00)
}

impl AddressingMode {
    pub fn n_bytes(&self) -> usize {
        match *self {
              AddressingMode::Implied
            | AddressingMode::Accumulator => 1,

              AddressingMode::Immediate
            | AddressingMode::ZeroPageIndexed
            | AddressingMode::Relative
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndexedIndirect
            | AddressingMode::IndirectIndexed => 2,

              AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 3,

            _ => panic!("Bad addressing mode {:?}", *self),
        }
    }

    pub fn get_bytes(&self, cpu: &mut CPU6507) -> Vec<u8> {
        let n_bytes = self.n_bytes() as u16;
        (0 .. n_bytes).map(|n| cpu.read(cpu.pc + n)).collect::<Vec<_>>()
    }

    pub fn get_data(&self, cpu: &mut CPU6507) -> (u16, bool) {
        // At this point, cpu.pc points to the next instruction.
        let pc = cpu.pc - self.n_bytes() as u16;

        match *self {
            AddressingMode::Immediate => {
                let addr = pc + 1;
                (addr, false)
            },
            AddressingMode::Absolute => {
                let lo = cpu.read(pc + 1) as u16;
                let hi = cpu.read(pc + 2) as u16;
                let addr = (hi << 8) | lo;
                (addr, false)
            },
            AddressingMode::Implied => (0, false),
            AddressingMode::Accumulator => (0, false),
            AddressingMode::ZeroPageIndexed => {
                let addr = cpu.read(pc + 1) as u16;
                (addr, false)
            },
            AddressingMode::Relative => {
                let offset = cpu.read(pc + 1) as u16;

                // NOTE This has to be based off the current program counter,
                // _after_ it has been advanced, but before the instruction is
                // being executed. I don't know why though?

                // All of this casting is to handle negative offsets
                (((cpu.pc as i16) + (offset as i8 as i16)) as u16, false)
            },
            AddressingMode::AbsoluteX => {
                let lo = cpu.read(pc + 1) as u16;
                let hi = cpu.read(pc + 2) as u16;
                let addr = (hi << 8) | lo;
                let n_addr = addr.wrapping_add(cpu.x as u16);
                (n_addr, pages_differ(addr, n_addr))
            },
            AddressingMode::AbsoluteY => {
                let lo = cpu.read(pc + 1) as u16;
                let hi = cpu.read(pc + 2) as u16;
                let addr = (hi << 8) | lo;
                let n_addr = addr.wrapping_add(cpu.y as u16);
                (n_addr, pages_differ(addr, n_addr))
            },
            AddressingMode::Indirect => {
                let lo = cpu.read(pc + 1) as u16;
                let hi = cpu.read(pc + 2) as u16;
                let addr = (hi << 8) | lo;

                let lo = cpu.read(addr) as u16;

                let hi =
                    if addr & 0xff == 0xff {
                        cpu.read(addr & 0xff00) as u16
                    } else {
                        cpu.read(addr + 1) as u16
                    };

                let addr = (hi << 8) | lo;

                (addr, false)
            }
            AddressingMode::ZeroPageX => {
                let addr = cpu.read(pc + 1)
                    .wrapping_add(cpu.x) as u16;
                (addr, false)
            },
            AddressingMode::ZeroPageY => {
                let addr = cpu.read(pc + 1)
                    .wrapping_add(cpu.y) as u16;
                (addr, false)
            },
            AddressingMode::IndexedIndirect => {
                let lo = cpu.read(pc + 1);
                let addr = lo.wrapping_add(cpu.x) as u16;

                let lo = cpu.read(addr) as u16;

                let hi =
                    if addr & 0xff == 0xff {
                        cpu.read(addr & 0xff00) as u16
                    } else {
                        cpu.read(addr + 1) as u16
                    };

                let addr = (hi << 8) | lo;
                (addr, false)
            },
            AddressingMode::IndirectIndexed => {
                let addr = cpu.read(pc + 1) as u16;

                let lo = cpu.read(addr) as u16;

                let hi =
                    if addr & 0xff == 0xff {
                        cpu.read(addr & 0xff00) as u16
                    } else {
                        cpu.read(addr + 1) as u16
                    };

                let addr = (hi << 8) | lo;
                let n_addr = addr.wrapping_add(cpu.y as u16);

                (n_addr, pages_differ(addr, n_addr))
            },

            _ => panic!("Bad addressing mode {:?}", *self)
        }
    }
}
