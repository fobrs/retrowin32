use bitflags::bitflags;
use tsify::Tsify;

bitflags! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Flags: u32 {
        /// carry
        const CF = 1 << 0;
        /// zero
        const ZF = 1 << 6;
        /// sign
        const SF = 1 << 7;
        /// direction
        const DF = 1 << 10;
        /// overflow
        const OF = 1 << 11;
    }
}

bitflags! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct FPUStatus: u16 {
        const C3 = 1 << 14;
        const C2 = 1 << 10;
        const C1 = 1 << 9;
        const C0 = 1 << 8;
    }
}

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Tsify)]
pub struct Registers {
    // Warning: get32 assumes the registers start with eax and are in a particular order.
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,

    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,

    pub eip: u32,

    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,
    // TODO: segment registers are actually 16-bit indexes into the GDT/LDT,
    // but for our purposes all we ever care about is making FS-relative accesses point
    // at the Windows TEB.
    /// Address that FS-relative accesses point to.
    pub fs_addr: u32,

    pub flags: Flags,

    /// FPU registers.
    pub st: [f64; 8],
    /// Top of FPU stack; 8 when stack empty.
    pub st_top: usize,
    /// FPU status word (TODO fold st_top in here?)
    pub fpu_status: FPUStatus,

    /// MMX registers.
    // TODO: officially these should alias the FPU registers(!).
    pub mm: [u64; 8],
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            eip: 0,
            cs: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            ss: 0,
            fs_addr: 0,
            flags: Flags::empty(),

            st: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            st_top: 8,
            fpu_status: FPUStatus::empty(),

            mm: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    pub fn get32(&self, reg: iced_x86::Register) -> u32 {
        // This function is hot in profiles, and even if we write
        // a match statement that maps register N to struct offset 4*N,
        // llvm doesn't seem to optimize it to the obvious math.
        // I tried the equivalent in C++ and that didn't optimize either.
        // So instead here's some unsafe hackery.
        let idx = reg as usize - iced_x86::Register::EAX as usize;
        if idx >= 8 {
            unreachable!("{reg:?}");
        }
        // XXX this assumes register order matches between our struct and iced_x86.
        unsafe { *(self as *const Registers as *const u32).add(idx) }
    }

    pub fn get16(&self, reg: iced_x86::Register) -> u16 {
        match reg {
            iced_x86::Register::AX => self.eax as u16,
            iced_x86::Register::CX => self.ecx as u16,
            iced_x86::Register::DX => self.edx as u16,
            iced_x86::Register::BX => self.ebx as u16,
            iced_x86::Register::SP => self.esp as u16,
            iced_x86::Register::BP => self.ebp as u16,
            iced_x86::Register::SI => self.esi as u16,
            iced_x86::Register::DI => self.edi as u16,
            iced_x86::Register::ES => self.es,
            iced_x86::Register::CS => self.cs,
            iced_x86::Register::SS => self.ss,
            iced_x86::Register::DS => self.ds,
            iced_x86::Register::FS => self.fs,
            iced_x86::Register::GS => self.gs,
            _ => unreachable!("{reg:?}"),
        }
    }

    pub fn get8(&self, reg: iced_x86::Register) -> u8 {
        match reg {
            iced_x86::Register::AL => self.eax as u8,
            iced_x86::Register::CL => self.ecx as u8,
            iced_x86::Register::DL => self.edx as u8,
            iced_x86::Register::BL => self.ebx as u8,
            iced_x86::Register::AH => (self.eax >> 8) as u8,
            iced_x86::Register::CH => (self.ecx >> 8) as u8,
            iced_x86::Register::DH => (self.edx >> 8) as u8,
            iced_x86::Register::BH => (self.ebx >> 8) as u8,
            _ => unreachable!("{reg:?}"),
        }
    }

    pub fn set32(&mut self, reg: iced_x86::Register, value: u32) {
        let idx = reg as usize - iced_x86::Register::EAX as usize;
        if idx >= 8 {
            unreachable!("{reg:?}");
        }
        // XXX this assumes register order matches between our struct and iced_x86.
        unsafe {
            *(self as *mut Registers as *mut u32).add(idx) = value;
        }
    }

