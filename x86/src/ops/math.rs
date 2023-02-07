use iced_x86::Instruction;
use num_traits::{
    ops::overflowing::{OverflowingAdd, OverflowingSub},
    PrimInt,
};

use crate::{registers::Flags, x86::X86, Result};

use super::helpers::*;

/// This trait is implemented for u32/u16/u8 and lets us write operations generically
/// over all those bit sizes.
///
/// Even when we need size-specific masks like "the high bit"
/// (which is x.shr(I::bits() - 1))
/// that math optimizes down to the appropriate constant.
pub(crate) trait Int: PrimInt {
    fn as_usize(self) -> usize;
    fn bits() -> usize;

    /// Wrapper just to avoid the random "u32" in the num-traits definition.
    fn sar(self, other: Self) -> Self {
        self.signed_shr(other.as_usize() as u32)
    }
}
impl Int for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
    fn bits() -> usize {
        32
    }
}
impl Int for u16 {
    fn as_usize(self) -> usize {
        self as usize
    }
    fn bits() -> usize {
        16
    }
}
impl Int for u8 {
    fn as_usize(self) -> usize {
        self as usize
    }
    fn bits() -> usize {
        8
    }
}

fn shl<I: Int>(x86: &mut X86, x: I, y: u8) -> I {
    // Note: overflowing_shl is not what we want.
    let val = (x.as_usize() as u64).wrapping_shl(y.as_usize() as u32);
    x86.regs.flags.set(Flags::CF, val & (1 << 31) != 0);
    val
}

fn shl32(x86: &mut X86, x: u32, y: u8) -> u32 {
    // Note: overflowing_shl is not what we want.
    let val = (x as u64).wrapping_shl(y as u32);
    x86.regs.flags.set(Flags::CF, val & (1 << 31) != 0);
    val as u32
}

fn shl8(x86: &mut X86, x: u8, y: u8) -> u8 {
    // Note: overflowing_shl is not what we want.
    let val = (x as u16).wrapping_shl(y as u32);
    x86.regs.flags.set(Flags::CF, val & (1 << 7) != 0);
    val as u8
}

fn shr32(x86: &mut X86, x: u32, y: u8) -> u32 {
    if y == 0 {
        return x; // Don't affect flags.
    }
    x86.regs.flags.set(Flags::CF, (x >> (y - 1)) & 1 != 0);
    let val = x >> y;
    x86.regs.flags.set(Flags::SF, false); // ?
    x86.regs.flags.set(Flags::ZF, val == 0);

    // Note: OF state undefined for shifts > 1 bit, but the following behavior
    // matches what my Windows box does in practice.
    x86.regs.flags.set(Flags::OF, x & (1 << 31) != 0);
    val
}

// pub(crate) for use in the test opcode impl.
pub(crate) fn and<I: Int>(x86: &mut X86, x: I, y: I) -> I {
    let result = x & y;
    // XXX More flags.
    x86.regs.flags.set(Flags::ZF, result.is_zero());
    x86.regs
        .flags
        .set(Flags::SF, (result >> (I::bits() - 1)).is_one());
    x86.regs.flags.set(Flags::OF, false);
    result
}

pub fn and_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate32();
    rm32_x(x86, instr, |x86, x| and(x86, x, y));
    Ok(())
}

pub fn and_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to32() as u32;
    rm32_x(x86, instr, |x86, x| and(x86, x, y));
    Ok(())
}

pub fn and_rm32_r32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.get32(instr.op1_register());
    rm32_x(x86, instr, |x86, x| and(x86, x, y));
    Ok(())
}

pub fn and_r32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let y = op1_rm32(x86, instr);
    let value = x86.regs.get32(reg) & y;
    x86.regs.set32(reg, value);
    Ok(())
}

pub fn and_rm16_imm16(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate16();
    rm16_x(x86, instr, |x86, x| and(x86, x, y));
    Ok(())
}

pub fn and_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |x86, x| and(x86, x, y));
    Ok(())
}

fn or<I: Int>(x86: &mut X86, x: I, y: I) -> I {
    let result = x | y;
    // XXX More flags.
    x86.regs.flags.set(Flags::ZF, result.is_zero());
    result
}

pub fn or_rm32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = op1_rm32(x86, instr);
    rm32_x(x86, instr, |x86, x| or(x86, x, y));
    Ok(())
}

pub fn or_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate32();
    rm32_x(x86, instr, |x86, x| or(x86, x, y));
    Ok(())
}

pub fn or_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to32() as u32;
    rm32_x(x86, instr, |x86, x| or(x86, x, y));
    Ok(())
}

pub fn or_rm16_imm16(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate16();
    rm16_x(x86, instr, |x86, x| or(x86, x, y));
    Ok(())
}

pub fn or_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |x86, x| or(x86, x, y));
    Ok(())
}

pub fn shl_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm32_x(x86, instr, |x86, x| shl32(x86, x, y));
    Ok(())
}

pub fn shl_rm32_cl(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.ecx as u8;
    rm32_x(x86, instr, |x86, x| shl32(x86, x, y));
    Ok(())
}

