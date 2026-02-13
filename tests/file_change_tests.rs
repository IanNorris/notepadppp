use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_file_modification_time_changes() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();

    fs::write(&path, "initial content").unwrap();
    let time1 = fs::metadata(&path).unwrap().modified().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));

    fs::write(&path, "modified content").unwrap();
    let time2 = fs::metadata(&path).unwrap().modified().unwrap();

    assert!(time2 > time1);
}
