// Ratatui-driven TUI adapter - PURE UI LAYER
// All business logic is delegated to use case functions in the domain layer.
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::Span;

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
            ProviderSelection,
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

        // Define color scheme
        let bg_color = Color::Rgb(54, 54, 54);        // #363636 - main background
        let highlight_color = Color::Rgb(235, 81, 53); // #eb5135 - highlight/selected
        let text_color = Color::Rgb(240, 240, 240);    // #F0F0F0 - default text
        let separator_color = Color::Rgb(136, 136, 136); // #888888 - separators

        // main event/draw loop
        loop {
            terminal.draw(|f| {
                let size = f.size();
                let block = Block::default()
                    .title("Git Account Manager")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(separator_color))
                    .style(Style::default().bg(bg_color).fg(text_color));
                f.render_widget(block, size);
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
                    .split(size);

                // Header
                let header = Paragraph::new("Git Account Manager")
                    .style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD))
                    .block(Block::default().borders(Borders::NONE).style(Style::default().bg(bg_color)));
                f.render_widget(header, chunks[0]);

                // Body: menu or submenu
                match state {
                    ScreenState::MainMenu => {
                        let items = vec![
                            ListItem::new("1 - Profiles").style(Style::default().fg(text_color)),
                            ListItem::new("2 - Help/About").style(Style::default().fg(text_color)),
                            ListItem::new("q - Quit").style(Style::default().fg(text_color)),
                        ];
                        let list = List::new(items)
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(separator_color))
                                .title(Span::styled("Menu", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                                .style(Style::default().bg(bg_color)));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::ProviderSelection => {
                        let items = vec![
                            ListItem::new("Select a provider:").style(Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                            ListItem::new(""),
                            ListItem::new("1 - GitHub").style(Style::default().fg(text_color)),
                            ListItem::new("2 - GitLab").style(Style::default().fg(text_color)),
                            ListItem::new(""),
                            ListItem::new("b - Back").style(Style::default().fg(text_color)),
                        ];
                        let list = List::new(items)
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(separator_color))
                                .title(Span::styled("Add New Profile - Provider Selection", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                                .style(Style::default().bg(bg_color)));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::Help => {
                        let help_text = vec![
                            ("Git Account Manager (git-acc-mngr)", true),
                            ("", false),
                            ("WHAT IT DOES:", true),
                            ("Manages multiple Git identities with OAuth authentication", false),
                            ("and SSH key management.", false),
                            ("", false),
                            ("BACKGROUND OPERATIONS:", true),
                            ("• OAuth: Authenticates with GitHub & GitLab", false),
                            ("• Git Config: Updates user.name and user.email", false),
                            ("• SSH Keys: Generates Ed25519 keys per profile", false),
                            ("• SSH Config: Updates ~/.ssh/config", false),
                            ("• Tokens: Encrypts and stores OAuth tokens", false),
                            ("", false),
                            ("FILES & LOCATIONS:", true),
                            ("• Config: ~/.config/git-account-manager/", false),
                            ("• Profiles: profiles.json", false),
                            ("• SSH Keys: keys/<profile>/id_ed25519", false),
                            ("• Encryption: master.key", false),
                            ("", false),
                            ("SETUP REQUIREMENTS:", true),
                            ("GitHub:", false),
                            ("  GITHUB_CLIENT_ID=<your_client_id>", false),
                            ("  GITHUB_CLIENT_SECRET=<your_client_secret>", false),
                            ("GitLab:", false),
                            ("  GITLAB_APP_ID=<your_app_id>", false),
                            ("  GITLAB_CLIENT_SECRET=<your_client_secret>", false),
                            ("", false),
                            ("Press 'b' to go back", false),
                        ];
                        let items: Vec<ListItem> = help_text.iter().map(|(s, is_header)| {
                            if *is_header {
                                ListItem::new(*s).style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD))
                            } else {
                                ListItem::new(*s).style(Style::default().fg(text_color))
                            }
                        }).collect();
                        let list = List::new(items)
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(separator_color))
                                .title(Span::styled("Help / About", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                                .style(Style::default().bg(bg_color)));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::Profiles => {
                        match profiles_action {
                            ProfilesAction::None => {
                                let mut items = vec![ListItem::new("0 - Add new").style(Style::default().fg(text_color))];
                                if profiles_list.is_empty() {
                                    items.push(ListItem::new(""));
                                    items.push(ListItem::new("(no profiles yet)").style(Style::default().fg(separator_color)));
                                } else {
                                    items.push(ListItem::new(""));
                                    for (i, (key, name, email, is_current)) in profiles_list.iter().enumerate() {
                                        // Extract adapter name from key (format: "username@adapter")
                                        let adapter = key.split('@').nth(1).unwrap_or("unknown");
                                        let mut disp = format!("{}: {} <{}> [{}]", i + 1, name, email, adapter);
                                        if *is_current {
                                            disp.push_str(" - current");
                                            items.push(ListItem::new(disp).style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)));
                                        } else {
                                            items.push(ListItem::new(disp).style(Style::default().fg(text_color)));
                                        }
                                    }
                                }
                                items.push(ListItem::new(""));
                                items.push(ListItem::new("b - Back").style(Style::default().fg(text_color)));
                                let list = List::new(items)
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(separator_color))
                                        .title(Span::styled("Profiles", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                                        .style(Style::default().bg(bg_color)));
                                f.render_widget(list, chunks[1]);
                            }
                            ProfilesAction::ProfileMenu(sel) => {
                                let mut items = vec![];
                                if sel < profiles_list.len() {
                                    let (key, _, _, _) = &profiles_list[sel];
                                    items.push(ListItem::new(format!("Selected: {}", key))
                                        .style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)));
                                } else {
                                    items.push(ListItem::new("Selected: (invalid)")
                                        .style(Style::default().fg(separator_color)));
                                }
                                items.push(ListItem::new(""));
                                items.push(ListItem::new("1 - Switch profile").style(Style::default().fg(text_color)));
                                items.push(ListItem::new("2 - Remove profile").style(Style::default().fg(text_color)));
                                items.push(ListItem::new("3 - Sync account").style(Style::default().fg(text_color)));
                                items.push(ListItem::new(""));
                                items.push(ListItem::new("b - Back").style(Style::default().fg(text_color)));
                                let list = List::new(items)
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(separator_color))
                                        .title(Span::styled("Profile Actions", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                                        .style(Style::default().bg(bg_color)));
                                f.render_widget(list, chunks[1]);
                            }
                        }
                    }
                }

                // Footer / message area
                let (footer_text, is_success) = if let Some(msg) = &message {
                    // Check if message indicates success or failure
                    let is_success = msg.contains("Added") || msg.contains("Switched") || msg.contains("Removed") || msg.contains("Synced");
                    (msg.clone(), is_success)
                } else {
                    let default_msg = match state {
                        ScreenState::MainMenu => "Select an option: 1, 2, or q".to_string(),
                        ScreenState::Help => "Press 'b' to go back".to_string(),
                        ScreenState::ProviderSelection => "Select a provider: 1 (GitHub), 2 (GitLab), or 'b' to go back".to_string(),
                        ScreenState::Profiles => match profiles_action {
                            ProfilesAction::None => "Press '0' to add new or select a profile, 'b' to go back".to_string(),
                            ProfilesAction::ProfileMenu(_) => "Choose an action: 1-3 or 'b' to go back".to_string(),
                        },
                    };
                    (default_msg, false)
                };

                let footer_style = if is_success {
                    Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(text_color)
                };

                let footer = Paragraph::new(footer_text)
                    .style(footer_style)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(separator_color))
                        .title(Span::styled("Status", Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)))
                        .style(Style::default().bg(bg_color)));
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
                            ScreenState::ProviderSelection => match key_event.code {
                                KeyCode::Char('b') | KeyCode::Esc => {
                                    state = ScreenState::Profiles;
                                    message = None;
                                }
                                KeyCode::Char('1') => {
                                    // GitHub provider selected - Start GitHub OAuth flow in background
                                    message = Some("Starting GitHub OAuth... (browser should open)".to_string());
                                    state = ScreenState::Profiles;

                                    let (tx, rx) = std::sync::mpsc::channel();
                                    task_rx = Some(rx);

                                    tokio::spawn(async move {
                                        let storage = crate::adapters::system_io::LocalSystemIO::new();
                                        let github_adapter = crate::adapters::github::GithubAdapter::new();

                                        match crate::domain::use_cases::add_github_profile_use_case(&storage, &github_adapter).await {
                                            Ok(key) => {
                                                let _ = tx.send(Ok(format!("Added GitHub profile: {}", key)));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Err(e.to_string()));
                                            }
                                        }
                                    });
                                }
                                KeyCode::Char('2') => {
                                    // GitLab provider selected - Start GitLab OAuth flow in background
                                    message = Some("Starting GitLab OAuth... (browser should open)".to_string());
                                    state = ScreenState::Profiles;

                                    let (tx, rx) = std::sync::mpsc::channel();
                                    task_rx = Some(rx);

                                    tokio::spawn(async move {
                                        let storage = crate::adapters::system_io::LocalSystemIO::new();
                                        let gitlab_adapter = crate::adapters::gitlab::GitlabAdapter::new();

                                        match crate::domain::use_cases::add_gitlab_profile_use_case(&storage, &gitlab_adapter).await {
                                            Ok(key) => {
                                                let _ = tx.send(Ok(format!("Added GitLab profile: {}", key)));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Err(e.to_string()));
                                            }
                                        }
                                    });
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
                                            // Go to provider selection menu
                                            state = ScreenState::ProviderSelection;
                                            message = None;
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
                                        KeyCode::Char('b') | KeyCode::Esc => {
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
                                            // Sync account - re-fetch from provider and rotate SSH key (async)
                                            if sel < profiles_list.len() {
                                                let (key, _, _, _) = &profiles_list[sel];
                                                let key_clone = key.clone();
                                                message = Some(format!("Syncing account {}...", key_clone));

                                                let (tx, rx) = std::sync::mpsc::channel();
                                                task_rx = Some(rx);

                                                tokio::spawn(async move {
                                                    let storage = crate::adapters::system_io::LocalSystemIO::new();

                                                    match crate::domain::use_cases::sync_account_use_case(&storage, &key_clone).await {
                                                        Ok(_) => {
                                                            let _ = tx.send(Ok(format!("Synced account: {}", key_clone)));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(Err(format!("Failed to sync account: {}", e)));
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
