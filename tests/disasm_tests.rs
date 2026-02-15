use notepadppp::tools::disasm::{DisasmArch, Disassembler};

#[test]
fn test_from_bytes_x86_64() {
    let bytes = vec![0x55, 0x48, 0x89, 0xe5]; // push rbp; mov rbp, rsp
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x401000);
    assert_eq!(disasm.base_address(), 0x401000);
    assert_eq!(disasm.arch(), DisasmArch::X86_64);
    assert_eq!(disasm.bytes().len(), 4);
}

#[test]
fn test_disassemble_range_x86_64() {
    let bytes = vec![0x55, 0x48, 0x89, 0xe5]; // push rbp; mov rbp, rsp
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x401000);
    let lines = disasm.disassemble_range(0, 10);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].mnemonic, "push");
    assert_eq!(lines[0].operands, "rbp");
    assert_eq!(lines[0].address, 0x401000);
    assert_eq!(lines[0].bytes, vec![0x55]);

    assert_eq!(lines[1].mnemonic, "mov");
    assert_eq!(lines[1].operands, "rbp, rsp");
    assert_eq!(lines[1].address, 0x401001);
    assert_eq!(lines[1].bytes, vec![0x48, 0x89, 0xe5]);
}

#[test]
fn test_disassemble_range_instruction_count() {
    // nop; nop; nop; nop; nop
    let bytes = vec![0x90, 0x90, 0x90, 0x90, 0x90];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0);
    let lines = disasm.disassemble_range(0, 3);
    assert_eq!(lines.len(), 3);
    for line in &lines {
        assert_eq!(line.mnemonic, "nop");
    }
}

#[test]
fn test_disassemble_range_beyond_end() {
    let bytes = vec![0x90];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0);
    let lines = disasm.disassemble_range(100, 10);
    assert!(lines.is_empty());
}

#[test]
fn test_set_base_address() {
    let bytes = vec![0x90]; // nop
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0);
    let lines = disasm.disassemble_range(0, 1);
    assert_eq!(lines[0].address, 0);

    disasm.set_base_address(0x1000);
    let lines = disasm.disassemble_range(0, 1);
    assert_eq!(lines[0].address, 0x1000);
}

#[test]
fn test_set_arch() {
    let bytes = vec![0x55, 0x48, 0x89, 0xe5];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0);
    assert_eq!(disasm.arch(), DisasmArch::X86_64);

    disasm.set_arch(DisasmArch::X86_32);
    assert_eq!(disasm.arch(), DisasmArch::X86_32);
    // In 32-bit mode, same bytes decode differently
    let lines = disasm.disassemble_range(0, 10);
    assert!(!lines.is_empty());
    assert_eq!(lines[0].mnemonic, "push");
    assert_eq!(lines[0].operands, "ebp");
}

#[test]
fn test_address_to_offset() {
    let bytes = vec![0x90; 100];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    assert_eq!(disasm.address_to_offset(0x1000), Some(0));
    assert_eq!(disasm.address_to_offset(0x1032), Some(0x32));
    assert_eq!(disasm.address_to_offset(0x0FFF), None);
    assert_eq!(disasm.address_to_offset(0x2000), None);
}

#[test]
fn test_offset_to_address() {
    let bytes = vec![0x90; 100];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    assert_eq!(disasm.offset_to_address(0), 0x1000);
    assert_eq!(disasm.offset_to_address(50), 0x1032);
}

#[test]
fn test_find_symbol() {
    let bytes = vec![0x90; 100];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    // Manually add symbols since from_bytes doesn't load any
    // We'll test via from_bytes + manual symbol insertion indirectly by testing find_symbol
    // returns None when no symbols loaded
    assert!(disasm.find_symbol("main").is_none());
}

#[test]
fn test_symbol_at_address() {
    let bytes = vec![0x90; 100];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    assert!(disasm.symbol_at_address(0x1000).is_none());
}

#[test]
fn test_disassemble_at_address() {
    let bytes = vec![0x90, 0x90, 0x90, 0x55, 0x48, 0x89, 0xe5];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    let lines = disasm.disassemble_at_address(0x1003, 2);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].mnemonic, "push");
    assert_eq!(lines[0].address, 0x1003);
}

#[test]
fn test_disassemble_at_invalid_address() {
    let bytes = vec![0x90];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    let lines = disasm.disassemble_at_address(0, 1);
    assert!(lines.is_empty());
}

#[test]
fn test_x86_32_disassembly() {
    // push ebp; mov ebp, esp; sub esp, 0x10
    let bytes = vec![0x55, 0x89, 0xe5, 0x83, 0xec, 0x10];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_32, 0);
    let lines = disasm.disassemble_range(0, 10);
    assert!(lines.len() >= 3);
    assert_eq!(lines[0].mnemonic, "push");
    assert_eq!(lines[0].operands, "ebp");
    assert_eq!(lines[1].mnemonic, "mov");
    assert_eq!(lines[1].operands, "ebp, esp");
}

#[test]
fn test_elf_magic_detection() {
    // Construct a minimal ELF header (64-bit x86_64)
    let mut elf = vec![0u8; 64]; // ELF header size for 64-bit
    // e_ident
    elf[0] = 0x7f;
    elf[1] = b'E';
    elf[2] = b'L';
    elf[3] = b'F';
    elf[4] = 2; // ELFCLASS64
    elf[5] = 1; // ELFDATA2LSB
    elf[6] = 1; // EV_CURRENT
    // e_type = ET_EXEC (2)
    elf[16] = 2;
    elf[17] = 0;
    // e_machine = EM_X86_64 (0x3e)
    elf[18] = 0x3e;
    elf[19] = 0;
    // e_version
    elf[20] = 1;
    // e_ehsize = 64
    elf[52] = 64;
    elf[53] = 0;

    // Write to temp file and test from_file
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.elf");
    std::fs::write(&path, &elf).unwrap();

    let result = Disassembler::from_file(&path);
    // Should succeed (even if it's a minimal/empty ELF)
    assert!(result.is_ok());
    let disasm = result.unwrap();
    assert_eq!(disasm.arch(), DisasmArch::X86_64);
}

#[test]
fn test_empty_bytes() {
    let disasm = Disassembler::from_bytes(vec![], DisasmArch::X86_64, 0);
    let lines = disasm.disassemble_range(0, 10);
    assert!(lines.is_empty());
}

#[test]
fn test_disasm_line_bytes_match_instruction() {
    // sub rsp, 0x20
    let bytes = vec![0x48, 0x83, 0xec, 0x20];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0);
    let lines = disasm.disassemble_range(0, 1);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].mnemonic, "sub");
    assert_eq!(lines[0].bytes, vec![0x48, 0x83, 0xec, 0x20]);
}
