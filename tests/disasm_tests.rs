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

#[test]
fn test_pe_file_loading() {
    // Cross-compile a small test program to PE
    let dir = tempfile::tempdir().unwrap();
    let c_path = dir.path().join("test.c");
    let exe_path = dir.path().join("test.exe");

    std::fs::write(&c_path, r#"
        int add(int a, int b) { return a + b; }
        int multiply(int a, int b) { return a * b; }
        int main() { return add(1, 2) + multiply(3, 4); }
    "#).unwrap();

    let output = std::process::Command::new("x86_64-w64-mingw32-gcc")
        .args(["-g", "-O0", "-o"])
        .arg(&exe_path)
        .arg(&c_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let disasm = Disassembler::from_file(&exe_path).expect("Failed to load PE");
            assert_eq!(disasm.arch(), DisasmArch::X86_64);
            assert!(!disasm.bytes().is_empty(), "PE .text should have bytes");

            // Should have COFF symbols including our functions
            let syms = disasm.symbols();
            assert!(!syms.is_empty(), "PE should have COFF symbols");

            let has_main = syms.iter().any(|s| s.name == "main");
            let has_add = syms.iter().any(|s| s.name == "add");
            let has_multiply = syms.iter().any(|s| s.name == "multiply");
            assert!(has_main, "Should find 'main' symbol, got: {:?}",
                syms.iter().map(|s| &s.name).collect::<Vec<_>>());
            assert!(has_add, "Should find 'add' symbol");
            assert!(has_multiply, "Should find 'multiply' symbol");

            // Should disassemble at entry point
            let lines = disasm.disassemble_range(0, 20);
            assert!(!lines.is_empty(), "Should disassemble PE .text");
            assert!(lines[0].mnemonic.len() > 0, "Instructions should have mnemonics");

            // Go to a symbol by name
            let add_sym = disasm.find_symbol("add").expect("Should find add symbol");
            let add_lines = disasm.disassemble_at_address(add_sym.address, 5);
            assert!(!add_lines.is_empty(), "Should disassemble at 'add' address");
        }
        _ => {
            eprintln!("Skipping PE test: x86_64-w64-mingw32-gcc not available");
        }
    }
}

#[test]
fn test_pe_coff_symbol_addresses_are_virtual() {
    // Ensure COFF symbols have proper virtual addresses (image_base + section VA + value)
    let dir = tempfile::tempdir().unwrap();
    let c_path = dir.path().join("test.c");
    let exe_path = dir.path().join("test.exe");

    std::fs::write(&c_path, "int myfunc(void) { return 42; } int main() { return myfunc(); }").unwrap();

    let output = std::process::Command::new("x86_64-w64-mingw32-gcc")
        .args(["-g", "-O0", "-o"])
        .arg(&exe_path)
        .arg(&c_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let disasm = Disassembler::from_file(&exe_path).expect("Failed to load PE");
            if let Some(sym) = disasm.find_symbol("myfunc") {
                // Symbol address should be >= base_address (in virtual address space)
                assert!(sym.address >= disasm.base_address(),
                    "Symbol address 0x{:X} should be >= base 0x{:X}",
                    sym.address, disasm.base_address());
                // Should be able to disassemble at that address
                let lines = disasm.disassemble_at_address(sym.address, 3);
                assert!(!lines.is_empty(), "Should disassemble at myfunc");
            }
        }
        _ => {
            eprintln!("Skipping PE COFF test: mingw not available");
        }
    }
}

#[test]
fn test_find_symbol_prefers_exact_match() {
    // find_symbol should prefer exact case-insensitive match over substring
    let dir = tempfile::tempdir().unwrap();
    let c_path = dir.path().join("test.c");
    let exe_path = dir.path().join("test.exe");

    std::fs::write(&c_path, r#"
        int submain(void) { return 1; }
        int main() { return submain(); }
    "#).unwrap();

    let output = std::process::Command::new("x86_64-w64-mingw32-gcc")
        .args(["-g", "-O0", "-o"])
        .arg(&exe_path)
        .arg(&c_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let disasm = Disassembler::from_file(&exe_path).expect("Failed to load PE");
            // "main" should match exactly "main", not "__tmainCRTStartup" or "submain"
            let sym = disasm.find_symbol("main").expect("Should find main");
            assert_eq!(sym.name, "main", "Should prefer exact match, got: {}", sym.name);
        }
        _ => {
            eprintln!("Skipping exact match test: mingw not available");
        }
    }
}

