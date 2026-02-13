use notepadppp::editor::folding::FoldManager;

const RUST_CODE: &str = "fn foo() {
    line1
    line2
}

fn bar() {
    if true {
        nested
    }
}";

#[test]
fn test_detect_regions_basic() {
    let mut fm = FoldManager::new();
    fm.detect_regions("fn main() {\n    hello\n}");
    assert_eq!(fm.regions().len(), 1);
    assert_eq!(fm.regions()[0].start_line, 0);
    assert_eq!(fm.regions()[0].end_line, 2);
}

#[test]
fn test_detect_regions_nested() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    // Should detect: foo(){}, bar(){}, and inner if{}
    assert_eq!(fm.regions().len(), 3);
    // Regions sorted by start_line
    assert_eq!(fm.regions()[0].start_line, 0); // fn foo
    assert_eq!(fm.regions()[0].end_line, 3);
    assert_eq!(fm.regions()[1].start_line, 5); // fn bar
    assert_eq!(fm.regions()[2].start_line, 6); // if true
}

#[test]
fn test_detect_regions_empty() {
    let mut fm = FoldManager::new();
    fm.detect_regions("no braces here\njust text\n");
    assert_eq!(fm.regions().len(), 0);
}

#[test]
fn test_fold_unfold_toggle() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    assert!(!fm.is_folded(0));
    fm.toggle_fold(0);
    assert!(fm.is_folded(0));
    fm.toggle_fold(0);
    assert!(!fm.is_folded(0));
}

#[test]
fn test_fold_all_unfold_all() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    fm.fold_all();
    for region in fm.regions() {
        assert!(fm.is_folded(region.start_line));
    }
    fm.unfold_all();
    for region in fm.regions() {
        assert!(!fm.is_folded(region.start_line));
    }
}

#[test]
fn test_is_hidden() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    fm.fold(0); // fold fn foo
    // Lines 1, 2, 3 (line1, line2, }) should be hidden
    assert!(!fm.is_hidden(0));
    assert!(fm.is_hidden(1));
    assert!(fm.is_hidden(2));
    assert!(fm.is_hidden(3));
    assert!(!fm.is_hidden(4));
}

#[test]
fn test_visible_to_doc_line() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    let total = RUST_CODE.lines().count();
    // No folds: visible line == doc line
    assert_eq!(fm.visible_to_doc_line(0, total), 0);
    assert_eq!(fm.visible_to_doc_line(5, total), 5);

    // Fold fn foo (lines 0-3), hides lines 1,2,3
    fm.fold(0);
    // visible 0 = doc 0 (fn foo - the fold header)
    assert_eq!(fm.visible_to_doc_line(0, total), 0);
    // visible 1 = doc 4 (empty line after foo)
    assert_eq!(fm.visible_to_doc_line(1, total), 4);
    // visible 2 = doc 5 (fn bar)
    assert_eq!(fm.visible_to_doc_line(2, total), 5);
}

#[test]
fn test_doc_to_visible_line() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    fm.fold(0);
    // doc line 0 -> visible 0
    assert_eq!(fm.doc_to_visible_line(0), 0);
    // doc line 4 -> visible 1 (lines 1,2,3 hidden)
    assert_eq!(fm.doc_to_visible_line(4), 1);
    // doc line 5 -> visible 2
    assert_eq!(fm.doc_to_visible_line(5), 2);
}

#[test]
fn test_visible_line_count() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    let total = RUST_CODE.lines().count();
    assert_eq!(fm.visible_line_count(total), total);

    fm.fold(0); // hides 3 lines (1,2,3)
    assert_eq!(fm.visible_line_count(total), total - 3);
}

#[test]
fn test_fold_level() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    // Level 0: fold regions with indent_level <= 0 (top-level functions)
    fm.fold_level(0);
    assert!(fm.is_folded(0)); // fn foo at indent 0
    assert!(fm.is_folded(5)); // fn bar at indent 0
}

#[test]
fn test_clear_folds() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    fm.fold_all();
    assert!(!fm.regions().is_empty());
    fm.clear();
    assert!(fm.regions().is_empty());
    assert!(!fm.is_folded(0));
}

#[test]
fn test_hidden_line_count() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    fm.fold(0);
    // fn foo spans lines 0-3, so 3 hidden lines
    assert_eq!(fm.hidden_line_count(0), 3);
    // Not folded line returns 0
    assert_eq!(fm.hidden_line_count(5), 0);
}

#[test]
fn test_multiple_folds() {
    let mut fm = FoldManager::new();
    fm.detect_regions(RUST_CODE);
    let total = RUST_CODE.lines().count();
    fm.fold(0); // fold foo: hides 3
    fm.fold(5); // fold bar: hides lines 6..9 (4 lines)

    // Check both folds work
    assert!(fm.is_hidden(1));
    assert!(fm.is_hidden(2));
    assert!(fm.is_hidden(3));
    assert!(fm.is_hidden(6));
    assert!(fm.is_hidden(7));

    // Visible count = total - 3 - hidden_in_bar
    let bar_hidden = fm.hidden_line_count(5);
    assert_eq!(fm.visible_line_count(total), total - 3 - bar_hidden);
}
