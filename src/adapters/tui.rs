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

    let mut state = ScreenState::MainMenu;
    let mut message: Option<String> = None;

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
                        let items = vec![ListItem::new("(no profiles yet)")];
                        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Profiles"));
                        f.render_widget(list, chunks[1]);
                    }
                }

                // Footer / message area
                let footer_text = if let Some(msg) = &message {
                    msg.clone()
                } else {
                    match state {
                        ScreenState::MainMenu => "Select an option: 1, 2 or q".to_string(),
                        ScreenState::AddMenu => "Choose a provider or 'b' to go back".to_string(),
                        ScreenState::Profiles => "Press 'b' to go back".to_string(),
                    }
                };

                let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(footer, chunks[2]);
            })?;

            // wait for input
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
                                state = ScreenState::Profiles;
                                message = None;
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
                        ScreenState::Profiles => match key_event.code {
                            KeyCode::Char('b') | KeyCode::Esc => {
                                state = ScreenState::MainMenu;
                                message = None;
                            }
                            _ => {}
                        },
                    }
                }
                _ => {
                    // ignore other events for now (Mouse, Resize, Focus, Paste, ...)
                }
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
