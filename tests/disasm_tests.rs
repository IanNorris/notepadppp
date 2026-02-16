use notepadppp::tools::disasm::{
    compute_branch_arrows, render_arrow_column, DisasmArch, DisasmLine, Disassembler,
};

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

#[test]
fn test_write_bytes_updates_disassembly() {
    // Start with nops, overwrite first byte with ret (0xC3)
    let bytes = vec![0x90, 0x90, 0x90, 0x90];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);

    let lines = disasm.disassemble_range(0, 4);
    assert_eq!(lines[0].mnemonic, "nop");

    disasm.write_bytes(0, &[0xC3]);
    let lines = disasm.disassemble_range(0, 4);
    assert_eq!(lines[0].mnemonic, "ret", "After write_bytes, instruction should change");
    assert_eq!(lines[0].bytes, vec![0xC3]);
}

#[test]
fn test_write_bytes_out_of_bounds() {
    let bytes = vec![0x90, 0x90];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    // Writing beyond buffer should not panic
    disasm.write_bytes(1, &[0xC3, 0xCC, 0xCC]);
    // Only offset 1 should be written
    assert_eq!(disasm.bytes()[1], 0xC3);
}

// --- Branch Arrow Tests ---

fn make_line(addr: u64, mnemonic: &str, branch_target: Option<u64>) -> DisasmLine {
    DisasmLine {
        address: addr,
        bytes: vec![0x90],
        mnemonic: mnemonic.to_string(),
        operands: branch_target
            .map(|t| format!("0x{:x}", t))
            .unwrap_or_default(),
        comment: None,
        symbol: None,
        is_branch_target: false,
        branch_target,
    }
}