#[test]
fn test_call_target_resolved_to_symbol() {
    // call rel32 to address 0x1050 where we have a symbol "my_func"
    let mut bytes = vec![0x90; 0x60];
    bytes[0] = 0xE8; // call rel32
    let rel: i32 = 0x1050_i64 as i32 - (0x1000 + 5);
    bytes[1..5].copy_from_slice(&rel.to_le_bytes());
    bytes[0x50] = 0xC3;

    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    disasm.add_symbol("my_func".to_string(), 0x1050, 16);

    let lines = disasm.disassemble_range(0, 5);
    assert_eq!(lines[0].mnemonic, "call");
    // Operands should show the symbol name
    assert_eq!(lines[0].operands, "my_func");
    // Comment should show the raw operand
    assert!(lines[0].comment.is_some(), "should have raw operand as comment");
    assert!(lines[0].comment.as_deref().unwrap().contains("0x1050"),
        "comment should contain raw address, got: {:?}", lines[0].comment);
}

#[test]
fn test_call_target_with_offset() {
    // call to an address inside a function (not at its start)
    let mut bytes = vec![0x90; 0x60];
    bytes[0] = 0xE8; // call rel32
    let target = 0x1054_u64;
    let rel: i32 = target as i32 - (0x1000 + 5);
    bytes[1..5].copy_from_slice(&rel.to_le_bytes());

    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    disasm.add_symbol("my_func".to_string(), 0x1050, 16);

    let lines = disasm.disassemble_range(0, 5);
    assert_eq!(lines[0].mnemonic, "call");
    // Operands should show symbol+offset
    assert_eq!(lines[0].operands, "my_func+0x4");
    // Comment should show raw address
    assert!(lines[0].comment.is_some());
}

#[test]
fn test_rip_relative_lea_resolved() {
    // LEA rax, [rip + disp32] → operands should show [my_data]
    let mut bytes = vec![0x90; 0x60];
    let disp: i32 = 0x1050_i64 as i32 - 0x1007;
    bytes[0] = 0x48;
    bytes[1] = 0x8D;
    bytes[2] = 0x05;
    bytes[3..7].copy_from_slice(&disp.to_le_bytes());

    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    disasm.add_symbol("my_data".to_string(), 0x1050, 8);

    let lines = disasm.disassemble_range(0, 3);
    assert_eq!(lines[0].mnemonic, "lea");
    // Operands should have the symbol name substituted for [rip + ...]
    assert!(lines[0].operands.contains("my_data"),
        "operands should contain symbol name, got: {}", lines[0].operands);
    // Comment should have the raw operand
    assert!(lines[0].comment.is_some(), "should have raw operand as comment");
    assert!(lines[0].comment.as_deref().unwrap().contains("rip"),
        "comment should contain raw rip-relative, got: {:?}", lines[0].comment);
}

#[test]
fn test_comment_field_none_when_no_symbol() {
    // Simple nop instructions should have no comment
    let bytes = vec![0x90, 0x90, 0x90];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    let lines = disasm.disassemble_range(0, 3);
    for line in &lines {
        assert!(line.comment.is_none(), "nop should have no comment");
    }
}

#[test]
fn test_pe_symbolic_resolution() {
    // Compile a PE with known function calls and verify resolution
    let dir = tempfile::tempdir().unwrap();
    let c_path = dir.path().join("test.c");
    let exe_path = dir.path().join("test.exe");

    std::fs::write(&c_path, r#"
        int helper(int x) { return x * 2; }
        int caller(int x) { return helper(x) + 1; }
        int main() { return caller(21); }
    "#).unwrap();

    let output = std::process::Command::new("x86_64-w64-mingw32-gcc")
        .args(["-g", "-O0", "-o"])
        .arg(&exe_path)
        .arg(&c_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let disasm = Disassembler::from_file(&exe_path).expect("Failed to load PE");
            // Disassemble at 'caller' which calls 'helper'
            let caller_sym = disasm.find_symbol("caller").expect("Should find caller");
            let lines = disasm.disassemble_at_address(caller_sym.address, 20);

            // Find any call instruction and check if it has a comment
            let call_lines: Vec<_> = lines.iter().filter(|l| l.mnemonic == "call").collect();
            assert!(!call_lines.is_empty(), "caller should contain a call instruction");
            // At least one call should resolve to a symbol
            let resolved: Vec<_> = call_lines.iter().filter(|l| l.comment.is_some()).collect();
            assert!(!resolved.is_empty(),
                "At least one call in caller should resolve to a symbol name, got: {:?}",
                call_lines.iter().map(|l| (&l.operands, &l.comment)).collect::<Vec<_>>());
        }
        _ => {
            eprintln!("Skipping PE symbolic test: mingw not available");
        }
    }
}
