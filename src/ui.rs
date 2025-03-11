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
use std::time::Duration as StdDuration;

pub fn draw_stats(session: &Session, end_time: DateTime<Local>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let total_duration_secs = (end_time - session.start_time).num_seconds();
    let hours = total_duration_secs / 3600;
    let minutes = (total_duration_secs % 3600) / 60;
    let seconds = total_duration_secs % 60;

    let active_hours = session.active_time_seconds / 3600;
    let active_minutes = (session.active_time_seconds % 3600) / 60;
    let active_seconds = session.active_time_seconds % 60;

    let idle_time_secs = total_duration_secs - session.active_time_seconds;
    let idle_hours = idle_time_secs / 3600;
    let idle_minutes = (idle_time_secs % 3600) / 60;
    let idle_seconds = idle_time_secs % 60;

    let lines_written = session.lines_written.unwrap_or(0);
    let chars_written = session.chars_written.unwrap_or(0);

    let line_change_color = if lines_written >= 0 {
        Color::Green
    } else {
        Color::Red
    };

    let char_change_color = if chars_written >= 0 {
        Color::Green
    } else {
        Color::Red
    };

    let duration_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
    let active_str = format!(
        "{:02}:{:02}:{:02}",
        active_hours, active_minutes, active_seconds
    );
    let idle_str = format!("{:02}:{:02}:{:02}", idle_hours, idle_minutes, idle_seconds);

    let lines_per_active_hour = if session.active_time_seconds > 0 {
        (lines_written as f64) / (session.active_time_seconds as f64 / 3600.0)
    } else {
        0.0
    };

    let chars_per_active_hour = if session.active_time_seconds > 0 {
        (chars_written as f64) / (session.active_time_seconds as f64 / 3600.0)
    } else {
        0.0
    };

    let files_modified = session.files_modified.as_ref().map_or(0, |v| v.len());
    let files_created = session.files_created.as_ref().map_or(0, |v| v.len());
    let files_deleted = session.files_deleted.as_ref().map_or(0, |v| v.len());

    let stats = vec![
        Line::from(Span::styled(
            format!("Project Directory: {:?}", session.project_directory),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("Total Session Duration: {}", duration_str),
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::styled(
            format!("Active Time: {}", active_str),
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            format!("Idle Time: {}", idle_str),
            Style::default().fg(Color::Red),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!(
                "Lines Written: {}{}",
                if lines_written >= 0 { "+" } else { "" },
                lines_written
            ),
            Style::default().fg(line_change_color),
        )),
        Line::from(Span::styled(
            format!(
                "Characters Written: {}{}",
                if chars_written >= 0 { "+" } else { "" },
                chars_written
            ),
            Style::default().fg(char_change_color),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("Productivity Metrics:"),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  • Lines per active hour: {:.2}", lines_per_active_hour),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!(
                "  • Characters per active hour: {:.2}",
                chars_per_active_hour
            ),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("File Changes:"),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  • Created: {}", files_created),
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            format!(
                "  • Modified: {} (content changes detected)",
                files_modified
            ),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("  • Deleted: {}", files_deleted),
            Style::default().fg(Color::Red),
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

        if event::poll(StdDuration::from_millis(200))? {
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
