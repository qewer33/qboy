# qboy notes

## CPU

- ISA table: https://izik1.github.io/gbops/
- RGBDS ISA ref: https://rgbds.gbdev.io/docs/master/gbz80.7
- Pan Docs ISA ref: https://gbdev.io/pandocs/CPU_Instruction_Set.html

## Currently Implemented Instructions

- [ ] Misc
  - [x] `NOP` 
  - [x] `HALT` 
  - [x] `DI` 
  - [x] `EI` 
  - [ ] `STOP`
  - [ ] `DAA`
- [x] 8-bit loads
  - [x] `LD r8 r8`
  - [x] `LD r8 u8`
- [ ] 16-bit loads
  - [x] `LD r16 u16` 
  - [ ] `LD [r16] a` 
  - [ ] `LD a [r16]` 
  - [ ] `LD 16 sp` 
- [x] jumps & subroutine instructions
  - [x] `JP u16`
  - [x] `JP hl`
  - [x] `JP cond u16`
  - [x] `JR i8`
  - [x] `JR cond i8`
  - [x] `CALL u16` 
  - [x] `CALL cond u16` 
  - [x] `RET`
  - [x] `RET u16`
- [x] 8-bit arithmetic
  - [x] `ADD a r8` 
  - [x] `ADD a u8` 
  - [x] `ADC a r8` 
  - [x] `ADC a u8`
  - [x] `SUB a r8` 
  - [x] `SUB a u8` 
  - [x] `SBC a r8` 
  - [x] `SBC a u8`
  - [x] `CP a r8` 
  - [x] `CP a u8`
  - [x] `INC r8`
  - [x] `DEC r8`
