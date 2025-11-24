// Ratatui-driven TUI adapter - PURE UI LAYER
// All business logic is delegated to use case functions in the domain layer.
use crossterm::event::{Event, KeyCode, KeyModifiers};

pub struct TuiAdapter;

impl TuiAdapter {
    pub fn new() -> Self {
        Self {}
    }

    /// Run the TUI. This blocks and handles input until the user quits.
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};
        use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, Paragraph, List, ListItem}, layout::{Layout, Constraint, Direction}};
        use std::io::stdout;

        #[derive(PartialEq, Eq)]
        enum ScreenState {
            MainMenu,
            Help,
            Profiles,
        }

        #[derive(PartialEq, Eq)]
        enum ProfilesAction {
            None,
            ProfileMenu(usize),
        }

        let mut state = ScreenState::MainMenu;
        let mut message: Option<String> = None;

        // profiles cache shown in Profiles screen
        let mut profiles_list: Vec<(String, String, String, bool)> = Vec::new(); // (key, name, email, is_current)
        let mut profiles_action = ProfilesAction::None;

        // Optional receiver for an in-flight async task result
        let mut task_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>> = None;

        // setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // main event/draw loop
        loop {
            terminal.draw(|f| {
                let size = f.size();
                let block = Block::default().title("Git Account Manager").borders(Borders::ALL);
                f.render_widget(block, size);
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
                    .split(size);

                // Header
                let header = Paragraph::new("Git Account Manager").block(Block::default().borders(Borders::NONE));
                f.render_widget(header, chunks[0]);

                // Body: menu or submenu
                match state {
                    ScreenState::MainMenu => {
                        let items = vec![
                            ListItem::new("1 - Profiles"),
                            ListItem::new("2 - Help/About"),
                            ListItem::new("q - Quit"),
                        ];
                        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Menu"));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::Help => {
                        let help_text = vec![
                            "Git Account Manager (git-acc-mngr)",
                            "",
                            "WHAT IT DOES:",
                            "Manages multiple Git identities with OAuth authentication",
                            "and SSH key management.",
                            "",
                            "BACKGROUND OPERATIONS:",
                            "• OAuth: Authenticates with GitHub",
                            "• Git Config: Updates user.name and user.email",
                            "• SSH Keys: Generates Ed25519 keys per profile",
                            "• SSH Config: Updates ~/.ssh/config",
                            "• Tokens: Encrypts and stores OAuth tokens",
                            "",
                            "FILES & LOCATIONS:",
                            "• Config: ~/.config/git-account-manager/",
                            "• Profiles: profiles.json",
                            "• SSH Keys: keys/<profile>/id_ed25519",
                            "• Encryption: master.key",
                            "",
                            "SETUP REQUIREMENTS:",
                            "Set environment variables:",
                            "  GITHUB_CLIENT_ID=<your_client_id>",
                            "  GITHUB_CLIENT_SECRET=<your_client_secret>",
                            "",
                            "Press 'b' to go back",
                        ];
                        let items: Vec<ListItem> = help_text.iter().map(|s| ListItem::new(*s)).collect();
                        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Help / About"));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::Profiles => {
                        match profiles_action {
                            ProfilesAction::None => {
                                let mut items = vec![ListItem::new("0 - Add new")];
                                if profiles_list.is_empty() {
                                    items.push(ListItem::new(""));
                                    items.push(ListItem::new("(no profiles yet)"));
                                } else {
                                    items.push(ListItem::new(""));
                                    for (i, (_, name, email, is_current)) in profiles_list.iter().enumerate() {
                                        let mut disp = format!("{}: {} <{}>", i + 1, name, email);
                                        if *is_current {
                                            disp.push_str(" - current");
                                        }
                                        items.push(ListItem::new(disp));
                                    }
                                }
                                let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Profiles"));
                                f.render_widget(list, chunks[1]);
                            }
                            ProfilesAction::ProfileMenu(sel) => {
                                let mut items = vec![];
                                if sel < profiles_list.len() {
                                    let (key, _, _, _) = &profiles_list[sel];
                                    items.push(ListItem::new(format!("Selected: {}", key)));
                                } else {
                                    items.push(ListItem::new("Selected: (invalid)"));
                                }
                                items.push(ListItem::new(""));
                                items.push(ListItem::new("1 - Switch profile"));
                                items.push(ListItem::new("2 - Remove profile"));
                                items.push(ListItem::new("3 - Update profile"));
                                items.push(ListItem::new("4 - Back"));
                                let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Profile Actions"));
                                f.render_widget(list, chunks[1]);
                            }
                        }
                    }
                }

                // Footer / message area
                let footer_text = if let Some(msg) = &message {
                    msg.clone()
                } else {
                    match state {
                        ScreenState::MainMenu => "Select an option: 1, 2 or q".to_string(),
                        ScreenState::Help => "Press 'b' to go back".to_string(),
                        ScreenState::Profiles => match profiles_action {
                            ProfilesAction::None => "Press '0' to add new or select a profile, 'b' to go back".to_string(),
                            ProfilesAction::ProfileMenu(_) => "Choose an action: 1-4 or 'b' to go back".to_string(),
                        },
                    }
                };

                let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(footer, chunks[2]);
            })?;

            // wait for input, but use a timeout so we can poll background tasks
            if crossterm::event::poll(std::time::Duration::from_millis(250))? {
                match crossterm::event::read()? {
                    Event::Key(key_event) => {
                        // allow Ctrl-C to quit
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('c') {
                            break;
                        }

                        match state {
                            ScreenState::MainMenu => match key_event.code {
                                KeyCode::Char('1') => {
                                    // Load profiles using the use case function
                                    state = ScreenState::Profiles;
                                    message = None;
                                    let storage = crate::adapters::system_io::LocalSystemIO::new();
                                    match crate::domain::use_cases::list_profiles_use_case(&storage) {
                                        Ok(list) => {
                                            profiles_list = list;
                                        }
                                        Err(e) => {
                                            message = Some(format!("Failed to load profiles: {}", e));
                                            profiles_list = Vec::new();
                                        }
                                    }
                                }
                                KeyCode::Char('2') => {
                                    state = ScreenState::Help;
                                    message = None;
                                }
                                KeyCode::Char('q') => break,
                                _ => {}
                            },
                            ScreenState::Help => match key_event.code {
                                KeyCode::Char('b') | KeyCode::Esc => {
                                    state = ScreenState::MainMenu;
                                    message = None;
                                }
                                _ => {}
                            },
                            ScreenState::Profiles => {
                                match profiles_action {
                                    ProfilesAction::None => match key_event.code {
                                        KeyCode::Char('b') | KeyCode::Esc => {
                                            state = ScreenState::MainMenu;
                                            message = None;
                                            profiles_action = ProfilesAction::None;
                                            // Reload profiles when returning to main menu so it's fresh when reopening
                                            let storage = crate::adapters::system_io::LocalSystemIO::new();
                                            if let Ok(list) = crate::domain::use_cases::list_profiles_use_case(&storage) {
                                                profiles_list = list;
                                            }
                                        }
                                        KeyCode::Char('0') => {
                                            // Start GitHub OAuth flow in background (Add new profile)
                                            message = Some("Starting GitHub OAuth... (browser should open)".to_string());

                                            let (tx, rx) = std::sync::mpsc::channel();
                                            task_rx = Some(rx);

                                            tokio::spawn(async move {
                                                let storage = crate::adapters::system_io::LocalSystemIO::new();
                                                let github_adapter = crate::adapters::github::GithubAdapter::new();

                                                match crate::domain::use_cases::add_github_profile_use_case(&storage, &github_adapter).await {
                                                    Ok(key) => {
                                                        let _ = tx.send(Ok(key));
                                                    }
                                                    Err(e) => {
                                                        let _ = tx.send(Err(e.to_string()));
                                                    }
                                                }
                                            });
                                        }
                                        KeyCode::Char(c) if c.is_ascii_digit() => {
                                            let idx = c.to_digit(10).unwrap_or(0) as usize;
                                            if idx > 0 {
                                                let sel = idx - 1;
                                                if sel < profiles_list.len() {
                                                    profiles_action = ProfilesAction::ProfileMenu(sel);
                                                    message = None;
                                                }
                                            }
                                        }
                                        _ => {}
                                    },
                                    ProfilesAction::ProfileMenu(sel) => match key_event.code {
                                        KeyCode::Char('b') | KeyCode::Char('4') | KeyCode::Esc => {
                                            profiles_action = ProfilesAction::None;
                                            message = None;
                                        }
                                        KeyCode::Char('1') => {
                                            // Switch profile - use the use case function
                                            if sel < profiles_list.len() {
                                                let (key, _, _, _) = &profiles_list[sel];
                                                let key_clone = key.clone();
                                                let storage = crate::adapters::system_io::LocalSystemIO::new();

                                                match crate::domain::use_cases::switch_profile_use_case(&storage, &key_clone) {
                                                    Ok(_) => {
                                                        message = Some(format!("Switched to profile: {}", key_clone));
                                                        // Reload profiles list to update current status
                                                        if let Ok(list) = crate::domain::use_cases::list_profiles_use_case(&storage) {
                                                            profiles_list = list;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        message = Some(format!("Failed to switch profile: {}", e));
                                                    }
                                                }
                                            } else {
                                                message = Some("Invalid selection".to_string());
                                            }
                                            profiles_action = ProfilesAction::None;
                                        }
                                        KeyCode::Char('2') => {
                                            // Remove profile - use the use case function (async)
                                            if sel < profiles_list.len() {
                                                let (key, _, _, _) = &profiles_list[sel];
                                                let key_clone = key.clone();
                                                message = Some(format!("Removing profile {}...", key_clone));

                                                let (tx, rx) = std::sync::mpsc::channel();
                                                task_rx = Some(rx);

                                                tokio::spawn(async move {
                                                    let storage = crate::adapters::system_io::LocalSystemIO::new();

                                                    match crate::domain::use_cases::remove_profile_use_case(&storage, &key_clone).await {
                                                        Ok(_) => {
                                                            let _ = tx.send(Ok(format!("Removed profile: {}", key_clone)));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(Err(format!("Failed to remove profile: {}", e)));
                                                        }
                                                    }
                                                });
                                            } else {
                                                message = Some("Invalid selection".to_string());
                                            }
                                            profiles_action = ProfilesAction::None;
                                        }
                                        KeyCode::Char('3') => {
                                            // Update profile - re-fetch from GitHub (async)
                                            if sel < profiles_list.len() {
                                                let (key, _, _, _) = &profiles_list[sel];
                                                let key_clone = key.clone();
                                                message = Some(format!("Updating profile {}...", key_clone));

                                                let (tx, rx) = std::sync::mpsc::channel();
                                                task_rx = Some(rx);

                                                tokio::spawn(async move {
                                                    let storage = crate::adapters::system_io::LocalSystemIO::new();

                                                    match crate::domain::use_cases::update_profile_use_case(&storage, &key_clone).await {
                                                        Ok(_) => {
                                                            let _ = tx.send(Ok(format!("Updated profile: {}", key_clone)));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(Err(format!("Failed to update profile: {}", e)));
                                                        }
                                                    }
                                                });
                                            } else {
                                                message = Some("Invalid selection".to_string());
                                            }
                                            profiles_action = ProfilesAction::None;
                                        }
                                        _ => {}
                                    },
                                }
                            },
                        }
                    }
                    _ => {}
                }
            }

            // Poll for async task result (non-blocking)
            if let Some(rx) = &task_rx {
                match rx.try_recv() {
                    Ok(Ok(msg)) => {
                        // Task succeeded
                        message = Some(msg);
                        task_rx = None;

                        // Reload profiles list
                        let storage = crate::adapters::system_io::LocalSystemIO::new();
                        if let Ok(list) = crate::domain::use_cases::list_profiles_use_case(&storage) {
                            profiles_list = list;
                        }
                    }
                    Ok(Err(err)) => {
                        message = Some(format!("Operation failed: {}", err));
                        task_rx = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // still waiting
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        message = Some("Background task disconnected unexpectedly".to_string());
                        task_rx = None;
                    }
                }
            }
        }

        // restore terminal
        disable_raw_mode()?;
        execute!(std::io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }
}
