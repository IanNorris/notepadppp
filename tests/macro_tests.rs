use notepadppp::editor::macros::*;
use notepadppp::editor::document::Document;

#[test]
fn test_start_stop_recording() {
    let mut recorder = MacroRecorder::new();
    assert!(!recorder.is_recording());
    recorder.start_recording();
    assert!(recorder.is_recording());
    let m = recorder.stop_recording("test");
    assert!(!recorder.is_recording());
    assert_eq!(m.name, "test");
    assert!(m.actions.is_empty());
}

#[test]
fn test_record_and_play_insert() {
    let mut recorder = MacroRecorder::new();
    recorder.start_recording();
    recorder.record_action(MacroAction::InsertText("Hello".to_string()));
    let m = recorder.stop_recording("insert_test");

    let mut doc = Document::new();
    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "Hello");
}

#[test]
fn test_record_and_play_delete_backward() {
    let mut doc = Document::from_str("Hello World");
    // Position cursor at end
    doc.cursor.set_position(0, 11);

    let mut recorder = MacroRecorder::new();
    recorder.start_recording();
    recorder.record_action(MacroAction::DeleteBackward(5));
    let m = recorder.stop_recording("delete_test");

    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "Hello ");
}

#[test]
fn test_record_and_play_delete_forward() {
    let mut doc = Document::from_str("Hello World");
    doc.cursor.set_position(0, 5);

    let mut recorder = MacroRecorder::new();
    recorder.start_recording();
    recorder.record_action(MacroAction::DeleteForward(6));
    let m = recorder.stop_recording("delete_fwd");

    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "Hello");
}

#[test]
fn test_play_macro_n_times() {
    let mut doc = Document::new();
    let m = Macro {
        name: "repeat".to_string(),
        actions: vec![MacroAction::InsertText("x".to_string())],
    };

    MacroRecorder::play_macro_n_times(&m, &mut doc, 5);
    assert_eq!(doc.buffer.text(), "xxxxx");
}

#[test]
fn test_empty_macro() {
    let mut doc = Document::from_str("unchanged");
    let m = Macro {
        name: "empty".to_string(),
        actions: vec![],
    };
    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "unchanged");
}

#[test]
fn test_save_and_load_macros() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("macros.json");

    let mut recorder = MacroRecorder::new();
    recorder.start_recording();
    recorder.record_action(MacroAction::InsertText("saved".to_string()));
    recorder.stop_recording("my_macro");

    recorder.save_macros(&path).unwrap();

    let mut loader = MacroRecorder::new();
    loader.load_macros(&path).unwrap();
    let saved = loader.saved_macros();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].name, "my_macro");
    assert_eq!(saved[0].actions.len(), 1);
}

#[test]
fn test_is_recording_state() {
    let mut recorder = MacroRecorder::new();
    assert!(!recorder.is_recording());
    recorder.start_recording();
    assert!(recorder.is_recording());
    recorder.record_action(MacroAction::InsertText("a".to_string()));
    assert!(recorder.is_recording());
    recorder.stop_recording("test");
    assert!(!recorder.is_recording());
}

#[test]
fn test_delete_macro() {
    let mut recorder = MacroRecorder::new();
    recorder.start_recording();
    recorder.record_action(MacroAction::InsertText("a".to_string()));
    recorder.stop_recording("first");

    recorder.start_recording();
    recorder.record_action(MacroAction::InsertText("b".to_string()));
    recorder.stop_recording("second");

    assert_eq!(recorder.saved_macros().len(), 2);
    recorder.delete_macro(0);
    assert_eq!(recorder.saved_macros().len(), 1);
    assert_eq!(recorder.saved_macros()[0].name, "second");
}

#[test]
fn test_record_action_not_recording() {
    let mut recorder = MacroRecorder::new();
    recorder.record_action(MacroAction::InsertText("ignored".to_string()));
    recorder.start_recording();
    let m = recorder.stop_recording("empty");
    assert!(m.actions.is_empty());
}

#[test]
fn test_macro_cursor_move() {
    let mut doc = Document::from_str("Hello World");
    doc.cursor.set_position(0, 0);

    let m = Macro {
        name: "move_and_insert".to_string(),
        actions: vec![
            MacroAction::MoveCursor(CursorMove::Right(5)),
            MacroAction::InsertText("!".to_string()),
        ],
    };

    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "Hello! World");
}

#[test]
fn test_macro_undo() {
    let mut doc = Document::new();
    let m = Macro {
        name: "insert_then_undo".to_string(),
        actions: vec![
            MacroAction::InsertText("Hello".to_string()),
            MacroAction::Undo,
        ],
    };

    MacroRecorder::play_macro(&m, &mut doc);
    assert_eq!(doc.buffer.text(), "");
}