pub fn shl_rm8_cl(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.ecx as u8;
    rm8_x(x86, instr, |x86, x| shl8(x86, x, y));
    Ok(())
}

pub fn shl_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |x86, x| shl8(x86, x, y));
    Ok(())
}

pub fn shr_rm32_cl(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.ecx as u8;
    rm32_x(x86, instr, |x86, x| shr32(x86, x, y));
    Ok(())
}

pub fn shr_rm32_1(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm32_x(x86, instr, |x86, x| shr32(x86, x, 1));
    Ok(())
}

pub fn shr_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm32_x(x86, instr, |x86, x| shr32(x86, x, y));
    Ok(())
}

fn sar<I: Int>(x86: &mut X86, x: I, y: I) -> I {
    if y.is_zero() {
        return x;
    }
    x86.regs
        .flags
        .set(Flags::CF, x.shr(y.as_usize() - 1).bitand(I::one()).is_one());
    x86.regs.flags.set(Flags::OF, false);
    let result = x.sar(y);
    x86.regs
        .flags
        .set(Flags::SF, result.shr(I::bits() - 1).is_one());
    x86.regs.flags.set(Flags::ZF, result.is_zero());
    result
}

pub fn sar_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8() as u32;
    rm32_x(x86, instr, |x86, x| sar(x86, x, y));
    Ok(())
}

pub fn sar_rm32_cl(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.ecx as u8 as u32;
    rm32_x(x86, instr, |x86, x| sar(x86, x, y));
    Ok(())
}

pub fn sar_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8() as u8;
    rm8_x(x86, instr, |x86, x| sar(x86, x, y));
    Ok(())
}

pub fn ror_rm32_cl(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.ecx as u8;
    rm32_x(x86, instr, |x86, x| {
        let out = x.rotate_right(y as u32);
        let msb = (out & 0x8000_0000) != 0;
        x86.regs.flags.set(Flags::CF, msb);
        x86.regs
            .flags
            .set(Flags::OF, msb ^ ((out & 04000_0000) != 0));
        out
    });
    Ok(())
}

fn xor32(x86: &mut X86, x: u32, y: u32) -> u32 {
    let result = x ^ y;
    // The OF and CF flags are cleared; the SF, ZF, and PF flags are set according to the result. The state of the AF flag is undefined.
    x86.regs.flags.remove(Flags::OF);
    x86.regs.flags.remove(Flags::CF);
    x86.regs.flags.set(Flags::ZF, result == 0);
    x86.regs.flags.set(Flags::SF, result & 0x8000_0000 != 0);
    result
}

pub fn xor_rm32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = op1_rm32(x86, instr);
    rm32_x(x86, instr, |x86, x| xor32(x86, x, y));
    Ok(())
}

pub fn xor_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate32();
    rm32_x(x86, instr, |x86, x| xor32(x86, x, y));
    Ok(())
}

pub fn xor_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to32() as u32;
    rm32_x(x86, instr, |x86, x| xor32(x86, x, y));
    Ok(())
}

pub fn xor_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |_x86, x| x ^ y);
    // TODO: flags
    Ok(())
}

pub fn xor_r8_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = op1_rm8(x86, instr);
    rm8_x(x86, instr, |_x86, x| x ^ y);
    // TODO: flags
    Ok(())
}

fn add<I: Int + OverflowingAdd>(x86: &mut X86, x: I, y: I) -> I {
    // TODO "The CF, OF, SF, ZF, AF, and PF flags are set according to the result."
    let (result, carry) = x.overflowing_add(&y);
    x86.regs.flags.set(Flags::CF, carry);
    x86.regs.flags.set(Flags::ZF, result.is_zero());
    x86.regs
        .flags
        .set(Flags::SF, (result >> (I::bits() - 1)).is_one());
    // Overflow is true exactly when the high (sign) bits are like:
    //   x  y  result
    //   0  0  1
    //   1  1  0
    let of = !(((x ^ !y) & (x ^ result)) >> (I::bits() - 1)).is_zero();
    x86.regs.flags.set(Flags::OF, of);
    result
}

pub fn add_r32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let x = x86.regs.get32(reg);
    let y = op1_rm32(x86, &instr);
    let value = add(x86, x, y);
    x86.regs.set32(reg, value);
    Ok(())
}

pub fn add_rm32_r32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.get32(instr.op1_register());
    rm32_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}
pub fn add_rm32_r32_2(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.get32(instr.op1_register());
    rm32_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

pub fn add_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate32();
    rm32_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

pub fn add_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to32() as u32;
    rm32_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

pub fn add_rm16_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to16() as u16;
    rm16_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

pub fn add_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

pub fn add_r8_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = op1_rm8(x86, instr);
    rm8_x(x86, instr, |x86, x| add(x86, x, y));
    Ok(())
}

