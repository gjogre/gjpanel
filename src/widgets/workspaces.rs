use ratatui::{
    layout::{Alignment, Constraint, Direction, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::sync::mpsc::Sender;
use std::{
    collections::BTreeMap,
    env,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::mpsc,
    time::Duration,
};

use super::GJWidget;
use crate::config::WorkspacesConfig;

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

pub struct WorkspacesWidget {
    pub config: WorkspacesConfig,
    pub workspaces: Vec<Workspace>,
    connected: bool,
    tx_workspace: Option<std::sync::mpsc::Sender<Vec<Workspace>>>,
    rx_workspace: Option<std::sync::mpsc::Receiver<Vec<Workspace>>>,
    rx_event: Option<std::sync::mpsc::Receiver<()>>,
}

impl WorkspacesWidget {
    pub fn new(config: WorkspacesConfig) -> Self {
        let (tx_workspace, rx_workspace) = mpsc::channel::<Vec<Workspace>>();
        Self {
            config,
            workspaces: Vec::new(),
            connected: false,
            tx_workspace: Some(tx_workspace),
            rx_workspace: Some(rx_workspace),
            rx_event: None,
        }
    }

    fn handle_workspace_update(&mut self) {
        if let Ok(workspaces) = self.fetch_workspaces() {
            if let Ok(active_id_option) = self.fetch_active_workspace() {
                let active_id = active_id_option.unwrap_or(-1);

                self.workspaces = workspaces
                    .into_iter()
                    .map(|mut ws| {
                        ws.active = ws.id == active_id;
                        ws
                    })
                    .collect();
            } else {
                self.workspaces = workspaces;
            }
        }
    }

    fn fetch_workspaces(&mut self) -> Result<Vec<Workspace>, String> {
        let mut results = Vec::new();
        match Command::new("hyprctl").arg("workspaces").output() {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let workspace_blocks: Vec<&str> = output_str.split("\n\n").collect(); // Split by double newline

                for block in workspace_blocks {
                    if !block.trim().is_empty() {
                        match Workspace::from_str(block) {
                            Ok(wsp) => results.push(wsp),
                            Err(e) => eprintln!("Error parsing workspace block in fetch: {}", e),
                        }
                    }
                }
                Ok(results)
            }
            Ok(_) => Err("'hyprctl workspaces' failed.".to_string()),
            Err(err) => Err(format!("Error executing 'hyprctl workspaces': {}", err)),
        }
    }

    fn fetch_active_workspace(&mut self) -> Result<Option<i32>, String> {
        match Command::new("hyprctl").arg("activewindow").output() {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                match ActiveWorkspace::from_str(&output_str) {
                    Ok(active_ws) => Ok(Some(active_ws.id)),
                    Err(e) => {
                        eprintln!("Error parsing active workspace: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(_) => Ok(None), // No active window
            Err(err) => Err(format!("Error executing 'hyprctl activewindow': {}", err)),
        }
    }
}

impl GJWidget for WorkspacesWidget {
    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let mut grouped: BTreeMap<u32, Vec<Workspace>> = BTreeMap::new();
        for ws in &self.workspaces {
            let group = if ws.id < 0 { 10 } else { ws.monitor_id };
            grouped.entry(group as u32).or_default().push(ws.clone());
        }
        let column_width = 12;

        let column_start = column_width * grouped.len() as u16;
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double);

        let inner_area = outer_block.inner(area);

        frame.render_widget(outer_block, area);

        let column_constraints = vec![Constraint::Length(column_start); grouped.len()];
        let columns = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_constraints)
            .split(inner_area);

        for (i, (_monitor_id, column_workspaces)) in grouped.iter().rev().enumerate() {
            let mut column_workspaces = column_workspaces.clone();
            column_workspaces.sort_by_key(|ws| ws.id);
            let column_area = columns[i];
            let block_height = 3 as u16;

            for (j, ws) in column_workspaces.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if ws.active {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
                    .title(if ws.id.to_string() != ws.name {
                        ws.name.clone()
                    } else {
                        String::new()
                    });

                let paragraph = Paragraph::new(ws.id.to_string())
                    .alignment(Alignment::Center)
                    .block(block);
                let y = column_area.bottom() - (j as u16 + 1) * block_height;
                let rect = Rect::new(column_area.x, y, column_area.width, block_height);

                frame.render_widget(paragraph, rect);
            }
        }
    }

    fn poll(&mut self) {
        if !self.connected {
            self.connected = true;
            let (event_tx, event_rx) = mpsc::channel::<()>();
            self.rx_event = Some(event_rx);

            let (_workspace_tx, workspace_rx) = mpsc::channel::<Vec<Workspace>>();
            self.rx_workspace = Some(workspace_rx);

            let tx_workspaces = self.tx_workspace.clone();

            std::thread::spawn(move || {
                let mut socket = HyprSocketWorker {
                    event_tx,
                    tx: tx_workspaces.expect("Workspace sender should be initialized"),
                };
                if let Err(e) = socket.connect_hyprland_socket() {
                    eprintln!("Socket error in thread: {}", e);
                }
            });

            std::thread::sleep(Duration::from_millis(100));

            if let Some(rx) = &self.rx_workspace {
                if let Ok(initial_workspaces) = rx.try_recv() {
                    self.workspaces = initial_workspaces;
                }
            }
        }

        if let Some(rx) = self.rx_event.take() {
            while rx.try_recv().is_ok() {
                self.handle_workspace_update();
            }
            self.rx_event = Some(rx);
        }
    }
}

struct HyprSocketWorker {
    event_tx: Sender<()>,
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

        if let Ok(workspaces) = self.set_workspaces() {
            let _ = self.tx.send(workspaces);
        } else {
            eprintln!("HyprSocketWorker initial workspace fetch failed.");
        }

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    self.handle_socket_event(&line);
                }
                Err(e) => eprintln!("Failed to read line from socket: {}", e),
            }
        }
        Ok(())
    }
    fn handle_socket_event(&mut self, line: &str) {
        match line {
            l if l.starts_with("workspace>>")
                || l.starts_with("focusedmon>>")
                || l.starts_with("activewindow>>") =>
            {
                let _ = self.event_tx.send(());
            }
            _ => {}
        }
    }
    fn set_workspaces(&mut self) -> Result<Vec<Workspace>, String> {
        let mut results = Vec::new();
        match Command::new("hyprctl").arg("workspaces").output() {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let workspace_blocks: Vec<&str> = output_str.split("\n\n").collect(); // Split by double newline

                for block in workspace_blocks {
                    if !block.trim().is_empty() {
                        match Workspace::from_str(block) {
                            Ok(wsp) => results.push(wsp),
                            Err(e) => {
                                eprintln!("HyprSocketWorker error parsing workspace block: {}", e)
                            }
                        }
                    }
                }
                Ok(results)
            }
            Ok(_) => Err("'hyprctl workspaces' failed.".to_string()),
            Err(err) => {
                eprintln!(
                    "HyprSocketWorker error executing 'hyprctl workspaces': {}",
                    err
                );
                Err(format!(
                    "HyprSocketWorker error executing 'hyprctl workspaces': {}",
                    err
                ))
            }
        }
    }
}