    pub fn set16(&mut self, reg: iced_x86::Register, value: u16) {
        match reg {
            iced_x86::Register::AX => self.eax = (self.eax & 0xFFFF_0000) | value as u32,
            iced_x86::Register::CX => self.ecx = (self.ecx & 0xFFFF_0000) | value as u32,
            iced_x86::Register::DX => self.edx = (self.edx & 0xFFFF_0000) | value as u32,
            iced_x86::Register::BX => self.ebx = (self.ebx & 0xFFFF_0000) | value as u32,
            iced_x86::Register::SI => self.esi = (self.esi & 0xFFFF_0000) | value as u32,
            iced_x86::Register::DI => self.edi = (self.edi & 0xFFFF_0000) | value as u32,
            iced_x86::Register::ES => self.es = value,
            iced_x86::Register::CS => self.cs = value,
            iced_x86::Register::SS => self.ss = value,
            iced_x86::Register::DS => self.ds = value,
            iced_x86::Register::FS => self.fs = value,
            iced_x86::Register::GS => self.gs = value,
            _ => unreachable!("{reg:?}"),
        }
    }

    pub fn set8(&mut self, reg: iced_x86::Register, value: u8) {
        match reg {
            iced_x86::Register::AL => self.eax = (self.eax & 0xFFFF_FF00) | value as u32,
            iced_x86::Register::CL => self.ecx = (self.ecx & 0xFFFF_FF00) | value as u32,
            iced_x86::Register::DL => self.edx = (self.edx & 0xFFFF_FF00) | value as u32,
            iced_x86::Register::BL => self.ebx = (self.ebx & 0xFFFF_FF00) | value as u32,

            iced_x86::Register::AH => self.eax = (self.eax & 0xFFFF_00FF) | ((value as u32) << 8),
            iced_x86::Register::CH => self.ecx = (self.ecx & 0xFFFF_00FF) | ((value as u32) << 8),
            iced_x86::Register::DH => self.edx = (self.edx & 0xFFFF_00FF) | ((value as u32) << 8),
            iced_x86::Register::BH => self.ebx = (self.ebx & 0xFFFF_00FF) | ((value as u32) << 8),
            _ => unreachable!("{reg:?}"),
        }
    }

    /// Get st(0), the current top of the FPU stack.
    pub fn st_top(&mut self) -> &mut f64 {
        &mut self.st[self.st_top]
    }
    /// Offset from top of FP stack for a given ST0, ST1 etc reg.
    fn st_offset(&self, reg: iced_x86::Register) -> usize {
        self.st_top
            + match reg {
                iced_x86::Register::ST0 => 0,
                iced_x86::Register::ST1 => 1,
                _ => unreachable!("{reg:?}"),
            }
    }
    pub fn st_swap(&mut self, r1: iced_x86::Register, r2: iced_x86::Register) {
        let o1 = self.st_offset(r1);
        let o2 = self.st_offset(r2);
        self.st.swap(o1, o2);
    }
    pub fn getst(&mut self, reg: iced_x86::Register) -> &mut f64 {
        &mut self.st[self.st_offset(reg)]
    }

    pub fn get64(&self, reg: iced_x86::Register) -> u64 {
        match reg {
            iced_x86::Register::MM0 => self.mm[0],
            iced_x86::Register::MM1 => self.mm[1],
            iced_x86::Register::MM2 => self.mm[2],
            iced_x86::Register::MM3 => self.mm[3],
            iced_x86::Register::MM4 => self.mm[4],
            iced_x86::Register::MM5 => self.mm[5],
            iced_x86::Register::MM6 => self.mm[6],
            iced_x86::Register::MM7 => self.mm[7],
            _ => unimplemented!("{:?}", reg),
        }
    }
    pub fn set64(&mut self, reg: iced_x86::Register, value: u64) {
        match reg {
            iced_x86::Register::MM0 => self.mm[0] = value,
            iced_x86::Register::MM1 => self.mm[1] = value,
            iced_x86::Register::MM2 => self.mm[2] = value,
            iced_x86::Register::MM3 => self.mm[3] = value,
            iced_x86::Register::MM4 => self.mm[4] = value,
            iced_x86::Register::MM5 => self.mm[5] = value,
            iced_x86::Register::MM6 => self.mm[6] = value,
            iced_x86::Register::MM7 => self.mm[7] = value,
            _ => unimplemented!("{:?}", reg),
        }
    }
}
