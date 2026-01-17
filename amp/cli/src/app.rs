use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    text::Text,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use anyhow::Result;
use std::io;

use crate::{session::Session, config::Config};

pub struct App {
    pub should_quit: bool,
    pub current_session: Option<Session>,
    pub sessions: Vec<Session>,
    pub config: Config,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::from_env()?;
        let sessions = Session::list_sessions(&config.session_dir).await.unwrap_or_default();
        
        Ok(Self {
            should_quit: false,
            current_session: None,
            sessions,
            config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main event loop
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Cleanup terminal
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // ASCII Art Title with animation
        let ascii_art = r#"
    ╔═══╗ ╔═╗   ╔═╗ ╔═══╗
    ║╔═╗║ ║║╚╗ ╔╝║ ║╔═╗║
    ║║ ║║ ║╔╗╚╗╔╝╔╝ ║╚═╝║
    ║╚═╝║ ║║╚╗╔╝╔╝  ║╔══╝
    ║╔═╗║ ║║ ║║ ║║  ║║
    ╚╝ ╚╝ ╚╝ ╚╝ ╚╝  ╚╝
        "#;
        
        let title_text = format!("{}\n    Agentic Memory Protocol", ascii_art.trim());
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL).title("AMP Bridge"));
        f.render_widget(title, chunks[0]);

        // Main content
        let active_sessions: Vec<_> = self.sessions.iter()
            .filter(|s| matches!(s.status, crate::session::SessionStatus::Active))
            .collect();
        
        let content = if !self.sessions.is_empty() {
            let mut text = format!("Total Sessions: {}\nActive Sessions: {}\n\n", 
                self.sessions.len(), active_sessions.len());
            
            text.push_str("Recent Sessions:\n");
            for session in self.sessions.iter().take(5) {
                text.push_str(&format!(
                    "• {} - {} ({:?})\n",
                    session.started_at.format("%H:%M:%S"),
                    session.agent_command,
                    session.status
                ));
            }
            
            text.push_str("\nPress 'q' or 'Esc' to quit");
            text
        } else {
            "No sessions found\n\nPress 'q' or 'Esc' to quit".to_string()
        };

        let main_content = Paragraph::new(Text::from(content))
            .block(Block::default().borders(Borders::ALL).title("Session Status"));
        f.render_widget(main_content, chunks[1]);

        // Help
        let help = Paragraph::new("Press 'q' or 'Esc' to quit")
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[2]);
    }
}