// pub(crate) for use in the cmp opcode impl.
pub(crate) fn sub<I: Int + OverflowingSub>(x86: &mut X86, x: I, y: I) -> I {
    let (result, carry) = x.overflowing_sub(&y);
    // TODO "The CF, OF, SF, ZF, AF, and PF flags are set according to the result."
    x86.regs.flags.set(Flags::CF, carry);
    x86.regs.flags.set(Flags::ZF, result.is_zero());
    x86.regs
        .flags
        .set(Flags::SF, (result >> (I::bits() - 1)).is_one());
    // Overflow is true exactly when the high (sign) bits are like:
    //   x  y  result
    //   0  1  1
    //   1  0  0
    let of = !(((x ^ y) & (x ^ result)) >> (I::bits() - 1)).is_zero();
    x86.regs.flags.set(Flags::OF, of);
    result
}

pub fn sub_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8to32() as u32;
    rm32_x(x86, instr, |x86, x| sub(x86, x, y));
    Ok(())
}

pub fn sub_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate32();
    rm32_x(x86, instr, |x86, x| sub(x86, x, y));
    Ok(())
}

pub fn sub_rm32_r32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = x86.regs.get32(instr.op1_register());
    rm32_x(x86, instr, |x86, x| sub(x86, x, y));
    Ok(())
}

pub fn sub_r32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let y = op1_rm32(x86, instr);
    let value = sub(x86, x86.regs.get32(reg), y);
    x86.regs.set32(reg, value);
    Ok(())
}

pub fn sub_r8_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let y = op1_rm8(x86, instr);
    let value = sub(x86, x86.regs.get8(reg), y);
    x86.regs.set8(reg, value);
    Ok(())
}

pub fn sub_rm8_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let y = instr.immediate8();
    rm8_x(x86, instr, |x86, x| sub(x86, x, y));
    Ok(())
}

pub fn sbb_r32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let carry = x86.regs.flags.contains(Flags::CF) as u32;
    let y = op1_rm32(x86, instr).wrapping_add(carry);
    let value = sub(x86, x86.regs.get32(reg), y);
    x86.regs.set32(reg, value);
    Ok(())
}

pub fn sbb_r8_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let reg = instr.op0_register();
    let carry = x86.regs.flags.contains(Flags::CF) as u8;
    let y = op1_rm8(x86, instr).wrapping_add(carry);
    let value = sub(x86, x86.regs.get8(reg), y);
    x86.regs.set8(reg, value);
    Ok(())
}

pub fn imul_r32_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let x = x86.regs.get32(instr.op0_register());
    let y = op1_rm32(x86, instr);
    let value = x.wrapping_mul(y);
    x86.regs.set32(instr.op0_register(), value);
    Ok(())
}

pub fn imul_r32_rm32_imm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let x = op1_rm32(x86, instr) as i32;
    let y = instr.immediate32() as i32;
    let value = x.wrapping_mul(y);
    x86.regs.set32(instr.op0_register(), value as u32);
    Ok(())
}

pub fn imul_r32_rm32_imm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let x = op1_rm32(x86, instr) as i32;
    let y = instr.immediate8to32();
    let value = x.wrapping_mul(y);
    x86.regs.set32(instr.op0_register(), value as u32);
    Ok(())
}

pub fn idiv_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let x = (((x86.regs.edx as u64) << 32) | (x86.regs.eax as u64)) as i64;
    let y = op0_rm32(x86, instr) as i32 as i64;
    x86.regs.eax = (x / y) as i32 as u32;
    x86.regs.edx = (x % y) as i32 as u32;
    // TODO: flags.
    Ok(())
}

pub fn div_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    let x = ((x86.regs.edx as u64) << 32) | (x86.regs.eax as u64);
    let y = op0_rm32(x86, instr) as u64;
    x86.regs.eax = (x / y) as u32;
    x86.regs.edx = (x % y) as u32;
    // TODO: flags.
    Ok(())
}

pub fn dec_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm32_x(x86, instr, |x86, x| sub(x86, x, 1));
    Ok(())
}

pub fn dec_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm8_x(x86, instr, |x86, x| sub(x86, x, 1));
    Ok(())
}

pub fn inc_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    // TODO: flags.  Note that it's not add(1) because CF should be preserved.
    rm32_x(x86, instr, |_x86, x| x + 1);
    Ok(())
}

pub fn inc_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    // TODO: flags.  Note that it's not add(1) because CF should be preserved.
    rm8_x(x86, instr, |_x86, x| x.wrapping_add(1));
    Ok(())
}

pub fn neg_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm32_x(x86, instr, |x86, x| {
        x86.regs.flags.set(Flags::CF, x != 0);
        // TODO: other flags registers.
        -(x as i32) as u32
    });
    Ok(())
}

pub fn neg_rm8(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm8_x(x86, instr, |x86, x| {
        x86.regs.flags.set(Flags::CF, x != 0);
        // TODO: other flags registers.
        -(x as i8) as u8
    });
    Ok(())
}

pub fn not_rm32(x86: &mut X86, instr: &Instruction) -> Result<()> {
    rm32_x(x86, instr, |_x86, x| !x);
    Ok(())
}
