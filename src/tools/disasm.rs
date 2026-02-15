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
    pub comment: Option<String>,
    pub symbol: Option<String>,
    pub is_branch_target: bool,
    /// Target address if this instruction is a branch/jump (not call)
    pub branch_target: Option<u64>,
}

/// A branch arrow connecting a source instruction to a target within the visible range.
#[derive(Debug, Clone)]
pub struct BranchArrow {
    /// Index of the source instruction in the visible lines
    pub from_line: usize,
    /// Index of the target instruction in the visible lines
    pub to_line: usize,
    /// Lane assignment (0 = closest to instructions, higher = further left)
    pub lane: usize,
    /// Whether the jump goes upward (true) or downward (false)
    pub goes_up: bool,
}

/// Compute branch arrows for a set of disassembled lines.
/// Only includes arrows where both source and target are visible.
pub fn compute_branch_arrows(lines: &[DisasmLine]) -> Vec<BranchArrow> {
    // Build address → line index map
    let addr_to_idx: std::collections::HashMap<u64, usize> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| (l.address, i))
        .collect();

    // Collect arrows where both endpoints are visible
    let mut arrows: Vec<(usize, usize)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let mn = line.mnemonic.as_str();
        // Only draw arrows for jumps, not calls (calls go to different functions)
        if mn.starts_with('j') || (mn.starts_with('b') && mn != "bswap") {
            if let Some(target) = line.branch_target {
                if let Some(&target_idx) = addr_to_idx.get(&target) {
                    if target_idx != i {
                        arrows.push((i, target_idx));
                    }
                }
            }
        }
    }

    // Sort by span length (shorter arrows get inner lanes, closer to code)
    arrows.sort_by_key(|&(from, to)| {
        let min = from.min(to);
        let max = from.max(to);
        max - min
    });

    // Assign lanes - track which rows each lane occupies
    let mut lane_occupancy: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut result = Vec::new();

    for (from, to) in arrows {
        let min_row = from.min(to);
        let max_row = from.max(to);
        let goes_up = to < from;

        // Find first lane that doesn't overlap
        let mut lane = 0;
        'lane_search: loop {
            if lane >= lane_occupancy.len() {
                lane_occupancy.push(Vec::new());
            }
            for &(occ_min, occ_max) in &lane_occupancy[lane] {
                // Check overlap (ranges intersect if they share any row)
                if min_row <= occ_max && max_row >= occ_min {
                    lane += 1;
                    continue 'lane_search;
                }
            }
            break;
        }

        if lane >= lane_occupancy.len() {
            lane_occupancy.push(Vec::new());
        }
        lane_occupancy[lane].push((min_row, max_row));

        result.push(BranchArrow {
            from_line: from,
            to_line: to,
            lane,
            goes_up,
        });
    }

    result
}

