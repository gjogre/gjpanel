use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::Command,
    str::FromStr,
};

use crate::config::WorkspacesConfig;

pub struct WorkspacesWidget {
    pub config: WorkspacesConfig,
    pub workspaces: Vec<Workspace>,
}

impl WorkspacesWidget {
    pub fn new(config: WorkspacesConfig) -> Self {
        Self {
            config,
            workspaces: Vec::new(),
        }
    }
}
trait HyprSocket {
    fn connect_hyprland_socket(&mut self) -> Result<(), String>;
    fn handle_socket_event(&mut self, line: &str);
    fn set_workspaces(&mut self) -> Result<(), String>;
}
#[derive(Debug)]
struct Workspace {
    id: i32,
    name: String,
    monitor_id: u32,
    active: bool,
}
#[derive(Debug)]
struct ActiveWorkspace {
    id: i32,
}

impl HyprSocket for WorkspacesWidget {
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
        Ok(())
    }
    fn handle_socket_event(&mut self, line: &str) {
        match line {
            l if l.starts_with("workspace>>") => {
                if let Err(e) = self.set_workspaces() {
                    eprintln!("Failed to handle event: {}", e);
                }
            }
            l if l.starts_with("focusedmon>>") => {
                if let Err(e) = self.set_workspaces() {
                    eprintln!("Failed to handle event: {}", e);
                }
            }
            l if l.starts_with("activewindow>>") => {
                if let Err(e) = self.set_workspaces() {
                    eprintln!("Failed to handle event: {}", e);
                }
            }
            _ => println!("Unknown line: {}", line),
        }
    }

    fn set_workspaces(&mut self) -> Result<(), String> {
        match Command::new("hyprctl").arg("workspaces").output() {
            Ok(output) if output.status.success() => {
                let output_str = match String::from_utf8(output.stdout) {
                    Ok(s) => s,
                    Err(e) => return Err(e.to_string()),
                };
                let workspaces = output_str
                    .split("workspace ID")
                    .skip(1) // Skip the first empty split
                    .map(|s| Workspace::from_str(&format!("workspace ID{}", s)))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Failed to parse one or more workspaces".to_string())?;
                self.workspaces = workspaces;
                Ok(())
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

        for line in s.lines() {
            match line {
                l if line.starts_with("workspace ID") => {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    id = parts[2].parse().map_err(|_| "Invalid ID".to_string())?;
                    name = parts[3].to_string();
                }
                l if line.starts_with("\tmonitorID:") => {
                    monitor_id = l
                        .split(":")
                        .nth(1)
                        .unwrap()
                        .parse()
                        .map_err(|_| "Invalid monitor ID".to_string())?;
                }
                _ => return Err("Invalid format".to_string()),
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
