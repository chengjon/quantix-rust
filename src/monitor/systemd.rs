use std::path::PathBuf;
use std::process::Command;

use crate::core::{CliRuntime, QuantixError, Result};

const SERVICE_NAME: &str = "quantix-monitor.service";

#[derive(Debug, Clone)]
pub struct MonitorUserServiceInstaller {
    runtime: CliRuntime,
    executable_path: PathBuf,
}

impl MonitorUserServiceInstaller {
    pub fn new(runtime: CliRuntime, executable_path: PathBuf) -> Self {
        Self {
            runtime,
            executable_path,
        }
    }

    pub fn unit_path(&self) -> PathBuf {
        PathBuf::from("~/.config/systemd/user").join(SERVICE_NAME)
    }

    fn resolved_unit_path(&self) -> Result<PathBuf> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| QuantixError::Config("HOME is required for systemd --user".into()))?;
        Ok(home.join(".config").join("systemd").join("user").join(SERVICE_NAME))
    }

    pub fn render_unit(&self) -> String {
        let mut lines = vec![
            "[Unit]".to_string(),
            "Description=Quantix monitor daemon".to_string(),
            "After=network.target".to_string(),
            "".to_string(),
            "[Service]".to_string(),
            "Type=simple".to_string(),
            format!(
                "ExecStart={} monitor daemon run",
                self.executable_path.display()
            ),
            "Restart=on-failure".to_string(),
            "RestartSec=5".to_string(),
            format!(
                "Environment=QUANTIX_WATCHLIST_PATH={}",
                self.runtime.watchlist_path.display()
            ),
            format!(
                "Environment=QUANTIX_MONITOR_DB_PATH={}",
                self.runtime.monitor_db_path.display()
            ),
            format!(
                "Environment=QUANTIX_MONITOR_CONFIG_PATH={}",
                self.runtime.monitor_config_path.display()
            ),
            format!(
                "Environment=QUANTIX_TRADE_PATH={}",
                self.runtime.trade_path.display()
            ),
            format!(
                "Environment=QUANTIX_RISK_PATH={}",
                self.runtime.risk_path.display()
            ),
            "".to_string(),
            "[Install]".to_string(),
            "WantedBy=default.target".to_string(),
        ];
        lines.push(String::new());
        lines.join("\n")
    }

    pub fn systemctl_args(&self, action: &str) -> Vec<String> {
        if action == "daemon-reload" {
            vec!["--user".to_string(), "daemon-reload".to_string()]
        } else {
            vec![
                "--user".to_string(),
                action.to_string(),
                SERVICE_NAME.to_string(),
            ]
        }
    }

    pub fn install(&self) -> Result<()> {
        let unit_path = self.resolved_unit_path()?;
        if let Some(parent) = unit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&unit_path, self.render_unit())?;
        self.run_systemctl("daemon-reload")?;
        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        let unit_path = self.resolved_unit_path()?;
        if unit_path.exists() {
            std::fs::remove_file(unit_path)?;
        }
        self.run_systemctl("daemon-reload")?;
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        self.run_systemctl("start")
    }

    pub fn stop(&self) -> Result<()> {
        self.run_systemctl("stop")
    }

    pub fn enable(&self) -> Result<()> {
        self.run_systemctl("enable")
    }

    pub fn disable(&self) -> Result<()> {
        self.run_systemctl("disable")
    }

    pub fn status(&self) -> Result<String> {
        self.run_systemctl_capture("status")
    }

    fn run_systemctl(&self, action: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args(self.systemctl_args(action))
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(QuantixError::Other(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    fn run_systemctl_capture(&self, action: &str) -> Result<String> {
        let output = Command::new("systemctl")
            .args(self.systemctl_args(action))
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(QuantixError::Other(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }
}
