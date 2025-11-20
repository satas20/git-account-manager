// Ratatui-driven TUI adapter implementing a small menu and internal state.
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crate::domain::ports::AuthProviderPort;

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
            AddMenu,
            Profiles,
        }

        #[derive(PartialEq, Eq)]
        enum ProfilesAction {
            None,
            ProfileMenu(usize), // selected profile index -> show actions
        }

    let mut state = ScreenState::MainMenu;
    let mut message: Option<String> = None;

    // profiles cache shown in Profiles screen
    let mut profiles_list: Vec<String> = Vec::new();
    let mut profiles_display: Vec<String> = Vec::new();
    let mut profiles_action = ProfilesAction::None;

    // Optional receiver for an in-flight OAuth task result (token or error)
    let mut oauth_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>> = None;

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
                            ListItem::new("1 - Add new"),
                            ListItem::new("2 - Profiles"),
                            ListItem::new("q - Quit"),
                        ];
                        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Menu"));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::AddMenu => {
                        let items = vec![
                            ListItem::new("1 - GitHub"),
                            ListItem::new("b - Back"),
                        ];
                        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Add New"));
                        f.render_widget(list, chunks[1]);
                    }
                    ScreenState::Profiles => {
                        match profiles_action {
                            ProfilesAction::None => {
                                let items = if profiles_display.is_empty() {
                                    vec![ListItem::new("(no profiles yet)")]
                                } else {
                                    profiles_display.iter().map(|s| ListItem::new(s.as_str())).collect()
                                };
                                let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Profiles"));
                                f.render_widget(list, chunks[1]);
                            }
                            ProfilesAction::ProfileMenu(sel) => {
                                // show the selected profile and available actions
                                let mut items = vec![];
                                if sel < profiles_list.len() {
                                    let key = &profiles_list[sel];
                                    // show profile header line
                                    items.push(ListItem::new(format!("Selected: {}", key)));
                                } else {
                                    items.push(ListItem::new("Selected: (invalid)"));
                                }
                                items.push(ListItem::new(""));
                                items.push(ListItem::new("1 - Switch profile"));
                                items.push(ListItem::new("2 - Remove profile"));
                                items.push(ListItem::new("3 - Back"));
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
                        ScreenState::AddMenu => "Choose a provider or 'b' to go back".to_string(),
                        ScreenState::Profiles => match profiles_action {
                            ProfilesAction::None => "Choose a profile or 'b' to go back".to_string(),
                            ProfilesAction::ProfileMenu(_) => "Choose an action or 'b' to go back".to_string(),
                        },
                    }
                };

                let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(footer, chunks[2]);
            })?;

            // wait for input, but use a timeout so we can poll background tasks
            // (OAuth thread) and re-render when work completes.
            if crossterm::event::poll(std::time::Duration::from_millis(250))? {
                // there is an event available
                match crossterm::event::read()? {
                    Event::Key(key_event) => {
                    // allow Ctrl-C to quit
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('c') {
                        break;
                    }

                    match state {
                        ScreenState::MainMenu => match key_event.code {
                            KeyCode::Char('1') => {
                                state = ScreenState::AddMenu;
                                message = None;
                            }
                            KeyCode::Char('2') => {
                                // switch to profiles screen and load profiles from storage
                                state = ScreenState::Profiles;
                                message = None;
                                // load profiles via ProfilesManager using LocalSystemIO
                                let storage = crate::adapters::system_io::LocalSystemIO::new();
                                match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                    Ok(mgr) => match mgr.list_keys() {
                                        Ok(list) => {
                                            profiles_list = list;
                                            // build display strings and annotate current profile
                                            profiles_display.clear();
                                            // detect git current identity
                                            let git_name = mgr.get_git_config("user.name");
                                            let git_email = mgr.get_git_config("user.email");
                                            for (i, key) in profiles_list.iter().enumerate() {
                                                match mgr.get_profile(key) {
                                                    Ok(Some(rec)) => {
                                                        let mut disp = format!("{}: {} <{}>", i+1, rec.name, rec.email);
                                                        if (git_name.as_deref().map(|s| s == rec.name).unwrap_or(false)) ||
                                                           (git_email.as_deref().map(|s| s == rec.email).unwrap_or(false)) {
                                                            disp.push_str(" - current");
                                                        }
                                                        profiles_display.push(disp);
                                                    }
                                                    _ => profiles_display.push(format!("{}: {}", i+1, key)),
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            message = Some(format!("Failed to load profiles: {}", e));
                                            profiles_list = Vec::new();
                                            profiles_display = Vec::new();
                                        }
                                    },
                                    Err(e) => {
                                        message = Some(format!("Failed to create profiles manager: {}", e));
                                        profiles_list = Vec::new();
                                        profiles_display = Vec::new();
                                    }
                                }
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        },
                        ScreenState::AddMenu => match key_event.code {
                            KeyCode::Char('1') => {
                                // select GitHub -> start async OAuth flow in background
                                message = Some("Starting GitHub OAuth... (browser should open)".to_string());

                                // create channel and spawn async task to perform the flow
                                let (tx, rx) = std::sync::mpsc::channel();
                                oauth_rx = Some(rx);

                                // spawn on the existing Tokio runtime (main uses #[tokio::main])
                                let adapter = crate::adapters::github::GithubAdapter::new();
                                tokio::spawn(async move {
                                    // 1) Run OAuth and get token
                                    match adapter.start_oauth_flow_async("default").await {
                                        Ok(token) => {
                                            // 2) Fetch user profile from GitHub
                                            match adapter.fetch_profile(&token).await {
                                                Ok(profile) => {
                                                    // 3) Persist profile using ProfilesManager + LocalSystemIO
                                                    let storage = crate::adapters::system_io::LocalSystemIO::new();
                                                    match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                                        Ok(mgr) => {
                                                            match mgr.create_from_entity(&profile, Some("github".to_string()), None) {
                                                                Ok(key) => {
                                                                    let _ = tx.send(Ok(key));
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(Err(format!("Failed to save profile: {}", e)));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(Err(format!("Failed to create profiles manager: {}", e)));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Err(format!("Failed to fetch profile: {}", e)));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Err(e));
                                        }
                                    }
                                });
                            }
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
                                    }
                                    KeyCode::Char(c) if c.is_ascii_digit() => {
                                        // user selected a profile index; enter ProfileMenu(sel)
                                        let idx = c.to_digit(10).unwrap_or(0) as usize;
                                        if idx == 0 { /* ignore 0 */ }
                                        else {
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
                                    KeyCode::Char('b') | KeyCode::Char('3') | KeyCode::Esc => {
                                        // back to profiles list
                                        profiles_action = ProfilesAction::None;
                                        message = None;
                                    }
                                    KeyCode::Char('1') => {
                                        // switch profile (not implemented) — show status and return
                                        if sel < profiles_list.len() {
                                            let key = &profiles_list[sel];
                                            message = Some(format!("Switch profile selected: {} (not implemented)", key));
                                        } else {
                                            message = Some("Invalid selection".to_string());
                                        }
                                        profiles_action = ProfilesAction::None;
                                    }
                                    KeyCode::Char('2') => {
                                        // remove profile
                                        if sel < profiles_list.len() {
                                            let key = profiles_list[sel].clone();
                                            let storage = crate::adapters::system_io::LocalSystemIO::new();
                                            match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                                Ok(mgr) => match mgr.remove_profile(&key) {
                                                    Ok(true) => {
                                                        message = Some(format!("Removed profile: {}", key));
                                                        // reload profiles
                                                        match mgr.list_keys() {
                                                            Ok(list) => profiles_list = list,
                                                            Err(e) => { profiles_list = Vec::new(); message = Some(format!("Removed but failed reload: {}", e)); }
                                                        }
                                                        // rebuild display
                                                        profiles_display.clear();
                                                        let git_name = mgr.get_git_config("user.name");
                                                        let git_email = mgr.get_git_config("user.email");
                                                        for (i, key) in profiles_list.iter().enumerate() {
                                                            match mgr.get_profile(key) {
                                                                Ok(Some(rec)) => {
                                                                    let mut disp = format!("{}: {} <{}>", i+1, rec.name, rec.email);
                                                                    if (git_name.as_deref().map(|s| s == rec.name).unwrap_or(false)) ||
                                                                       (git_email.as_deref().map(|s| s == rec.email).unwrap_or(false)) {
                                                                        disp.push_str(" - current");
                                                                    }
                                                                    profiles_display.push(disp);
                                                                }
                                                                _ => profiles_display.push(format!("{}: {}", i+1, key)),
                                                            }
                                                        }
                                                    }
                                                    Ok(false) => { message = Some(format!("Profile not found: {}", key)); }
                                                    Err(e) => { message = Some(format!("Failed to remove profile: {}", e)); }
                                                },
                                                Err(e) => { message = Some(format!("Failed to create profiles manager: {}", e)); }
                                            }
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
                    _ => {
                        // ignore other events for now (Mouse, Resize, Focus, Paste, ...)
                    }
                }
            } else {
                // poll timed out: no user input, fall through to poll background results
            }

                // Poll for OAuth result (non-blocking)
                if let Some(rx) = &oauth_rx {
                    match rx.try_recv() {
                        Ok(Ok(token)) => {
                            message = Some(format!("GitHub OAuth succeeded. token={} (stored in memory)", token));
                            oauth_rx = None;
                        }
                        Ok(Err(err)) => {
                            message = Some(format!("GitHub OAuth failed: {}", err));
                            oauth_rx = None;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            // still waiting
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            message = Some("GitHub OAuth task disconnected unexpectedly".to_string());
                            oauth_rx = None;
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
