use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Message sent between instances via the named pipe / Unix socket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstanceMessage {
    /// Files to open
    pub files: Vec<PathBuf>,
    /// Go to line
    pub goto_line: Option<usize>,
    /// Go to column
    pub goto_column: Option<usize>,
    /// Override encoding
    pub encoding: Option<String>,
    /// Override language
    pub language: Option<String>,
    /// Open read-only
    pub read_only: bool,
    /// Open in new tab group
    pub new_tab_group: bool,
}

/// Result of trying to become the single instance
pub enum SingleInstanceResult {
    /// We are the primary instance — listen for incoming connections
    Primary(InstanceListener),
    /// Another instance is already running — we sent our message to it
    Secondary,
    /// Single instance mode is not available (e.g., socket error)
    Unavailable(String),
}

/// Listener for incoming instance messages (primary instance)
pub struct InstanceListener {
    #[cfg(unix)]
    listener: std::os::unix::net::UnixListener,
    #[cfg(windows)]
    _handle: (), // Placeholder — Windows uses named pipes
    socket_path: PathBuf,
}

impl InstanceListener {
    /// Try to accept a pending connection (non-blocking).
    /// Returns Some(message) if another instance sent us files to open.
    pub fn try_recv(&self) -> Option<InstanceMessage> {
        #[cfg(unix)]
        {
            self.listener
                .set_nonblocking(true)
                .ok()?;
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(msg) = serde_json::from_str::<InstanceMessage>(&line) {
                                return Some(msg);
                            }
                        }
                    }
                    None
                }
                Err(_) => None,
            }
        }
        #[cfg(windows)]
        {
            // Windows named pipe implementation would go here
            None
        }
    }
}

impl Drop for InstanceListener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Get the socket/pipe path for single instance communication
fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("notepadppp.sock")
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\notepadppp-instance")
    }
}

/// Try to become the single instance or send a message to the existing one.
pub fn try_single_instance(message: InstanceMessage) -> SingleInstanceResult {
    let path = socket_path();

    // Try to connect to existing instance first
    #[cfg(unix)]
    {
        if path.exists() {
            match std::os::unix::net::UnixStream::connect(&path) {
                Ok(mut stream) => {
                    // Existing instance found — send our message
                    if let Ok(json) = serde_json::to_string(&message) {
                        let _ = stream.write_all(json.as_bytes());
                        let _ = stream.write_all(b"\n");
                        let _ = stream.flush();
                    }
                    return SingleInstanceResult::Secondary;
                }
                Err(_) => {
                    // Stale socket — remove it
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        // No existing instance — become the primary
        match std::os::unix::net::UnixListener::bind(&path) {
            Ok(listener) => SingleInstanceResult::Primary(InstanceListener {
                listener,
                socket_path: path,
            }),
            Err(e) => SingleInstanceResult::Unavailable(e.to_string()),
        }
    }

    #[cfg(windows)]
    {
        // On Windows, we'd use named pipes via the windows crate
        // For now, return unavailable — full implementation requires winapi
        SingleInstanceResult::Unavailable("Windows named pipe support pending".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_message_serialize() {
        let msg = InstanceMessage {
            files: vec![PathBuf::from("test.rs"), PathBuf::from("main.rs")],
            goto_line: Some(42),
            goto_column: Some(10),
            encoding: Some("utf-8".to_string()),
            language: Some("rust".to_string()),
            read_only: false,
            new_tab_group: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: InstanceMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.files.len(), 2);
        assert_eq!(deserialized.goto_line, Some(42));
        assert_eq!(deserialized.goto_column, Some(10));
        assert_eq!(deserialized.encoding, Some("utf-8".to_string()));
        assert_eq!(deserialized.language, Some("rust".to_string()));
        assert!(!deserialized.read_only);
        assert!(deserialized.new_tab_group);
    }

    #[test]
    fn test_instance_message_minimal() {
        let msg = InstanceMessage {
            files: vec![],
            goto_line: None,
            goto_column: None,
            encoding: None,
            language: None,
            read_only: false,
            new_tab_group: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"files\":[]"));
    }

    #[test]
    fn test_socket_path_not_empty() {
        let path = socket_path();
        assert!(!path.as_os_str().is_empty());
    }
}
