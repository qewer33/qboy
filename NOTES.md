# qboy notes

## CPU

- ISA table: https://izik1.github.io/gbops/
- RGBDS ISA ref: https://rgbds.gbdev.io/docs/master/gbz80.7
- Pan Docs ISA ref: https://gbdev.io/pandocs/CPU_Instruction_Set.html

## Currently Implemented Instructions

- [ ] Misc
  - [x] `NOP` 
  - [x] `HALT` 
- [x] 8-bit loads
  - [x] `LD r8 r8`
  - [x] `LD r8 u8`
- [ ] 16-bit loads
  - [x] `LD r16 u16` 
  - [ ] `LD [r16] a` 
  - [ ] `LD a [r16]` 
  - [ ] `LD 16 sp` 
- [x] conditional & relative jumps
  - [x] `JP u16`
  - [x] `JP hl`
  - [x] `JP cond u16`
  - [x] `JR i8`
  - [x] `JR cond i8`
- [x] subroutine instructions
  - [x] `CALL u16` 
  - [x] `CALL cond u16` 
  - [x] `RET`
  - [x] `RET u16`
- [ ] 8-bit arithmetic
  - [x] `ADD a r8` 
  - [x] `ADD a u8` 
  - [ ] `ADD a [hl]` 
  - [x] `ADC a r8` 
  - [x] `ADC a u8`
  - [ ] `ADC a [hl]`
  - [x] `SUB a r8` 
  - [x] `SUB a u8` 
  - [ ] `SUB a [hl]` 
  - [x] `SBC a r8` 
  - [x] `SBC a u8`
  - [ ] `SBC a [hl]`
  - [x] `CP a r8` 
  - [x] `CP a u8`
  - [ ] `CP a [hl]`
  - [ ] `INC r8`
  - [ ] `INC [hl]`
  - [ ] `DEC r8`
  - [ ] `DEC [hl]`
