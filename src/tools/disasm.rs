use std::path::Path;

use capstone::prelude::*;
use capstone::{Capstone, Insn};
use goblin::Object;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisasmArch {
    X86_32,
    X86_64,
    ARM,
    AArch64,
}

impl std::fmt::Display for DisasmArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisasmArch::X86_32 => write!(f, "x86-32"),
            DisasmArch::X86_64 => write!(f, "x86-64"),
            DisasmArch::ARM => write!(f, "ARM"),
            DisasmArch::AArch64 => write!(f, "AArch64"),
        }
    }
}

pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DisasmLine {
    pub address: u64,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    pub symbol: Option<String>,
    pub is_branch_target: bool,
}

pub struct Disassembler {
    bytes: Vec<u8>,
    base_address: u64,
    arch: DisasmArch,
    symbols: Vec<Symbol>,
}

impl Disassembler {
    pub fn from_bytes(bytes: Vec<u8>, arch: DisasmArch, base_address: u64) -> Self {
        Self {
            bytes,
            base_address,
            arch,
            symbols: Vec::new(),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let obj = Object::parse(&data).map_err(|e| format!("Failed to parse binary: {}", e))?;

        match obj {
            Object::Elf(elf) => {
                let arch = match elf.header.e_machine {
                    goblin::elf::header::EM_386 => DisasmArch::X86_32,
                    goblin::elf::header::EM_X86_64 => DisasmArch::X86_64,
                    goblin::elf::header::EM_ARM => DisasmArch::ARM,
                    goblin::elf::header::EM_AARCH64 => DisasmArch::AArch64,
                    m => return Err(format!("Unsupported ELF machine type: {}", m)),
                };

                let mut symbols = Vec::new();
                for sym in elf.syms.iter() {
                    if let Some(name) = elf.strtab.get_at(sym.st_name) {
                        if !name.is_empty() {
                            symbols.push(Symbol {
                                name: name.to_string(),
                                address: sym.st_value,
                                size: sym.st_size,
                            });
                        }
                    }
                }
                for sym in elf.dynsyms.iter() {
                    if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                        if !name.is_empty() {
                            symbols.push(Symbol {
                                name: name.to_string(),
                                address: sym.st_value,
                                size: sym.st_size,
                            });
                        }
                    }
                }

                // Find .text section offset and use its data
                let (text_bytes, base_address) = if let Some(text_sh) =
                    elf.section_headers.iter().find(|sh| {
                        elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("") == ".text"
                    }) {
                    let start = text_sh.sh_offset as usize;
                    let end = start + text_sh.sh_size as usize;
                    if end <= data.len() {
                        (data[start..end].to_vec(), text_sh.sh_addr)
                    } else {
                        (data.clone(), elf.header.e_entry)
                    }
                } else {
                    (data.to_vec(), elf.header.e_entry)
                };

                Ok(Self {
                    bytes: text_bytes,
                    base_address,
                    arch,
                    symbols,
                })
            }
            Object::PE(pe) => {
                let arch = if pe.is_64 {
                    DisasmArch::X86_64
                } else {
                    DisasmArch::X86_32
                };

                let mut symbols = Vec::new();
                for export in &pe.exports {
                    if let Some(name) = export.name {
                        symbols.push(Symbol {
                            name: name.to_string(),
                            address: export.rva as u64,
                            size: 0,
                        });
                    }
                }

                let image_base = pe.image_base as u64;
                let entry = image_base + pe.entry as u64;

                // Find .text section
                let (text_bytes, base_address) =
                    if let Some(text_sec) = pe.sections.iter().find(|s| {
                        let name = String::from_utf8_lossy(&s.name);
                        name.starts_with(".text")
                    }) {
                        let start = text_sec.pointer_to_raw_data as usize;
                        let size = text_sec.size_of_raw_data as usize;
                        if start + size <= data.len() {
                            (
                                data[start..start + size].to_vec(),
                                image_base + text_sec.virtual_address as u64,
                            )
                        } else {
                            (data.to_vec(), entry)
                        }
                    } else {
                        (data.to_vec(), entry)
                    };

                Ok(Self {
                    bytes: text_bytes,
                    base_address,
                    arch,
                    symbols,
                })
            }
            Object::Mach(mach) => {
                match mach {
                    goblin::mach::Mach::Binary(macho) => {
                        Self::from_macho(&macho, &data)
                    }
                    goblin::mach::Mach::Fat(fat) => {
                        // Use the first architecture from a fat binary
                        let arches = fat.arches().map_err(|e| format!("Fat Mach-O error: {}", e))?;
                        if let Some(arch) = arches.first() {
                            let bytes = &data[arch.offset as usize..(arch.offset + arch.size) as usize];
                            let macho = goblin::mach::MachO::parse(bytes, 0)
                                .map_err(|e| format!("Failed to parse Mach-O slice: {}", e))?;
                            Self::from_macho(&macho, bytes)
                        } else {
                            Err("No architectures found in fat Mach-O".to_string())
                        }
                    }
                }
            }
            _ => {
                // Raw binary — just use the bytes as-is
                Ok(Self {
                    bytes: data.to_vec(),
                    base_address: 0,
                    arch: DisasmArch::X86_64,
                    symbols: Vec::new(),
                })
            }
        }
    }