#[test]
fn test_branch_arrows_forward_jump() {
    // jmp from line 0 to line 3
    let lines = vec![
        make_line(0x1000, "jmp", Some(0x1003)),
        make_line(0x1001, "nop", None),
        make_line(0x1002, "nop", None),
        make_line(0x1003, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 1);
    assert_eq!(arrows[0].from_line, 0);
    assert_eq!(arrows[0].to_line, 3);
    assert!(!arrows[0].goes_up);
    assert_eq!(arrows[0].lane, 0);
}

#[test]
fn test_branch_arrows_backward_jump() {
    // je from line 3 back to line 0
    let lines = vec![
        make_line(0x1000, "nop", None),
        make_line(0x1001, "nop", None),
        make_line(0x1002, "nop", None),
        make_line(0x1003, "je", Some(0x1000)),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 1);
    assert_eq!(arrows[0].from_line, 3);
    assert_eq!(arrows[0].to_line, 0);
    assert!(arrows[0].goes_up);
}

#[test]
fn test_branch_arrows_no_call_arrows() {
    // Calls should NOT generate arrows (they go to different functions)
    let lines = vec![
        make_line(0x1000, "call", Some(0x2000)),
        make_line(0x1005, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 0);
}

#[test]
fn test_branch_arrows_target_out_of_view() {
    // Jump target not in visible lines — no arrow
    let lines = vec![
        make_line(0x1000, "jmp", Some(0x5000)),
        make_line(0x1001, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 0);
}

#[test]
fn test_branch_arrows_multiple_non_overlapping() {
    // Two non-overlapping jumps should share lane 0
    let lines = vec![
        make_line(0x1000, "je", Some(0x1002)),
        make_line(0x1001, "nop", None),
        make_line(0x1002, "nop", None),
        make_line(0x1003, "je", Some(0x1005)),
        make_line(0x1004, "nop", None),
        make_line(0x1005, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 2);
    // Both should fit in lane 0 since they don't overlap
    assert_eq!(arrows[0].lane, 0);
    assert_eq!(arrows[1].lane, 0);
}

#[test]
fn test_branch_arrows_overlapping_use_different_lanes() {
    // Two overlapping jumps need different lanes
    let lines = vec![
        make_line(0x1000, "je", Some(0x1004)),  // spans 0..4
        make_line(0x1001, "je", Some(0x1003)),  // spans 1..3 (nested inside)
        make_line(0x1002, "nop", None),
        make_line(0x1003, "nop", None),
        make_line(0x1004, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 2);
    // They should be in different lanes
    let lanes: std::collections::HashSet<usize> = arrows.iter().map(|a| a.lane).collect();
    assert_eq!(lanes.len(), 2);
}

#[test]
fn test_render_arrow_column_empty() {
    let result = render_arrow_column(0, &[], 0);
    assert_eq!(result, "");
}

#[test]
fn test_render_arrow_column_source() {
    // Simple forward jump from line 0 to line 2
    let lines = vec![
        make_line(0x1000, "jmp", Some(0x1002)),
        make_line(0x1001, "nop", None),
        make_line(0x1002, "nop", None),
    ];
    let arrows = compute_branch_arrows(&lines);
    let num_lanes = arrows.iter().map(|a| a.lane + 1).max().unwrap_or(0);

    // Line 0 is source (forward jump = top-left corner ┌)
    let col0 = render_arrow_column(0, &arrows, num_lanes);
    assert!(col0.contains('┌'), "Source of forward jump should have ┌: {:?}", col0);

    // Line 1 is middle (vertical line │)
    let col1 = render_arrow_column(1, &arrows, num_lanes);
    assert!(col1.contains('│'), "Middle of arrow should have │: {:?}", col1);

    // Line 2 is target (arrow → and └)
    let col2 = render_arrow_column(2, &arrows, num_lanes);
    assert!(col2.contains('└'), "Target of forward jump should have └: {:?}", col2);
    assert!(col2.contains('→'), "Target should have arrow head →: {:?}", col2);
}

#[test]
fn test_branch_arrows_with_real_x86() {
    // Build a small code sequence with a real conditional jump
    // je +2 (0x74 0x02) = jump forward 2 bytes from end of this insn
    // nop nop (2 bytes to skip)
    // nop (target)
    let bytes = vec![
        0x74, 0x02, // je +2 (target = offset 4)
        0x90,       // nop (skipped)
        0x90,       // nop (skipped)
        0x90,       // nop (target of je)
    ];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    let lines = disasm.disassemble_range(0, 10);

    // Verify the je has branch_target set
    assert_eq!(lines[0].mnemonic, "je");
    assert!(lines[0].branch_target.is_some(), "je should have branch_target");
    let target = lines[0].branch_target.unwrap();
    assert_eq!(target, 0x1004);

    // Compute arrows
    let arrows = compute_branch_arrows(&lines);
    assert_eq!(arrows.len(), 1);
    assert_eq!(arrows[0].from_line, 0);
    // Target should be the last nop (at 0x1004 = line index 3)
    assert_eq!(lines[arrows[0].to_line].address, 0x1004);
}

#[test]
fn test_instruction_size_at() {
    // nop = 1 byte, mov rbp,rsp = 3 bytes
    let bytes = vec![0x90, 0x48, 0x89, 0xe5];
    let disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    assert_eq!(disasm.instruction_size_at(0), Some(1)); // nop
    assert_eq!(disasm.instruction_size_at(1), Some(3)); // mov rbp, rsp
    assert_eq!(disasm.instruction_size_at(100), None);  // out of bounds
}

#[test]
fn test_align_to_instruction() {
    // push rbp (0x55) = 1 byte, mov rbp,rsp (0x48 0x89 0xe5) = 3 bytes, nop = 1 byte
    let bytes = vec![0x55, 0x48, 0x89, 0xe5, 0x90];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    disasm.add_symbol("test_func".to_string(), 0x1000, 5);

    // Offset 0 is valid instruction boundary
    assert_eq!(disasm.align_to_instruction(0), 0);
    // Offset 1 is valid instruction boundary (mov rbp, rsp)
    assert_eq!(disasm.align_to_instruction(1), 1);
    // Offset 2 is mid-instruction — should align back to 1
    assert_eq!(disasm.align_to_instruction(2), 1);
    // Offset 3 is mid-instruction — should align back to 1
    assert_eq!(disasm.align_to_instruction(3), 1);
    // Offset 4 is valid (nop)
    assert_eq!(disasm.align_to_instruction(4), 4);
}

#[test]
fn test_rip_relative_branch_target() {
    // call qword ptr [rip + 0x10] — RIP-relative indirect call
    // This is encoded as FF 15 10 00 00 00
    let bytes = vec![
        0xFF, 0x15, 0x10, 0x00, 0x00, 0x00, // call [rip+0x10]
        0x90, // nop
    ];
    let mut disasm = Disassembler::from_bytes(bytes, DisasmArch::X86_64, 0x1000);
    // The RIP-relative target: insn_addr(0x1000) + insn_size(6) + 0x10 = 0x1016
    disasm.add_symbol("target_func".to_string(), 0x1016, 0);

    let lines = disasm.disassemble_range(0, 2);
    assert_eq!(lines[0].mnemonic, "call");
    // branch_target should be resolved via RIP-relative
    assert!(lines[0].branch_target.is_some(), "RIP-relative call should have branch_target");
    assert_eq!(lines[0].branch_target.unwrap(), 0x1016);
}
