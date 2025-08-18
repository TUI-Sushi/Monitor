use openssh::{Error, Session};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Sparkline, Widget},
};

use crate::session_manager::SessionManager;

#[derive(Debug, Clone)]
pub struct RemoteStats {
    cpu_percent: u64,
    cpu_history: Vec<u64>,
    mem_percent: u64,
    mem_history: Vec<u64>,
    disk_usage_percent: u64,
    ssh_conn: String,
}

impl RemoteStats {
    pub async fn make(ssh_conn: String, session_manager: &mut SessionManager) -> Self {
        session_manager.new_connection(ssh_conn.clone()).await;

        Self {
            cpu_percent: 0,
            cpu_history: Vec::new(),
            mem_percent: 0,
            mem_history: Vec::new(),
            disk_usage_percent: 0,
            ssh_conn,
        }
    }

    pub async fn refresh(&mut self, session_manager: &mut SessionManager) -> Result<(), Error> {
        let cpu_command = String::from("top -b -n 10 -d.2 | grep 'Cpu' |  awk 'NR==3{ print($2)}'");

        if let Ok(cpu_num) = session_manager
            .run_command(self.ssh_conn.clone(), cpu_command)
            .await
        {
            self.cpu_percent = cpu_num;
            self.cpu_history.push(self.cpu_percent);
        }

        let mem_command = String::from("free | grep Mem | awk '{print $3/$2 * 100.0}'");

        if let Ok(mem_num) = session_manager
            .run_command(self.ssh_conn.clone(), mem_command)
            .await
        {
            self.mem_percent = mem_num;
            self.mem_history.push(self.mem_percent);
        }

        let storage_command = String::from("df / | awk 'END{print $5}' | sed 's/%//g'");

        if let Ok(storage_num) = session_manager
            .run_command(self.ssh_conn.clone(), storage_command)
            .await
        {
            self.disk_usage_percent = storage_num;
        }

        Ok(())
    }
}

impl Widget for RemoteStats {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Block::default()
            .border_type(ratatui::widgets::BorderType::Double)
            .title(format!("host: {}", self.ssh_conn))
            .render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .margin(3)
            .split(area);

        let mem_vert = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(2), Constraint::Fill(1)])
            .split(layout[0]);

        Gauge::default()
            .percent(self.mem_percent as u16)
            .block(
                Block::bordered()
                    .title(Line::from("Memory %".bold()).centered())
                    .border_set(border::DOUBLE),
            )
            .render(mem_vert[0], buf);

        Sparkline::default()
            .block(
                Block::bordered()
                    .title(Line::from("History".bold()).centered())
                    .border_set(border::DOUBLE),
            )
            .max(100)
            .data(&self.cpu_history)
            .style(Style::default().fg(Color::Blue))
            .render(mem_vert[1], buf);

        let cpu_vert = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(2), Constraint::Fill(1)])
            .split(layout[1]);

        Gauge::default()
            .percent(self.cpu_percent as u16)
            .block(
                Block::bordered()
                    .title(Line::from("CPU %".bold()).centered())
                    .border_set(border::DOUBLE),
            )
            .render(cpu_vert[0], buf);

        Sparkline::default()
            .block(
                Block::bordered()
                    .title(Line::from("History".bold()).centered())
                    .border_set(border::DOUBLE),
            )
            .max(100)
            .data(&self.cpu_history)
            .style(Style::default().fg(Color::Blue))
            .render(cpu_vert[1], buf);

        let hdd_block = Block::bordered()
            .title(Line::from("Disk Usage %".bold()).centered())
            .border_set(border::DOUBLE);

        Gauge::default()
            .percent(self.disk_usage_percent as u16)
            .block(hdd_block)
            .render(layout[2], buf);
    }
}
