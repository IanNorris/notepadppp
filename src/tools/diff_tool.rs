/// Represents a single line in a diff result.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Same(String),
    Added(String),
    Removed(String),
    Changed { old: String, new: String },
}

/// Statistics about a diff result.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub same: usize,
}

/// The result of comparing two texts.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

/// Compute a line-by-line diff between two texts using the LCS algorithm.
pub fn diff_texts(left: &str, right: &str) -> DiffResult {
    let left_lines: Vec<&str> = split_lines(left);
    let right_lines: Vec<&str> = split_lines(right);

    let m = left_lines.len();
    let n = right_lines.len();

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if left_lines[i - 1] == right_lines[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to produce diff
    let mut lines = Vec::new();
    let mut i = m;
    let mut j = n;

    let mut temp = Vec::new();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && left_lines[i - 1] == right_lines[j - 1] {
            temp.push(DiffLine::Same(left_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            temp.push(DiffLine::Added(right_lines[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            temp.push(DiffLine::Removed(left_lines[i - 1].to_string()));
            i -= 1;
        }
    }

    temp.reverse();

    // Post-process: merge adjacent Removed+Added into Changed
    let mut idx = 0;
    while idx < temp.len() {
        if idx + 1 < temp.len() {
            if let (DiffLine::Removed(old), DiffLine::Added(new)) = (&temp[idx], &temp[idx + 1]) {
                lines.push(DiffLine::Changed {
                    old: old.clone(),
                    new: new.clone(),
                });
                idx += 2;
                continue;
            }
        }
        lines.push(temp[idx].clone());
        idx += 1;
    }

    let stats = compute_stats(&lines);
    DiffResult { lines, stats }
}

fn compute_stats(lines: &[DiffLine]) -> DiffStats {
    let mut stats = DiffStats {
        added: 0,
        removed: 0,
        changed: 0,
        same: 0,
    };
    for line in lines {
        match line {
            DiffLine::Same(_) => stats.same += 1,
            DiffLine::Added(_) => stats.added += 1,
            DiffLine::Removed(_) => stats.removed += 1,
            DiffLine::Changed { .. } => stats.changed += 1,
        }
    }
    stats
}

fn split_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.lines().collect()
}

/// Render a diff result in an egui UI with color-coded lines.
#[cfg(feature = "_ui")]
pub fn render_diff(ui: &mut egui::Ui, result: &DiffResult) {
    use egui::{Color32, RichText};

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(
            RichText::new(format!(
                "Stats: {} same, {} added, {} removed, {} changed",
                result.stats.same, result.stats.added, result.stats.removed, result.stats.changed
            ))
            .strong(),
        );
        ui.separator();

        for line in &result.lines {
            match line {
                DiffLine::Same(text) => {
                    ui.label(format!("  {text}"));
                }
                DiffLine::Added(text) => {
                    ui.label(RichText::new(format!("+ {text}")).color(Color32::from_rgb(80, 200, 80)));
                }
                DiffLine::Removed(text) => {
                    ui.label(RichText::new(format!("- {text}")).color(Color32::from_rgb(220, 80, 80)));
                }
                DiffLine::Changed { old, new } => {
                    ui.label(RichText::new(format!("- {old}")).color(Color32::from_rgb(220, 180, 60)));
                    ui.label(RichText::new(format!("+ {new}")).color(Color32::from_rgb(220, 180, 60)));
                }
            }
        }
    });
}