/// Render the arrow column for a specific line index.
/// Returns a string of fixed width (num_lanes * 2 chars wide) using box-drawing characters.
pub fn render_arrow_column(line_idx: usize, arrows: &[BranchArrow], num_lanes: usize) -> String {
    if num_lanes == 0 {
        return String::new();
    }

    // For each lane, determine what character to draw at this row
    // Lanes are drawn right-to-left: lane 0 is rightmost (closest to code)
    let mut chars: Vec<char> = vec![' '; num_lanes];

    for arrow in arrows {
        let min_row = arrow.from_line.min(arrow.to_line);
        let max_row = arrow.from_line.max(arrow.to_line);
        let lane = arrow.lane;

        if lane >= num_lanes {
            continue;
        }

        // The lane index in our chars array (rightmost = lane 0)
        let col = num_lanes - 1 - lane;

        if line_idx == arrow.from_line {
            // Source of the branch — corner going toward target
            if arrow.goes_up {
                chars[col] = '└'; // bottom-left corner, going up
            } else {
                chars[col] = '┌'; // top-left corner, going down
            }
        } else if line_idx == arrow.to_line {
            // Target of the branch — arrow pointing right
            if arrow.goes_up {
                chars[col] = '┌'; // top-left corner with arrow
            } else {
                chars[col] = '└'; // bottom-left corner with arrow
            }
        } else if line_idx > min_row && line_idx < max_row {
            // In between — vertical line
            chars[col] = '│';
        }
    }

    // Add arrow head indicator: find if this line is a target
    let is_target = arrows.iter().any(|a| a.to_line == line_idx);

    let mut result: String = chars.iter().collect();
    if is_target {
        result.push('→');
    } else {
        result.push(' ');
    }
    result
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

                let image_base = pe.image_base as u64;

                // Collect export symbols
                let mut symbols = Vec::new();
                for export in &pe.exports {
                    if let Some(name) = export.name {
                        symbols.push(Symbol {
                            name: name.to_string(),
                            address: image_base + export.rva as u64,
                            size: 0,
                        });
                    }
                }

                // Parse COFF symbol table from PE header
                Self::parse_pe_coff_symbols(&data, image_base, &pe.sections, &mut symbols);

                // Try loading companion PDB file
                if let Some(pdb_path) = Self::find_pdb_path(path, &data) {
                    if let Err(e) = Self::load_pdb_symbols(&pdb_path, image_base, &mut symbols) {
                        eprintln!("PDB loading note: {}", e);
                    }
                }

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

    /// Parse COFF symbol table embedded in a PE file.
    fn parse_pe_coff_symbols(
        data: &[u8],
        image_base: u64,
        sections: &[goblin::pe::section_table::SectionTable],
        symbols: &mut Vec<Symbol>,
    ) {
        // PE header at offset stored at 0x3C
        if data.len() < 0x40 {
            return;
        }
        let pe_offset = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
        if pe_offset + 24 > data.len() {
            return;
        }

        // COFF header starts at pe_offset + 4 (after "PE\0\0" signature)
        let coff_start = pe_offset + 4;
        if coff_start + 20 > data.len() {
            return;
        }

        let num_symbols = u32::from_le_bytes([
            data[coff_start + 12],
            data[coff_start + 13],
            data[coff_start + 14],
            data[coff_start + 15],
        ]) as usize;

        let sym_table_offset = u32::from_le_bytes([
            data[coff_start + 8],
            data[coff_start + 9],
            data[coff_start + 10],
            data[coff_start + 11],
        ]) as usize;

        if num_symbols == 0 || sym_table_offset == 0 {
            return;
        }

        // String table is right after symbol table (each symbol entry is 18 bytes)
        let str_table_offset = sym_table_offset + num_symbols * 18;

        let mut i = 0;
        while i < num_symbols {
            let entry_offset = sym_table_offset + i * 18;
            if entry_offset + 18 > data.len() {
                break;
            }

            let section_number = i16::from_le_bytes([
                data[entry_offset + 12],
                data[entry_offset + 13],
            ]);
            let storage_class = data[entry_offset + 16];
            let aux_count = data[entry_offset + 17] as usize;
            let value = u32::from_le_bytes([
                data[entry_offset + 8],
                data[entry_offset + 9],
                data[entry_offset + 10],
                data[entry_offset + 11],
            ]);

            // Only include function symbols (storage class 2=external, 3=static) in code sections
            if (storage_class == 2 || storage_class == 3) && section_number > 0 {
                // Get symbol name
                let name = if data[entry_offset] == 0
                    && data[entry_offset + 1] == 0
                    && data[entry_offset + 2] == 0
                    && data[entry_offset + 3] == 0
                {
                    // Long name: offset into string table
                    let str_offset = u32::from_le_bytes([
                        data[entry_offset + 4],
                        data[entry_offset + 5],
                        data[entry_offset + 6],
                        data[entry_offset + 7],
                    ]) as usize;
                    let abs_offset = str_table_offset + str_offset;
                    if abs_offset < data.len() {
                        let end = data[abs_offset..]
                            .iter()
                            .position(|&b| b == 0)
                            .unwrap_or(0);
                        String::from_utf8_lossy(&data[abs_offset..abs_offset + end]).to_string()
                    } else {
                        i += 1 + aux_count;
                        continue;
                    }
                } else {
                    // Short name: inline 8-byte field
                    let end = data[entry_offset..entry_offset + 8]
                        .iter()
                        .position(|&b| b == 0)
                        .unwrap_or(8);
                    String::from_utf8_lossy(&data[entry_offset..entry_offset + end]).to_string()
                };

                if !name.is_empty() && !name.starts_with('.') {
                    // Convert section-relative value to virtual address
                    let section_idx = (section_number - 1) as usize;
                    let vaddr = if section_idx < sections.len() {
                        image_base + sections[section_idx].virtual_address as u64 + value as u64
                    } else {
                        image_base + value as u64
                    };

                    // Avoid duplicates
                    if !symbols.iter().any(|s| s.address == vaddr && s.name == name) {
                        symbols.push(Symbol {
                            name,
                            address: vaddr,
                            size: 0,
                        });
                    }
                }
            }

            i += 1 + aux_count;
        }
    }

    /// Find companion PDB path from PE debug directory or by filename convention.
    fn find_pdb_path(exe_path: &Path, data: &[u8]) -> Option<std::path::PathBuf> {
        // Try to extract PDB path from PE debug directory (CodeView entry)
        if let Some(pdb_name) = Self::extract_pdb_path_from_pe(data) {
            // Try the embedded absolute path first
            let embedded = Path::new(&pdb_name);
            if embedded.exists() {
                return Some(embedded.to_path_buf());
            }
            // Try relative to the exe directory
            if let Some(dir) = exe_path.parent() {
                let filename = Path::new(&pdb_name)
                    .file_name()
                    .unwrap_or_default();
                let relative = dir.join(filename);
                if relative.exists() {
                    return Some(relative);
                }
            }
        }

        // Fallback: try same name with .pdb extension
        if let Some(dir) = exe_path.parent() {
            if let Some(stem) = exe_path.file_stem() {
                let pdb_path = dir.join(format!("{}.pdb", stem.to_string_lossy()));
                if pdb_path.exists() {
                    return Some(pdb_path);
                }
            }
        }

        None
    }

    /// Extract PDB file path from PE CodeView debug directory entry.
    fn extract_pdb_path_from_pe(data: &[u8]) -> Option<String> {
        if data.len() < 0x40 {
            return None;
        }
        let pe_offset = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;

        // Determine optional header size
        let coff_start = pe_offset + 4;
        if coff_start + 20 > data.len() {
            return None;
        }
        let opt_header_size = u16::from_le_bytes([
            data[coff_start + 16],
            data[coff_start + 17],
        ]) as usize;

        let opt_start = coff_start + 20;
        if opt_start + opt_header_size > data.len() {
            return None;
        }

        // Check PE32 vs PE32+
        let magic = u16::from_le_bytes([data[opt_start], data[opt_start + 1]]);
        let debug_dir_entry_offset = match magic {
            0x10b => opt_start + 144, // PE32: data dir at offset 96, debug is entry 6
            0x20b => opt_start + 160, // PE32+: data dir at offset 112, debug is entry 6
            _ => return None,
        };

        if debug_dir_entry_offset + 8 > data.len() {
            return None;
        }

        let debug_rva = u32::from_le_bytes([
            data[debug_dir_entry_offset],
            data[debug_dir_entry_offset + 1],
            data[debug_dir_entry_offset + 2],
            data[debug_dir_entry_offset + 3],
        ]);
        let debug_size = u32::from_le_bytes([
            data[debug_dir_entry_offset + 4],
            data[debug_dir_entry_offset + 5],
            data[debug_dir_entry_offset + 6],
            data[debug_dir_entry_offset + 7],
        ]);

        if debug_rva == 0 || debug_size == 0 {
            return None;
        }

        // We need to convert RVA to file offset using sections
        // Re-parse sections quickly
        let num_sections = u16::from_le_bytes([
            data[coff_start + 2],
            data[coff_start + 3],
        ]) as usize;

        let sections_start = opt_start + opt_header_size;

        let rva_to_offset = |rva: u32| -> Option<usize> {
            for i in 0..num_sections {
                let sec_off = sections_start + i * 40;
                if sec_off + 40 > data.len() {
                    break;
                }
                let sec_va = u32::from_le_bytes([
                    data[sec_off + 12], data[sec_off + 13],
                    data[sec_off + 14], data[sec_off + 15],
                ]);
                let sec_raw_size = u32::from_le_bytes([
                    data[sec_off + 16], data[sec_off + 17],
                    data[sec_off + 18], data[sec_off + 19],
                ]);
                let sec_raw_ptr = u32::from_le_bytes([
                    data[sec_off + 20], data[sec_off + 21],
                    data[sec_off + 22], data[sec_off + 23],
                ]);
                if rva >= sec_va && rva < sec_va + sec_raw_size {
                    return Some((sec_raw_ptr + (rva - sec_va)) as usize);
                }
            }
            None
        };

        let debug_file_offset = rva_to_offset(debug_rva)?;

        // Parse debug directory entries (each 28 bytes)
        let num_entries = debug_size as usize / 28;
        for i in 0..num_entries {
            let entry_off = debug_file_offset + i * 28;
            if entry_off + 28 > data.len() {
                break;
            }
            let entry_type = u32::from_le_bytes([
                data[entry_off + 12], data[entry_off + 13],
                data[entry_off + 14], data[entry_off + 15],
            ]);

            // Type 2 = IMAGE_DEBUG_TYPE_CODEVIEW
            if entry_type == 2 {
                let cv_offset = u32::from_le_bytes([
                    data[entry_off + 24], data[entry_off + 25],
                    data[entry_off + 26], data[entry_off + 27],
                ]) as usize;

                if cv_offset + 24 > data.len() {
                    continue;
                }

                // Check for RSDS signature
                if &data[cv_offset..cv_offset + 4] == b"RSDS" {
                    // PDB path starts at offset 24 (after signature + GUID + age)
                    let path_start = cv_offset + 24;
                    let path_end = data[path_start..]
                        .iter()
                        .position(|&b| b == 0)
                        .map(|p| path_start + p)
                        .unwrap_or(data.len().min(path_start + 260));
                    return Some(
                        String::from_utf8_lossy(&data[path_start..path_end]).to_string(),
                    );
                }
            }
        }

        None
    }

    /// Load symbols from a PDB file.
    fn load_pdb_symbols(
        pdb_path: &Path,
        image_base: u64,
        symbols: &mut Vec<Symbol>,
    ) -> Result<(), String> {
        use pdb::FallibleIterator;

        let file = std::fs::File::open(pdb_path)
            .map_err(|e| format!("Failed to open PDB: {}", e))?;
        let mut pdb = pdb::PDB::open(file)
            .map_err(|e| format!("Failed to parse PDB: {}", e))?;

        // Get the global symbols
        let symbol_table = pdb.global_symbols()
            .map_err(|e| format!("Failed to read PDB global symbols: {}", e))?;
        let address_map = pdb.address_map()
            .map_err(|e| format!("Failed to read PDB address map: {}", e))?;

        let mut iter = symbol_table.iter();
        while let Some(symbol) = iter.next().map_err(|e| format!("PDB symbol iter error: {}", e))? {
            if let Ok(pdb::SymbolData::Public(pub_sym)) = symbol.parse() {
                if let Some(rva) = pub_sym.offset.to_rva(&address_map) {
                    let name = pub_sym.name.to_string().to_string();
                    if !name.is_empty() {
                        let addr = image_base + rva.0 as u64;
                        if !symbols.iter().any(|s| s.address == addr && s.name == name) {
                            symbols.push(Symbol {
                                name,
                                address: addr,
                                size: 0,
                            });
                        }
                    }
                }
            }
        }

        // Also try to get procedure symbols (functions with size info)
        if let Ok(dbi) = pdb.debug_information() {
            if let Ok(mut modules) = dbi.modules() {
                while let Ok(Some(module)) = modules.next() {
                    if let Ok(Some(module_info)) = pdb.module_info(&module) {
                        if let Ok(symbols_iter) = module_info.symbols() {
                            let mut sym_iter = symbols_iter;
                            while let Ok(Some(symbol)) = sym_iter.next() {
                                if let Ok(pdb::SymbolData::Procedure(proc)) = symbol.parse() {
                                    if let Some(rva) = proc.offset.to_rva(&address_map) {
                                        let name = proc.name.to_string().to_string();
                                        if !name.is_empty() {
                                            let addr = image_base + rva.0 as u64;
                                            if !symbols.iter().any(|s| s.address == addr) {
                                                symbols.push(Symbol {
                                                    name,
                                                    address: addr,
                                                    size: proc.len as u64,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
        // Prefer exact match (case-insensitive), fall back to substring
        self.symbols.iter().find(|s| s.name.to_lowercase() == lower)
            .or_else(|| self.symbols.iter().find(|s| s.name.to_lowercase().contains(&lower)))
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

    pub fn add_symbol(&mut self, name: String, address: u64, size: u64) {
        self.symbols.push(Symbol {
            name,
            address,
            size,
        });
    }

    /// Write bytes at the given offset, updating the in-memory binary.
    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) {
        for (i, &b) in data.iter().enumerate() {
            if offset + i < self.bytes.len() {
                self.bytes[offset + i] = b;
            }
        }
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
        let insn_size = insn.bytes().len() as u64;
        let symbol = self.symbol_at_address(addr).and_then(|s| {
            if s.address == addr {
                Some(s.name.clone())
            } else {
                None
            }
        });

        let operands_raw = insn.op_str().unwrap_or("").to_string();
        let mnemonic = insn.mnemonic().unwrap_or("").to_string();

        // Parse branch target address for arrow rendering
        let branch_target = if mnemonic.starts_with('j')
            || mnemonic == "call"
            || (mnemonic.starts_with('b') && mnemonic != "bswap")
        {
            parse_hex_address(&operands_raw)
        } else {
            None
        };

        // Resolve addresses in operands to symbolic names.
        // The resolved form replaces the address with the symbol name in the operands field,
        // and the raw operand goes to the comment.
        let (operands, comment) =
            self.resolve_operands(&mnemonic, &operands_raw, addr, insn_size);

        DisasmLine {
            address: addr,
            bytes: insn.bytes().to_vec(),
            mnemonic,
            operands,
            comment,
            symbol,
            is_branch_target: branch_targets.contains(&addr),
            branch_target,
        }
    }

    /// Resolve addresses in operands to symbolic names.
    /// Returns (resolved_operands, raw_comment).
    /// When a symbol is found, the operands contain the symbolic form and
    /// the comment contains the raw operand string.
    fn resolve_operands(
        &self,
        mnemonic: &str,
        operands: &str,
        insn_addr: u64,
        insn_size: u64,
    ) -> (String, Option<String>) {
        // 1. Direct branch/call target: "call 0x140001234" or "jmp 0x140001234"
        if mnemonic.starts_with('j') || mnemonic == "call" || mnemonic.starts_with('b') {
            if let Some(target) = parse_hex_address(operands) {
                if let Some(sym_name) = self.format_symbol_ref(target) {
                    return (sym_name, Some(operands.to_string()));
                }
            }
        }

        // 2. RIP-relative addressing: [rip + 0x1234] or [rip - 0x1234]
        if let Some(target) = self.resolve_rip_relative(operands, insn_addr, insn_size) {
            if let Some(sym_name) = self.format_symbol_ref(target) {
                // Replace the [rip + ...] portion with [sym_name]
                let resolved = self.substitute_rip_with_symbol(operands, &sym_name);
                return (resolved, Some(operands.to_string()));
            }
        }

        // 3. Absolute address in operands
        if let Some((target, addr_str)) = self.find_absolute_address_with_text(operands) {
            if let Some(sym_name) = self.format_symbol_ref(target) {
                let resolved = operands.replace(&addr_str, &sym_name);
                return (resolved, Some(operands.to_string()));
            }
        }

        // No resolution — operands unchanged, no comment
        (operands.to_string(), None)
    }

    /// Replace [rip + 0x...] in operand string with [symbol_name].
    fn substitute_rip_with_symbol(&self, operands: &str, sym_name: &str) -> String {
        let lower = operands.to_lowercase();
        if let Some(rip_pos) = lower.find("rip") {
            if let Some(bracket_start) = lower[..rip_pos].rfind('[') {
                if let Some(bracket_end_rel) = lower[rip_pos..].find(']') {
                    let bracket_end = rip_pos + bracket_end_rel;
                    let mut result = String::new();
                    result.push_str(&operands[..bracket_start + 1]);
                    result.push_str(sym_name);
                    result.push_str(&operands[bracket_end..]);
                    return result;
                }
            }
        }
        operands.to_string()
    }

    /// Parse RIP-relative addressing and compute the target address.
    fn resolve_rip_relative(&self, operands: &str, insn_addr: u64, insn_size: u64) -> Option<u64> {
        let lower = operands.to_lowercase();
        let rip_pos = lower.find("rip")?;

        // Find the bracket context around rip
        let bracket_start = lower[..rip_pos].rfind('[')?;
        let bracket_end = lower[rip_pos..].find(']').map(|i| rip_pos + i)?;
        let inner = &lower[bracket_start + 1..bracket_end].trim();

        // Parse "rip + 0x1234" or "rip - 0x1234"
        let after_rip = inner[3..].trim(); // skip "rip"
        if after_rip.is_empty() {
            // [rip] with no displacement
            return Some(insn_addr + insn_size);
        }

        let (sign, hex_part) = if let Some(rest) = after_rip.strip_prefix('+') {
            (1i64, rest.trim())
        } else if let Some(rest) = after_rip.strip_prefix('-') {
            (-1i64, rest.trim())
        } else {
            return None;
        };

        let disp = if let Some(h) = hex_part.strip_prefix("0x") {
            i64::from_str_radix(h, 16).ok()?
        } else {
            hex_part.parse::<i64>().ok()?
        };

        let target = (insn_addr + insn_size) as i64 + sign * disp;
        Some(target as u64)
    }

    /// Find a bare hex address in operands that might be a symbol reference.
    /// Returns the address and the original text matched.
    fn find_absolute_address_with_text(&self, operands: &str) -> Option<(u64, String)> {
        // Match 0x followed by 8+ hex digits (likely an absolute address)
        // We need to find the exact substring to replace
        let mut i = 0;
        let bytes = operands.as_bytes();
        while i + 10 <= bytes.len() {
            if i + 2 <= bytes.len()
                && bytes[i] == b'0'
                && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
            {
                let start = i;
                i += 2;
                let hex_start = i;
                while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let hex_len = i - hex_start;
                if hex_len >= 8 {
                    let hex_str = &operands[hex_start..i];
                    if let Ok(addr) = u64::from_str_radix(hex_str, 16) {
                        if self.symbol_at_address(addr).is_some()
                            || self.nearest_symbol_before(addr).is_some()
                        {
                            return Some((addr, operands[start..i].to_string()));
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        None
    }

    /// Format an address as a symbol reference like "function1+0x3c" or just "function1".
    fn format_symbol_ref(&self, target: u64) -> Option<String> {
        // Exact match
        if let Some(sym) = self.symbol_at_address(target) {
            if sym.address == target {
                return Some(sym.name.clone());
            }
            // Within symbol bounds
            let offset = target - sym.address;
            return Some(format!("{}+0x{:x}", sym.name, offset));
        }

        // Find nearest symbol before this address
        if let Some(sym) = self.nearest_symbol_before(target) {
            let offset = target - sym.address;
            // Only annotate if within reasonable distance (64KB)
            if offset < 0x10000 {
                return Some(format!("{}+0x{:x}", sym.name, offset));
            }
        }

        None
    }

    /// Find the nearest symbol at or before the given address.
    fn nearest_symbol_before(&self, addr: u64) -> Option<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.address <= addr)
            .max_by_key(|s| s.address)
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
