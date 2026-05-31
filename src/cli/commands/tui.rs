use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub(super) fn run_api_tui(flow_id: Option<&str>) {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(err) => {
            let _ = disable_raw_mode();
            eprintln!("Failed to initialize terminal: {}", err);
            return;
        }
    };
    let mut app = crate::tui::api_app::ApiApp::new(flow_id);
    loop {
        app.poll_run_events();
        let _ = terminal.draw(|frame| crate::tui::api_views::render(frame, &mut app));
        if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if key.code == KeyCode::Char('c')
                        && key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    app.handle_key(key);
                }
            }
        }
        if app.should_quit {
            break;
        }
    }
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
}
