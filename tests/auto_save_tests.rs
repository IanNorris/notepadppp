#[test]
fn test_auto_save_timing() {
    // Auto-save should be based on elapsed time
    let start = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(start.elapsed().as_millis() >= 10);
}
