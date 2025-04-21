use ratatui::{
    layout::{Alignment, Constraint, Direction},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::mpsc,
};

use crate::config::WorkspacesConfig;

use super::GJWidget;

pub struct WorkspacesWidget {
    pub config: WorkspacesConfig,
    pub workspaces: Vec<Workspace>,
    connected: bool,
    rx: Option<std::sync::mpsc::Receiver<Vec<Workspace>>>,
}

impl WorkspacesWidget {
    pub fn new(config: WorkspacesConfig) -> Self {
        Self {
            config,
            workspaces: Vec::new(),
            connected: false,
            rx: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    id: i32,
    name: String,
    monitor_id: u32,
    active: bool,
}
#[derive(Debug)]
pub struct ActiveWorkspace {
    id: i32,
}

impl GJWidget for WorkspacesWidget {
    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        // Group by monitor_id
        let mut grouped: BTreeMap<u32, Vec<Workspace>> = BTreeMap::new();
        for ws in &self.workspaces {
            grouped.entry(ws.monitor_id).or_default().push(ws.clone());
        }

        let column_constraints = vec![Constraint::Length(12); grouped.len()];
        let columns = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_constraints)
            .split(area);

        for (i, (_monitor_id, column_workspaces)) in grouped.iter().enumerate() {
            let column_height = column_workspaces.len();
            let rows = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3); column_height])
                .split(columns[i]);

            for (j, ws) in column_workspaces.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if ws.active {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default().fg(Color::Gray)
                    });

                let paragraph = Paragraph::new(ws.id.to_string())
                    .alignment(Alignment::Center)
                    .block(block);

                frame.render_widget(paragraph, rows[j]);
            }
        }
    }
    fn poll(&mut self) {
        if !self.connected {
            self.connected = true;
            let (tx, rx) = mpsc::channel::<Vec<Workspace>>();
            self.rx = Some(rx);

            std::thread::spawn(move || {
                let mut socket = HyprSocketWorker { tx };
                if let Err(e) = socket.connect_hyprland_socket() {
                    eprintln!("Socket error: {}", e);
                }
            });
        }

        if let Some(rx) = &self.rx {
            while let Ok(thread_workspaces) = rx.try_recv() {
                self.workspaces = thread_workspaces;
            }
        }
    }
}
struct HyprSocketWorker {
    tx: Sender<Vec<Workspace>>,
}

trait HyprSocket {
    fn connect_hyprland_socket(&mut self) -> Result<(), String>;
    fn handle_socket_event(&mut self, line: &str);
    fn set_workspaces(&mut self) -> Result<Vec<Workspace>, String>;
}

impl HyprSocket for HyprSocketWorker {
    fn connect_hyprland_socket(&mut self) -> Result<(), String> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");

        let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE is not set");

        let mut socket_path = PathBuf::from(xdg_runtime_dir);
        socket_path.push("hypr");
        socket_path.push(hyprland_instance_signature);
        socket_path.push(".socket2.sock");

        let stream = match UnixStream::connect(&socket_path) {
            Ok(stream) => stream,
            Err(e) => return Err(format!("Failed to connect to socket: {}", e)),
        };

        let reader = BufReader::new(stream);

        for line in reader.lines() {
            match line {
                Ok(line) => self.handle_socket_event(&line),
                Err(e) => eprintln!("Failed to read line: {}", e),
            }
        }
        // send right away one set of workspaces
        let initial = self.set_workspaces();
        if let Ok(workspaces) = initial {
            let _ = self.tx.send(workspaces);
        }
        Ok(())
    }
    fn handle_socket_event(&mut self, line: &str) {
        match line {
            l if l.starts_with("workspace>>")
                || l.starts_with("focusedmon>>")
                || l.starts_with("activewindow>>") =>
            {
                match self.set_workspaces() {
                    Ok(workspaces) => {
                        let _ = self.tx.send(workspaces);
                    }
                    Err(e) => eprintln!("Failed to handle event: {}", e),
                }
            }
            _ => (),
        }
    }

    fn set_workspaces(&mut self) -> Result<Vec<Workspace>, String> {
        let mut results = Vec::new();
        match Command::new("hyprctl").arg("workspaces").output() {
            Ok(output) if output.status.success() => {
                let output_str = match String::from_utf8(output.stdout) {
                    Ok(s) => s,
                    Err(e) => return Err(e.to_string()),
                };
                let workspaces = output_str.split("workspace ID");
                for ws in workspaces.skip(1) {
                    match Workspace::from_str(ws) {
                        Ok(wsp) => results.push(wsp),
                        Err(e) => return Err(e),
                    }
                }
                Ok(results)
            }
            Ok(_) => Err("Command executed but failed".to_string()),
            Err(err) => Err(format!("Command execution failed: {}", err)),
        }
    }
}

impl FromStr for Workspace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = 0;
        let mut name = String::new();
        let mut monitor_id = 0;
        let active = false;

        for line in s.lines().skip(1) {
            match line {
                l if line.starts_with("workspace ID") => {
                    let parts: Vec<&str> = line.split_whitespace().collect();

                    id = parts[2].parse().map_err(|_| "Invalid ID".to_string())?;
                    name = parts[3].to_string();
                }
                l if line.starts_with("\tmonitorID:") => {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() < 2 {
                        return Err("Invalid monitorID line format".to_string());
                    }
                    monitor_id = parts[1]
                        .trim()
                        .parse()
                        .map_err(|_| "Invalid monitor ID".to_string())?;
                }
                _ => (),
            }
        }
        Ok(Workspace {
            id,
            name,
            monitor_id,
            active,
        })
    }
}
impl FromStr for ActiveWorkspace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = 0;

        for line in s.lines() {
            match line {
                l if line.starts_with("\tworkspace:") => {
                    id = l
                        .split(":")
                        .nth(1)
                        .unwrap()
                        .parse()
                        .map_err(|_| "Invalid monitor ID".to_string())?;
                }
                _ => return Err("Invalid format".to_string()),
            }
        }
        Ok(ActiveWorkspace { id })
    }
}
