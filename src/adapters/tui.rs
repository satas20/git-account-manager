// Ratatui-driven TUI adapter implementing a small menu and internal state.
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crate::domain::ports::AuthProviderPort;
//TODO: moudle which only should have UI logic 
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
                                                        if (git_name.as_deref().map(|s| s == rec.name).unwrap_or(false)) &&
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
                                            Ok((token, refresh_token)) => {
                                                // 2) Fetch user profile from GitHub (using direct token since profile doesn't exist yet)
                                                match adapter.fetch_profile_with_token(&token).await {
                                                    Ok(profile) => {
                                                        // 3) Persist profile using ProfilesManager + LocalSystemIO
                                                        let storage: crate::adapters::system_io::LocalSystemIO = crate::adapters::system_io::LocalSystemIO::new();
                                                        match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                                            Ok(mgr) => {
                                                                match mgr.create_from_entity(&profile, Some("github".to_string()), None) {
                                                                    Ok(key) => {
                                                                        // 4) store token encrypted in profile
                                                                        if let Err(e) = mgr.set_token_for_profile(&key, &token) {
                                                                            let _ = tx.send(Err(format!("Saved profile but failed to store token: {}", e)));
                                                                            return;
                                                                        }

                                                                        // 4b) store refresh token if present
                                                                        if let Some(rt) = refresh_token {
                                                                            if let Err(e) = mgr.set_refresh_token_for_profile(&key, &rt) {
                                                                                let _ = tx.send(Err(format!("Saved profile but failed to store refresh token: {}", e)));
                                                                                return;
                                                                            }
                                                                        }

                                                                        // 5) generate SSH keypair for profile under config dir
                                                                        if let Err(e) = mgr.generate_and_assign_ssh_key(&key) {
                                                                            let _ = tx.send(Err(format!("Saved profile but failed to generate SSH key: {}", e)));
                                                                            return;
                                                                        }

                                                                        // 6) Upload public key to GitHub (using direct token method)
                                                                        match mgr.get_public_key_content(&key) {
                                                                            Ok(pub_key) => {
                                                                                match adapter.upload_ssh_key_with_token(&token, &pub_key).await {
                                                                                    Ok(key_id) => {
                                                                                        // Update profile with key_id
                                                                                        if let Ok(Some(mut rec)) = mgr.get_profile(&key) {
                                                                                            rec.ssh_key_id = Some(key_id);
                                                                                            if let Err(e) = mgr.upsert_profile(&key, rec) {
                                                                                                let _ = tx.send(Err(format!("Uploaded key but failed to save key ID: {}", e)));
                                                                                                return;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        let _ = tx.send(Err(format!("Generated key but failed to upload to GitHub: {}", e)));
                                                                                        return;
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                let _ = tx.send(Err(format!("Generated key but failed to read public key: {}", e)));
                                                                                return;
                                                                            }
                                                                        }

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
                                        // switch profile
                                        if sel < profiles_list.len() {
                                            let key = &profiles_list[sel];
                                            let key_clone = key.clone();
                                            let storage = crate::adapters::system_io::LocalSystemIO::new();
                                            match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                                Ok(mgr) => match mgr.switch_profile(&key_clone) {
                                                    Ok(_) => {
                                                        message = Some(format!("git and ssh configured using profile {}", key_clone));
                                                    }
                                                    Err(e) => {
                                                        message = Some(format!("Failed to switch profile: {}", e));
                                                    }
                                                },
                                                Err(e) => {
                                                    message = Some(format!("Failed to create profiles manager: {}", e));
                                                }
                                            }
                                        } else {
                                            message = Some("Invalid selection".to_string());
                                        }
                                        profiles_action = ProfilesAction::None;
                                    }
                                    KeyCode::Char('2') => {
                                        // remove profile
                                        if sel < profiles_list.len() {
                                            let key = profiles_list[sel].clone();
                                            message = Some(format!("Removing profile {}...", key));
                                            
                                            // We need to do async work (delete from GitHub) but we are in sync loop.
                                            // Spawn a task to do the removal and send result back via channel.
                                            let (tx, rx) = std::sync::mpsc::channel();
                                            oauth_rx = Some(rx); // reuse the oauth_rx channel for status updates
                                            
                                            // Clone key for the closure
                                            let key_clone = key.clone();
                                            
                                            tokio::spawn(async move {
                                                let storage = crate::adapters::system_io::LocalSystemIO::new();
                                                match crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                                    Ok(mgr) => {
                                                        // 1. Try to delete from GitHub if we have key_id
                                                        if let Ok(Some(rec)) = mgr.get_profile(&key_clone) {
                                                            if let Some(key_id) = rec.ssh_key_id {
                                                                let adapter = crate::adapters::github::GithubAdapter::new();
                                                                // Use the new profile-based API that handles token refresh
                                                                if let Err(e) = adapter.delete_ssh_key(&key_clone, key_id).await {
                                                                    // Log error but continue to remove local profile?
                                                                    // Or fail? User said "remove it form the git account also".
                                                                    // Let's report error but proceed with local removal or stop?
                                                                    // Usually better to try best effort or warn.
                                                                    // For now, let's fail if API fails so user knows.
                                                                    let _ = tx.send(Err(format!("Failed to delete key from GitHub: {}", e)));
                                                                    return;
                                                                }
                                                            }
                                                        }

                                                        // 2. Remove locally (files and config)
                                                        match mgr.remove_profile(&key_clone) {
                                                            Ok(true) => {
                                                                let _ = tx.send(Ok(format!("Removed profile: {}", key_clone)));
                                                            }
                                                            Ok(false) => {
                                                                let _ = tx.send(Err(format!("Profile not found: {}", key_clone)));
                                                            }
                                                            Err(e) => {
                                                                let _ = tx.send(Err(format!("Failed to remove profile: {}", e)));
                                                            }
                                                        }
                                                    },
                                                    Err(e) => {
                                                        let _ = tx.send(Err(format!("Failed to create profiles manager: {}", e)));
                                                    }
                                                }
                                            });
                                            
                                            profiles_action = ProfilesAction::None;
                                        } else {
                                            message = Some("Invalid selection".to_string());
                                            profiles_action = ProfilesAction::None;
                                        }
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
                        Ok(Ok(msg)) => {
                            // Task succeeded.
                            // If we were adding a profile, switch to Profiles screen.
                            if state == ScreenState::AddMenu {
                                state = ScreenState::Profiles;
                                profiles_action = ProfilesAction::None;
                                message = Some(format!("Profile saved: {}", msg));
                            } else {
                                // If we were removing, just show the message
                                message = Some(msg);
                            }
                            oauth_rx = None;

                            // Reload profiles list
                            let storage = crate::adapters::system_io::LocalSystemIO::new();
                            if let Ok(mgr) = crate::domain::use_cases::ProfilesManager::new(&storage, None) {
                                if let Ok(list) = mgr.list_keys() {
                                    profiles_list = list;
                                    profiles_display.clear();
                                    let git_name = mgr.get_git_config("user.name");
                                    let git_email = mgr.get_git_config("user.email");
                                    for (i, key) in profiles_list.iter().enumerate() {
                                        match mgr.get_profile(key) {
                                            Ok(Some(rec)) => {
                                                let mut disp = format!("{}: {} <{}>", i+1, rec.name, rec.email);
                                                if (git_name.as_deref().map(|s| s == rec.name).unwrap_or(false)) &&
                                                   (git_email.as_deref().map(|s| s == rec.email).unwrap_or(false)) {
                                                    disp.push_str(" - current");
                                                }
                                                profiles_display.push(disp);
                                            }
                                            _ => profiles_display.push(format!("{}: {}", i+1, key)),
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Err(err)) => {
                            message = Some(format!("Operation failed: {}", err));
                            oauth_rx = None;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            // still waiting
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            message = Some("Background task disconnected unexpectedly".to_string());
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
