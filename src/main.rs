// Hide the console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> iced::Result {
    use notepadppp::ui_iced::app;
    use notepadppp::platform::single_instance::{self, InstanceMessage, SingleInstanceResult};

    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let cli = match notepadppp::platform::cli::CliArgs::parse(&args) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Try 'notepadppp --help' for usage.");
            std::process::exit(1);
        }
    };

    if cli.help {
        println!("{}", notepadppp::platform::cli::CliArgs::help_text());
        std::process::exit(0);
    }

    if cli.version {
        println!("Notepad+++ v{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    if cli.register_shell || cli.unregister_shell {
        handle_shell_registration(&cli);
        std::process::exit(0);
    }

    // Single instance: try to reuse existing instance unless --new-instance
    if !cli.new_instance {
        let msg = InstanceMessage {
            files: cli.files.clone(),
            goto_line: cli.goto_line,
            goto_column: cli.goto_column,
            encoding: cli.encoding.clone(),
            language: cli.language.clone(),
            read_only: cli.read_only,
            new_tab_group: cli.new_tab_group,
        };
        match single_instance::try_single_instance(msg) {
            SingleInstanceResult::Secondary => {
                // Files sent to existing instance — exit
                std::process::exit(0);
            }
            SingleInstanceResult::Primary(listener) => {
                app::INSTANCE_LISTENER
                    .set(std::sync::Mutex::new(Some(listener)))
                    .ok();
            }
            SingleInstanceResult::Unavailable(e) => {
                log::warn!("Single instance unavailable: {e}");
            }
        }
    }

    // Store CLI args so the app's Default impl can read them
    app::CLI_ARGS.set(cli).ok();

    iced::application(app::title, app::update, app::view)
        .subscription(app::subscription)
        .theme(app::theme)
        .window_size(iced::Size::new(1200.0, 800.0))
        .run()
}

fn handle_shell_registration(cli: &notepadppp::platform::cli::CliArgs) {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "notepadppp".to_string());

    let associations = notepadppp::platform::default_associations();
    let context_menu = notepadppp::platform::default_context_menu_entries(&exe_path);
    let _ = &context_menu;

    if cli.register_shell {
        #[cfg(target_os = "windows")]
        {
            use notepadppp::platform::windows::WindowsIntegration;
            use notepadppp::platform::PlatformIntegration;
            let integration = WindowsIntegration::new();
            match integration.register_file_associations(&exe_path, &associations) {
                Ok(()) => println!("Shell integration registered successfully!"),
                Err(e) => eprintln!("Failed to register: {e}"),
            }
        }
        #[cfg(target_os = "linux")]
        {
            use notepadppp::platform::linux::LinuxIntegration;
            use notepadppp::platform::PlatformIntegration;
            let integration = LinuxIntegration::new();
            match integration.register_file_associations(&exe_path, &associations) {
                Ok(()) => println!("Shell integration registered successfully!"),
                Err(e) => eprintln!("Failed to register: {e}"),
            }
        }
        #[cfg(target_os = "macos")]
        {
            println!("macOS: File associations are set via the app bundle Info.plist.");
        }

        #[cfg(target_os = "windows")]
        {
            use notepadppp::platform::windows::WindowsIntegration;
            let reg = WindowsIntegration::generate_reg_file(&exe_path, &associations, &context_menu);
            let ps = WindowsIntegration::generate_powershell_register(&exe_path, &associations, &context_menu);
            if let Some(config) = dirs::config_dir() {
                let dir = config.join("notepadppp");
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(dir.join("register.reg"), &reg);
                let _ = std::fs::write(dir.join("register.ps1"), &ps);
                println!("Registration scripts saved to: {}", dir.display());
            }
        }
    }

    if cli.unregister_shell {
        #[cfg(target_os = "windows")]
        {
            use notepadppp::platform::windows::WindowsIntegration;
            use notepadppp::platform::PlatformIntegration;
            let integration = WindowsIntegration::new();
            match integration.unregister_file_associations(&associations) {
                Ok(()) => println!("Shell integration removed."),
                Err(e) => eprintln!("Failed to unregister: {e}"),
            }
        }
        #[cfg(target_os = "linux")]
        {
            use notepadppp::platform::linux::LinuxIntegration;
            use notepadppp::platform::PlatformIntegration;
            let integration = LinuxIntegration::new();
            match integration.unregister_file_associations(&associations) {
                Ok(()) => println!("Shell integration removed."),
                Err(e) => eprintln!("Failed to unregister: {e}"),
            }
        }
        #[cfg(target_os = "macos")]
        {
            println!("macOS: Remove the app bundle to unregister.");
        }
    }
}
