/// Persistent marked ranges for "Mark All" highlighting.
#[derive(Default, Clone, Debug)]
pub struct MarkManager {
    marks: Vec<(usize, usize)>,
}

impl MarkManager {
    pub fn mark_all(&mut self, ranges: Vec<(usize, usize)>) {
        self.marks.extend(ranges);
    }

    pub fn clear(&mut self) {
        self.marks.clear();
    }

    pub fn marks(&self) -> &[(usize, usize)] {
        &self.marks
    }

    pub fn count(&self) -> usize {
        self.marks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }

    /// Check if a byte offset falls within any marked range.
    pub fn contains(&self, offset: usize) -> bool {
        self.marks.iter().any(|&(s, e)| offset >= s && offset < e)
    }
}