    fn from_macho(macho: &goblin::mach::MachO, data: &[u8]) -> Result<Self, String> {
        use goblin::mach::cputype::*;
        let arch = match macho.header.cputype() {
            CPU_TYPE_X86 => DisasmArch::X86_32,
            CPU_TYPE_X86_64 => DisasmArch::X86_64,
            CPU_TYPE_ARM => DisasmArch::ARM,
            CPU_TYPE_ARM64 => DisasmArch::AArch64,
            t => return Err(format!("Unsupported Mach-O CPU type: {}", t)),
        };

        let mut symbols = Vec::new();
        if let Some(ref syms) = macho.symbols {
            for sym in syms.iter() {
                if let Ok((name, nlist)) = sym {
                    if !name.is_empty() {
                        symbols.push(Symbol {
                            name: name.to_string(),
                            address: nlist.n_value,
                            size: 0,
                        });
                    }
                }
            }
        }

        let entry = macho.entry as u64;

        // Find __text section
        let (text_bytes, base_address) = 'find_text: {
            for seg in &macho.segments {
                for (sec, _) in seg.sections().unwrap_or_default() {
                    let name = std::str::from_utf8(&sec.sectname).unwrap_or("");
                    if name.starts_with("__text") {
                        let start = sec.offset as usize;
                        let size = sec.size as usize;
                        if start + size <= data.len() {
                            break 'find_text (data[start..start + size].to_vec(), sec.addr);
                        }
                    }
                }
            }
            (data.to_vec(), entry)
        };

        Ok(Self {
            bytes: text_bytes,
            base_address,
            arch,
            symbols,
        })
    }

    /// Disassemble `count` instructions starting from byte offset.
    pub fn disassemble_range(&self, offset: usize, count: usize) -> Vec<DisasmLine> {
        if offset >= self.bytes.len() {
            return Vec::new();
        }
        let cs = match self.make_capstone() {
            Ok(cs) => cs,
            Err(_) => return Vec::new(),
        };

        let slice = &self.bytes[offset..];
        let addr = self.offset_to_address(offset);

        let insns = match cs.disasm_count(slice, addr, count) {
            Ok(insns) => insns,
            Err(_) => return Vec::new(),
        };

        self.insns_to_lines(&insns)
    }

    /// Disassemble starting at a virtual address.
    pub fn disassemble_at_address(&self, address: u64, count: usize) -> Vec<DisasmLine> {
        match self.address_to_offset(address) {
            Some(off) => self.disassemble_range(off, count),
            None => Vec::new(),
        }
    }

    /// Find a symbol by name (case-insensitive substring match).
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        let lower = name.to_lowercase();
        self.symbols.iter().find(|s| s.name.to_lowercase().contains(&lower))
    }

    /// Find the symbol containing the given address.
    pub fn symbol_at_address(&self, addr: u64) -> Option<&Symbol> {
        self.symbols.iter().find(|s| {
            if s.size > 0 {
                addr >= s.address && addr < s.address + s.size
            } else {
                addr == s.address
            }
        })
    }

    pub fn set_base_address(&mut self, addr: u64) {
        self.base_address = addr;
    }

    pub fn set_arch(&mut self, arch: DisasmArch) {
        self.arch = arch;
    }

    pub fn address_to_offset(&self, addr: u64) -> Option<usize> {
        if addr >= self.base_address {
            let off = (addr - self.base_address) as usize;
            if off < self.bytes.len() {
                Some(off)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn offset_to_address(&self, offset: usize) -> u64 {
        self.base_address + offset as u64
    }

    pub fn base_address(&self) -> u64 {
        self.base_address
    }

    pub fn arch(&self) -> DisasmArch {
        self.arch
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn make_capstone(&self) -> Result<Capstone, capstone::Error> {
        match self.arch {
            DisasmArch::X86_32 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode32)
                .syntax(arch::x86::ArchSyntax::Intel)
                .detail(false)
                .build(),
            DisasmArch::X86_64 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .syntax(arch::x86::ArchSyntax::Intel)
                .detail(false)
                .build(),
            DisasmArch::ARM => Capstone::new()
                .arm()
                .mode(arch::arm::ArchMode::Arm)
                .detail(false)
                .build(),
            DisasmArch::AArch64 => Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .detail(false)
                .build(),
        }
    }

    fn insns_to_lines(&self, insns: &capstone::Instructions) -> Vec<DisasmLine> {
        // Collect branch target addresses to mark them
        let mut branch_targets: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for insn in insns.as_ref() {
            let mn = insn.mnemonic().unwrap_or("");
            if mn.starts_with('j') || mn.starts_with('b') || mn == "call" {
                if let Some(ops) = insn.op_str() {
                    if let Some(addr) = parse_hex_address(ops) {
                        branch_targets.insert(addr);
                    }
                }
            }
        }

        insns
            .as_ref()
            .iter()
            .map(|insn| self.insn_to_line(insn, &branch_targets))
            .collect()
    }

    fn insn_to_line(
        &self,
        insn: &Insn,
        branch_targets: &std::collections::HashSet<u64>,
    ) -> DisasmLine {
        let addr = insn.address();
        let symbol = self.symbol_at_address(addr).and_then(|s| {
            if s.address == addr {
                Some(s.name.clone())
            } else {
                None
            }
        });
        DisasmLine {
            address: addr,
            bytes: insn.bytes().to_vec(),
            mnemonic: insn.mnemonic().unwrap_or("").to_string(),
            operands: insn.op_str().unwrap_or("").to_string(),
            symbol,
            is_branch_target: branch_targets.contains(&addr),
        }
    }
}

fn parse_hex_address(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).ok()
    } else {
        u64::from_str_radix(s, 16).ok()
    }
}
