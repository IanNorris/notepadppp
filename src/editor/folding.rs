use std::collections::HashSet;

/// A foldable region in the document
#[derive(Debug, Clone)]
pub struct FoldRegion {
    /// Start line (0-based) — the line containing the opening bracket/keyword
    pub start_line: usize,
    /// End line (0-based) — the line containing the closing bracket/keyword
    pub end_line: usize,
    /// Indentation level
    pub indent_level: usize,
}

/// Manages code folding state for a document
#[derive(Debug, Default)]
pub struct FoldManager {
    /// Set of folded start lines
    folded: HashSet<usize>,
    /// Cached fold regions (recomputed when text changes)
    regions: Vec<FoldRegion>,
}

impl FoldManager {
    pub fn new() -> Self {
        Self {
            folded: HashSet::new(),
            regions: Vec::new(),
        }
    }

    /// Detect fold regions from text content using bracket matching.
    pub fn detect_regions(&mut self, text: &str) {
        self.regions.clear();
        let lines: Vec<&str> = text.lines().collect();
        let mut stack: Vec<(usize, usize)> = Vec::new(); // (line_index, indent_level)

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let indent_level = line.len() - line.trim_start().len();

            // Check if line ends with '{' (ignoring trailing whitespace and comments)
            let code_part = if let Some(pos) = trimmed.find("//") {
                trimmed[..pos].trim()
            } else {
                trimmed
            };

            if code_part.ends_with('{') {
                stack.push((i, indent_level));
            }

            // Check if line is (or contains) a closing '}'
            if trimmed.starts_with('}') || trimmed == "}" {
                if let Some((start_line, indent)) = stack.pop() {
                    if i > start_line {
                        self.regions.push(FoldRegion {
                            start_line,
                            end_line: i,
                            indent_level: indent,
                        });
                    }
                }
            }
        }

        // Sort regions by start_line for consistent access
        self.regions.sort_by_key(|r| r.start_line);

        // Remove any folds that no longer correspond to valid regions
        let valid_starts: HashSet<usize> = self.regions.iter().map(|r| r.start_line).collect();
        self.folded.retain(|line| valid_starts.contains(line));
    }

    /// Toggle fold at a specific line
    pub fn toggle_fold(&mut self, line: usize) {
        if self.is_fold_point(line) {
            if self.folded.contains(&line) {
                self.folded.remove(&line);
            } else {
                self.folded.insert(line);
            }
        }
    }

    /// Fold at a specific line
    pub fn fold(&mut self, line: usize) {
        if self.is_fold_point(line) {
            self.folded.insert(line);
        }
    }

    /// Unfold at a specific line
    pub fn unfold(&mut self, line: usize) {
        self.folded.remove(&line);
    }

    /// Fold all regions
    pub fn fold_all(&mut self) {
        for region in &self.regions {
            self.folded.insert(region.start_line);
        }
    }

    /// Unfold all regions
    pub fn unfold_all(&mut self) {
        self.folded.clear();
    }

    /// Fold all regions at a specific indent level
    pub fn fold_level(&mut self, level: usize) {
        for region in &self.regions {
            if region.indent_level <= level {
                self.folded.insert(region.start_line);
            }
        }
    }

    /// Check if a line is folded (is a fold start and is collapsed)
    pub fn is_folded(&self, line: usize) -> bool {
        self.folded.contains(&line)
    }

    /// Check if a line is a fold start (can be folded)
    pub fn is_fold_point(&self, line: usize) -> bool {
        self.regions.iter().any(|r| r.start_line == line)
    }

    /// Check if a line is hidden (inside a folded region)
    pub fn is_hidden(&self, line: usize) -> bool {
        for region in &self.regions {
            if self.folded.contains(&region.start_line)
                && line > region.start_line
                && line <= region.end_line
            {
                return true;
            }
        }
        false
    }

    /// Get the fold region starting at a specific line
    pub fn get_region(&self, line: usize) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.start_line == line)
    }

    /// Get all fold regions
    pub fn regions(&self) -> &[FoldRegion] {
        &self.regions
    }

    /// Get the number of hidden lines in a folded region starting at the given line
    pub fn hidden_line_count(&self, line: usize) -> usize {
        if let Some(region) = self.get_region(line) {
            if self.folded.contains(&line) {
                return region.end_line - region.start_line;
            }
        }
        0
    }

    /// Given a visible line index, return the actual document line index
    /// (accounting for folded/hidden lines)
    pub fn visible_to_doc_line(&self, visible_line: usize, total_lines: usize) -> usize {
        let mut vis = 0;
        let mut doc = 0;
        while doc < total_lines {
            if self.is_hidden(doc) {
                doc += 1;
                continue;
            }
            if vis == visible_line {
                return doc;
            }
            vis += 1;
            doc += 1;
        }
        doc.saturating_sub(1).min(total_lines.saturating_sub(1))
    }

    /// Given a document line, return the visible line index
    pub fn doc_to_visible_line(&self, doc_line: usize) -> usize {
        let mut vis = 0;
        for line in 0..doc_line {
            if !self.is_hidden(line) {
                vis += 1;
            }
        }
        vis
    }

    /// Get total number of visible lines
    pub fn visible_line_count(&self, total_lines: usize) -> usize {
        let mut count = 0;
        for line in 0..total_lines {
            if !self.is_hidden(line) {
                count += 1;
            }
        }
        count
    }

    /// Clear all folds
    pub fn clear(&mut self) {
        self.folded.clear();
        self.regions.clear();
    }
}
