use notepadppp::tools::diff_tool::*;

#[test]
fn diff_identical_texts() {
    let text = "line 1\nline 2\nline 3";
    let result = diff_texts(text, text);

    assert_eq!(result.stats.same, 3);
    assert_eq!(result.stats.added, 0);
    assert_eq!(result.stats.removed, 0);
    assert_eq!(result.stats.changed, 0);
    assert!(result.lines.iter().all(|l| matches!(l, DiffLine::Same(_))));
}

#[test]
fn diff_completely_different() {
    let left = "aaa\nbbb\nccc";
    let right = "xxx\nyyy\nzzz";
    let result = diff_texts(left, right);

    assert_eq!(result.stats.same, 0);
    // All lines differ; they'll be some combination of changed/removed/added
    let total = result.stats.changed + result.stats.removed + result.stats.added;
    assert!(total > 0);
}

#[test]
fn diff_added_lines() {
    let left = "line 1\nline 3";
    let right = "line 1\nline 2\nline 3";
    let result = diff_texts(left, right);

    assert_eq!(result.stats.same, 2);
    assert_eq!(result.stats.added, 1);
    assert_eq!(result.stats.removed, 0);
}

#[test]
fn diff_removed_lines() {
    let left = "line 1\nline 2\nline 3";
    let right = "line 1\nline 3";
    let result = diff_texts(left, right);

    assert_eq!(result.stats.same, 2);
    assert_eq!(result.stats.removed, 1);
    assert_eq!(result.stats.added, 0);
}

#[test]
fn diff_changed_lines() {
    let left = "line 1\nold line\nline 3";
    let right = "line 1\nnew line\nline 3";
    let result = diff_texts(left, right);

    assert_eq!(result.stats.same, 2);
    assert_eq!(result.stats.changed, 1);
}

#[test]
fn diff_empty_inputs() {
    let result = diff_texts("", "");
    assert_eq!(result.stats.same, 0);
    assert_eq!(result.stats.added, 0);
    assert_eq!(result.stats.removed, 0);
    assert_eq!(result.stats.changed, 0);
    assert!(result.lines.is_empty());
}

#[test]
fn diff_left_empty() {
    let result = diff_texts("", "new line 1\nnew line 2");
    assert_eq!(result.stats.added, 2);
    assert_eq!(result.stats.same, 0);
}

#[test]
fn diff_right_empty() {
    let result = diff_texts("old line 1\nold line 2", "");
    assert_eq!(result.stats.removed, 2);
    assert_eq!(result.stats.same, 0);
}

#[test]
fn diff_single_line_same() {
    let result = diff_texts("hello", "hello");
    assert_eq!(result.stats.same, 1);
    assert_eq!(result.lines.len(), 1);
    assert_eq!(result.lines[0], DiffLine::Same("hello".to_string()));
}

#[test]
fn diff_single_line_different() {
    let result = diff_texts("hello", "world");
    assert_eq!(result.stats.changed, 1);
    assert_eq!(result.lines.len(), 1);
    assert_eq!(
        result.lines[0],
        DiffLine::Changed {
            old: "hello".to_string(),
            new: "world".to_string(),
        }
    );
}

#[test]
fn diff_mixed_changes() {
    let left = "header\nalpha\nbeta\nfooter";
    let right = "header\nalpha\ngamma\ndelta\nfooter";
    let result = diff_texts(left, right);

    // header, alpha, footer are common
    assert!(result.stats.same >= 2);
    // The total of all diff types should account for all lines
    let total = result.stats.same + result.stats.added + result.stats.removed + result.stats.changed;
    assert!(total > 0);
}

#[test]
fn diff_stats_consistency() {
    let left = "a\nb\nc\nd\ne";
    let right = "a\nB\nc\nD\nf";
    let result = diff_texts(left, right);

    // Verify stats match actual line counts
    let mut same = 0;
    let mut added = 0;
    let mut removed = 0;
    let mut changed = 0;
    for line in &result.lines {
        match line {
            DiffLine::Same(_) => same += 1,
            DiffLine::Added(_) => added += 1,
            DiffLine::Removed(_) => removed += 1,
            DiffLine::Changed { .. } => changed += 1,
        }
    }
    assert_eq!(result.stats.same, same);
    assert_eq!(result.stats.added, added);
    assert_eq!(result.stats.removed, removed);
    assert_eq!(result.stats.changed, changed);
}
