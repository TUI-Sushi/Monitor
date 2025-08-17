use openssh::{Error, Session};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Sparkline, Widget},
};

#[derive(Debug, Clone)]
pub struct RemoteStats {
    cpu_percent: f32,
    cpu_history: Vec<u64>,
    mem_percent: f32,
    mem_history: Vec<u64>,
    disk_usage_percent: f32,
    ssh_conn: String,
}

impl RemoteStats {
    pub fn make(ssh_conn: String) -> Self {
        Self {
            cpu_percent: 0.0,
            cpu_history: Vec::new(),
            mem_percent: 0.0,
            mem_history: Vec::new(),
            disk_usage_percent: 0.0,
            ssh_conn,
        }
    }

    pub async fn refresh(&mut self) -> Result<(), Error> {
        let session = Session::connect(&self.ssh_conn, openssh::KnownHosts::Strict)
            .await
            .expect("Failed connection");

        let cpu_result = session
            .raw_command("top -b -n 10 -d.2 | grep 'Cpu' |  awk 'NR==3{ print($2)}'")
            .output()
            .await
            .expect("Failed to get cpu");

        if let Ok(cpu_num) = String::from_utf8(cpu_result.stdout) {
            self.cpu_percent = cpu_num
                .strip_suffix("\r\n")
                .or(cpu_num.strip_suffix("\n"))
                .unwrap_or(cpu_num.as_str())
                .to_string()
                .trim()
                .parse()
                .unwrap_or(0.0);
            self.cpu_history.push(self.cpu_percent as u64);
        }

        let mem_result = session
            .raw_command("free | grep Mem | awk '{print $3/$2 * 100.0}'")
            .output()
            .await
            .expect("Failed to get memory");

        if let Ok(mem_num) = String::from_utf8(mem_result.stdout) {
            self.mem_percent = mem_num
                .strip_suffix("\r\n")
                .or(mem_num.strip_suffix("\n"))
                .unwrap_or(mem_num.as_str())
                .to_string()
                .trim()
                .parse()
                .unwrap_or(0.0);
            self.mem_history.push(self.mem_percent as u64);
        }

        let storage_result = session
            .raw_command("df / | awk 'END{print $5}' | sed 's/%//g'")
            .output()
            .await
            .expect("Failed to get storage");

        if let Ok(storage_num) = String::from_utf8(storage_result.stdout) {
            self.disk_usage_percent = storage_num
                .strip_suffix("\r\n")
                .or(storage_num.strip_suffix("\n"))
                .unwrap_or(storage_num.as_str())
                .to_string()
                .trim()
                .parse()
                .unwrap_or(0.0);
        }

        session.close().await?;

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

        // let data = vec![(10.0, 10.0), (20.0, 20.0), (30.0, 30.0)];

        // let _ = Chart::new(vec![
        //    Dataset::default()
        //        .data(&data)
        //        .marker(ratatui::symbols::Marker::HalfBlock)
        //        .style(Style::new().fg(ratatui::style::Color::Blue))
        //        .graph_type(ratatui::widgets::GraphType::Bar),
        //])
        //.block(block)
        // .x_axis(Axis::default().style(Style::default()).bounds([0.0, 50.0]))
        //.y_axis(Axis::default().style(Style::default()).bounds([0.0, 50.0]));
    }
}
