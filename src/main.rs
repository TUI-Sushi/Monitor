use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    widgets::Widget,
};
use remote_stats::RemoteStats;
use std::time::{Duration, Instant};

mod remote_stats;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(&mut terminal).await;
    ratatui::restore();

    result
}

#[derive(Debug, Default)]
struct App {
    remote_stats: Vec<RemoteStats>,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            remote_stats: Vec::new(),
            exit: false,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_secs_f32(5.0);
        let mut last_tick = Instant::now();
        self.remote_stats = vec![
            RemoteStats::make(String::from("ctrlc")),
            RemoteStats::make(String::from("sdf")),
        ];

        while !self.exit {
            terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                self.handle_events()?;
            }

            if last_tick.elapsed() >= tick_rate {
                self.update_stats().await;

                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let constraints = self
            .remote_stats
            .iter()
            .map(|_| Constraint::Ratio(1, self.remote_stats.len() as u32));

        let app_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(2)
            .constraints(constraints)
            .split(frame.area());

        for (i, server) in self.remote_stats.iter().enumerate() {
            server.clone().render(app_layout[i], frame.buffer_mut());
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true
    }

    async fn update_stats(&mut self) {
        for server in &mut self.remote_stats {
            server.refresh().await.expect("Failed to refresh server");
        }
    }
}