impl FromStr for Workspace {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id: i32 = 0;
        let mut name = String::new();
        let mut monitor_id: u32 = 0;
        let mut active = false;

        for line in s.lines() {
            if line.starts_with("workspace ID") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(parsed_id) = parts[2].parse::<i32>() {
                        id = parsed_id;
                    } else {
                        eprintln!("Error parsing workspace ID: {}", parts[2]);
                    }
                    if parts[3].starts_with('(') && parts[3].ends_with(')') {
                        name = parts[3][1..parts[3].len() - 1].to_string();
                    } else {
                        name = parts[2].to_string();
                    }
                } else {
                    eprintln!("Error parsing workspace ID line: Not enough parts");
                }
            } else if line.starts_with("\tmonitorID:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(parsed_monitor_id) = parts[1].trim().parse::<u32>() {
                        monitor_id = parsed_monitor_id;
                    } else {
                        eprintln!("Error parsing '\\tmonitorID:' value");
                    }
                }
            } else if line.starts_with("\tactive:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    active = parts[1].trim() == "1";
                }
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
        let mut id: i32 = -1; // Initialize with a default invalid value

        for line in s.lines() {
            if line.starts_with("\tworkspace:") {
                if let Some(workspace_info) = line.split(':').nth(1).map(|s| s.trim()) {
                    // "5 (5)" or "5"
                    let parts: Vec<&str> = workspace_info.split_whitespace().collect();
                    if let Some(id_str) = parts.first() {
                        if let Ok(parsed_id) = id_str.parse::<i32>() {
                            id = parsed_id;
                            break;
                        } else {
                            return Err(format!("Invalid workspace ID format: {}", id_str));
                        }
                    } else {
                        return Err("Invalid format for workspace ID".to_string());
                    }
                } else {
                    return Err("Invalid format for workspace line".to_string());
                }
            }
        }

        if id != -1 {
            Ok(ActiveWorkspace { id })
        } else {
            Err("Workspace information not found in activewindow output".to_string())
        }
    }
}
