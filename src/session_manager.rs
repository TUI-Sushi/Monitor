use std::{cell::RefCell, collections::HashMap};

use openssh::{Error, Session};

#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn make() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub async fn new_connection(&mut self, ssh_con: String) {
        if let Ok(session) = Session::connect(ssh_con.clone(), openssh::KnownHosts::Strict).await {
            self.sessions.insert(ssh_con.clone(), session);
        }
    }

    pub async fn close_connection(&mut self, ssh_con: String) -> Result<(), Error> {
        if self.sessions.contains_key(&ssh_con) {
            let session = self.sessions.remove_entry(&ssh_con).unwrap();
            session.1.close().await?;
        }
        Ok(())
    }

    pub async fn close_all_connections(&mut self) {
        let keys: Vec<String> = self.sessions.keys().cloned().collect();

        for i in keys {
            self.close_connection(i).await.ok();
        }
    }

    pub async fn run_command(
        &mut self,
        ssh_con: String,
        command: String,
    ) -> Result<u64, tokio::time::error::Error> {
        if let Some(session) = self.sessions.get(&ssh_con) {
            if session.check().await.is_ok() {
                if let Ok(output) = session.raw_command(command).output().await {
                    return Ok::<u64, tokio::time::error::Error>(Self::extract_number_value(
                        output,
                    ));
                }
            } else {
                self.new_connection(ssh_con.clone()).await;
            }
        }

        Ok(0)
    }

    fn extract_number_value(output: std::process::Output) -> u64 {
        if let Ok(format_string) = String::from_utf8(output.stdout) {
            let float_num = format_string
                .strip_suffix("\r\n")
                .or(format_string.strip_suffix("\n"))
                .unwrap_or(format_string.as_str())
                .to_string()
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0);

            float_num as u64
        } else {
            0
        }
    }
}
