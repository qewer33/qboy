#[cfg(test)]
mod tests {
    use qboy_core::{
        cpu::Cpu,
        mem::{FlatMem, Mem},
    };

    fn setup_test_cpu() -> Cpu {
        let cpu = Cpu::new();
        cpu
    }

    fn setup_test_mem() -> FlatMem {
        let mem = FlatMem::new();
        mem
    }

    #[test]
    fn test_nop() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x00 = NOP
        mem.write(0x0000, 0x00);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(cpu.regs.pc, 1, "program counter should be incremented by 1");
        assert_eq!(mcycles, 1, "1 M-cycles should have passed")
    }

    #[test]
    fn test_halt() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x76 = HALT
        mem.write(0x0000, 0x76);

        let mcycles = cpu.step(&mut mem);

        assert!(cpu.halted, "cpu should be halted");
        assert_eq!(cpu.regs.pc, 1, "program counter should be incremented by 1");
        assert_eq!(mcycles, 1, "1 M-cycles should have passed");
    }

    #[test]
    fn test_ld_b_a() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // write a value to A
        cpu.regs.a = 0x42;

        // 0x47 = LD B A
        mem.write(0x0000, 0x47);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(cpu.regs.b, cpu.regs.a, "B should match value of A");
        assert_eq!(cpu.regs.pc, 1, "program counter should be incremented by 1");
        assert_eq!(mcycles, 1, "1 M-cycles should have passed");
    }

    #[test]
    fn test_ld_a_hl() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // write an address to HL
        cpu.regs.set_hl(0x00FF);

        // write a value at HL
        mem.write(cpu.regs.hl(), 0x42);

        // 0x7E = LD A (HL)
        mem.write(0x0000, 0x7E);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.a,
            mem.read(cpu.regs.hl()),
            "register A should match value at HL address"
        );
        assert_eq!(cpu.regs.pc, 1, "program counter should be incremented by 1");
        assert_eq!(mcycles, 2, "2 M-cycles should have passed");
    }

    #[test]
    fn test_ld_a_u8() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x3E = LD A u8
        mem.write(0x0000, 0x3E);
        // write operand
        mem.write(0x0001, 0x42);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(cpu.regs.a, 0x42, "register A should match operand value");
        assert_eq!(cpu.regs.pc, 2, "program counter should be incremented by 2");
        assert_eq!(mcycles, 2, "2 M-cycles should have passed");
    }

    #[test]
    fn test_ld_sp_u16() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x31 = LD SP u16
        mem.write(0x0000, 0x31);
        // write operand
        mem.write(0x0001, 0x42);
        mem.write(0x0002, 0x42);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.sp, 0x4242,
            "register SP should match operand value"
        );
        assert_eq!(cpu.regs.pc, 3, "program counter should be incremented by 2");
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_jp_u16() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xC3 = JP u16
        mem.write(0x0000, 0xC3);
        // write operand
        mem.write(0x0001, 0x42);
        mem.write(0x0002, 0x42);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.pc, 0x4242,
            "program counter should match operand value"
        );
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_jp_z_u16() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCA = JP z u16
        mem.write(0x0000, 0xCA);
        // write operand
        mem.write(0x0001, 0x42);
        mem.write(0x0002, 0x42);

        // set zero flag
        cpu.regs.set_f_zero(true);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.pc, 0x4242,
            "program counter should match operand value"
        );
        assert_eq!(mcycles, 4, "4 M-cycles should have passed");
    }

    #[test]
    fn test_jp_z_u16_nobranch() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCA = JP z u16
        mem.write(0x0000, 0xCA);
        // write operand
        mem.write(0x0001, 0x42);
        mem.write(0x0002, 0x42);

        // set zero flag
        cpu.regs.set_f_zero(false);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(cpu.regs.pc, 3, "program counter should be incremented by 3");
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_jr_i8() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xC3 = JR i8
        mem.write(0x0000, 0x18);
        // write operand
        mem.write(0x0001, 0x12);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.pc,
            0x12 + 2,
            "program counter should offset by operand value plus 2"
        );
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_jr_c_i8() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x38 = JR C i8
        mem.write(0x0000, 0x38);
        // write operand
        mem.write(0x0001, 0x12);

        // set carry flag
        cpu.regs.set_f_carry(true);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(
            cpu.regs.pc,
            0x12 + 2,
            "program counter should offset by operand value plus 2"
        );
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_jr_c_i8_nobranch() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0x38 = JR C i8
        mem.write(0x0000, 0x38);
        // write operand
        mem.write(0x0001, 0x12);

        // set carry flag
        cpu.regs.set_f_carry(false);

        let mcycles = cpu.step(&mut mem);

        assert_eq!(cpu.regs.pc, 2, "program counter should be incremented by 2");
        assert_eq!(mcycles, 2, "2 M-cycles should have passed");
    }

    #[test]
    fn test_call_u16_ret() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCD = CALL u16
        mem.write(0x0000, 0xCD);
        // write operand
        mem.write(0x0001, 0x00);
        mem.write(0x0001, 0x42);

        // 0xC9 = RET
        mem.write(0x0042, 0xC9);

        let mcycles = cpu.step(&mut mem); // CALL u16

        assert_eq!(
            cpu.regs.pc, 0x0042,
            "program counter should match operand value"
        );
        assert_eq!(mcycles, 6, "6 M-cycles should have passed");

        let mcycles = cpu.step(&mut mem); // RET

        assert_eq!(cpu.regs.pc, 3, "program counter should be 3");
        assert_eq!(mcycles, 4, "4 M-cycles should have passed");
    }

    #[test]
    fn test_call_z_u16_ret_z() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCC = CALL Z u16
        mem.write(0x0000, 0xCC);
        // write operand
        mem.write(0x0001, 0x00);
        mem.write(0x0001, 0x42);

        // 0xC8 = RET Z
        mem.write(0x0042, 0xC8);

        // set zero flag
        cpu.regs.set_f_zero(true);

        let mcycles = cpu.step(&mut mem); // CALL > u16

        assert_eq!(
            cpu.regs.pc, 0x0042,
            "program counter should match operand value"
        );
        assert_eq!(mcycles, 6, "6 M-cycles should have passed");

        let mcycles = cpu.step(&mut mem); // RET >

        assert_eq!(cpu.regs.pc, 3, "program counter should be 3");
        assert_eq!(mcycles, 4, "4 M-cycles should have passed");
    }

    #[test]
    fn test_call_z_nobranch() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCC = CALL Z u16
        mem.write(0x0000, 0xCC);
        // write operand
        mem.write(0x0001, 0x00);
        mem.write(0x0001, 0x42);

        // set zero flag
        cpu.regs.set_f_zero(false);

        let mcycles = cpu.step(&mut mem); // CALL > u16

        assert_eq!(cpu.regs.pc, 2, "program counter should be 2");
        assert_eq!(mcycles, 3, "3 M-cycles should have passed");
    }

    #[test]
    fn test_call_u16_ret_z_nobranch() {
        let mut cpu = setup_test_cpu();
        let mut mem = setup_test_mem();

        // 0xCD = CALL u16
        mem.write(0x0000, 0xCD);
        // write operand
        mem.write(0x0001, 0x00);
        mem.write(0x0001, 0x42);

        // 0xC8 = RET Z
        mem.write(0x0042, 0xC8);

        // set zero flag
        cpu.regs.set_f_zero(false);

        let mcycles = cpu.step(&mut mem); // CALL u16

        assert_eq!(
            cpu.regs.pc, 0x0042,
            "program counter should match operand value"
        );
        assert_eq!(mcycles, 6, "6 M-cycles should have passed");

        let mcycles = cpu.step(&mut mem); // RET Z

        assert_eq!(
            cpu.regs.pc,
            0x0042 + 1,
            "program counter should be previous + 1"
        );
        assert_eq!(mcycles, 2, "2 M-cycles should have passed");
    }
}
