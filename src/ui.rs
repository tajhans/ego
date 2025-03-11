use crate::session::Session;
use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::Duration;

pub fn draw_stats(session: &Session, end_time: DateTime<Local>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let duration_secs = (end_time - session.start_time).num_seconds();
    let hours = duration_secs / 3600;
    let minutes = (duration_secs % 3600) / 60;
    let seconds = duration_secs % 60;

    let lines_written = session.lines_written.unwrap_or(0);
    let line_change_color = if lines_written >= 0 {
        Color::Green
    } else {
        Color::Red
    };

    let duration_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    let stats = vec![
        Line::from(Span::styled(
            format!("Project Directory: {:?}", session.project_directory),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("Session Duration: {}", duration_str),
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::styled(
            format!("Initial Line Count: {}", session.initial_line_count),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!(
                "Final Line Count: {}",
                session.final_line_count.unwrap_or(0)
            ),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!(
                "Lines Written: {}{}",
                if lines_written >= 0 { "+" } else { "" },
                lines_written
            ),
            Style::default().fg(line_change_color),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "Press any key to exit.",
            Style::default().add_modifier(Modifier::ITALIC),
        )),
    ];

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Ego - Session Stats");
            f.render_widget(block, size);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(stats.len() as u16), Constraint::Min(0)].as_ref())
                .split(size);

            let paragraph = Paragraph::new(stats.clone())
                .alignment(Alignment::Left)
                .block(Block::default());
            f.render_widget(paragraph, chunks[0]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code != KeyCode::Null {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
