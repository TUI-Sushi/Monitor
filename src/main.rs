use clap::Parser;
use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    widgets::Widget,
};
use remote_stats::RemoteStats;
use session_manager::SessionManager;
use std::time::{Duration, Instant};

mod remote_stats;
mod session_manager;

#[derive(Parser, Debug)]
struct Args {
    /// Hosts to monitor
    #[arg(required=true,long,num_args(0..))]
    host: Vec<String>,
    /// Seconds between pings of each server
    #[arg(default_value_t = 5, long)]
    poll_rate: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let mut terminal = ratatui::init();
    let mut app = App::new(args.poll_rate);

    for i in args.host {
        app.add_host(i).await;
    }

    let result = app.run(&mut terminal).await;
    ratatui::restore();

    result
}

#[derive(Debug, Default)]
struct App {
    remote_stats: Vec<RemoteStats>,
    exit: bool,
    session_manager: SessionManager,
    poll_rate: u16,
}

impl App {
    fn new(poll_rate: u16) -> Self {
        Self {
            remote_stats: Vec::new(),
            exit: false,
            poll_rate,
            session_manager: SessionManager::make(),
        }
    }

    pub async fn add_host(&mut self, host: String) {
        self.remote_stats
            .push(RemoteStats::make(host, &mut self.session_manager).await);
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_secs_f32(self.poll_rate as f32);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                self.handle_events().await?;
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

    async fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit().await,
            _ => {}
        }
    }

    async fn exit(&mut self) {
        self.session_manager.close_all_connections().await;
        self.exit = true
    }

    async fn update_stats(&mut self) {
        for server in &mut self.remote_stats {
            server
                .refresh(&mut self.session_manager)
                .await
                .expect("Failed to refresh server");
        }
    }
}
