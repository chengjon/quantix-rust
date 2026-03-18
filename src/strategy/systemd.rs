use std::path::PathBuf;
use std::process::Command;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::strategy::{JsonStrategyServiceConfigStore, StrategyServiceConfig};

const SERVICE_NAME: &str = "quantix-strategy.service";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyServiceStatusSummary {
    pub installed: bool,
    pub enabled: bool,
    pub active: String,
    pub unit_path: PathBuf,
    pub wrapper_path: PathBuf,
    pub quantix_bin_path: PathBuf,
    pub environment_file_path: Option<PathBuf>,
    pub raw_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StrategyUserServiceInstaller {
    runtime: CliRuntime,
    service_config: StrategyServiceConfig,
}

impl StrategyUserServiceInstaller {
    pub fn new(runtime: CliRuntime, service_config: StrategyServiceConfig) -> Self {
        Self {
            runtime,
            service_config,
        }
    }

    pub fn wrapper_path(&self) -> PathBuf {
        PathBuf::from("~/.local/bin/quantix-strategy-run")
    }

    pub fn unit_path(&self) -> PathBuf {
        PathBuf::from("~/.config/systemd/user").join(SERVICE_NAME)
    }

    fn resolved_wrapper_path(&self) -> Result<PathBuf> {
        Ok(home_dir()?.join(".local").join("bin").join("quantix-strategy-run"))
    }

    fn resolved_unit_path(&self) -> Result<PathBuf> {
        Ok(home_dir()?
            .join(".config")
            .join("systemd")
            .join("user")
            .join(SERVICE_NAME))
    }

    pub fn render_wrapper_script(&self) -> String {
        format!(
            "#!/bin/sh\nexec \"{}\" strategy daemon run\n",
            self.service_config.quantix_bin_path.display()
        )
    }

    pub fn render_unit(&self) -> String {
        let mut lines = vec![
            "[Unit]".to_string(),
            "Description=Quantix strategy signal daemon".to_string(),
            "After=network.target".to_string(),
            "".to_string(),
            "[Service]".to_string(),
            "Type=simple".to_string(),
            format!("ExecStart={}", self.wrapper_path().display()),
            "Restart=on-failure".to_string(),
            "RestartSec=5".to_string(),
            format!(
                "Environment=QUANTIX_STRATEGY_CONFIG_PATH={}",
                self.runtime.strategy_config_path.display()
            ),
            format!(
                "Environment=QUANTIX_STRATEGY_RUNTIME_DB_PATH={}",
                self.runtime.strategy_runtime_db_path.display()
            ),
        ];

        if let Some(env_file) = &self.service_config.environment_file_path {
            lines.push(format!("EnvironmentFile=-{}", env_file.display()));
        }

        lines.push(String::new());
        lines.push("[Install]".to_string());
        lines.push("WantedBy=default.target".to_string());
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

    pub fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
        Ok(StrategyServiceStatusSummary {
            installed: self.resolved_unit_path()?.exists(),
            enabled: self.run_systemctl_capture("is-enabled").is_ok(),
            active: if self.run_systemctl("is-active").is_ok() {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            unit_path: self.unit_path(),
            wrapper_path: self.wrapper_path(),
            quantix_bin_path: self.service_config.quantix_bin_path.clone(),
            environment_file_path: self.service_config.environment_file_path.clone(),
            raw_status: self.run_systemctl_capture("status").ok(),
        })
    }

    pub fn install(&self) -> Result<()> {
        JsonStrategyServiceConfigStore::validate(&self.service_config)?;

        let wrapper_path = self.resolved_wrapper_path()?;
        if let Some(parent) = wrapper_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let unit_path = self.resolved_unit_path()?;
        if let Some(parent) = unit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&wrapper_path, self.render_wrapper_script())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&wrapper_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&wrapper_path, perms)?;
        }

        if let Err(err) = std::fs::write(&unit_path, self.render_unit()) {
            let _ = std::fs::remove_file(&wrapper_path);
            return Err(err.into());
        }

        if let Err(err) = self.run_systemctl("daemon-reload") {
            let _ = std::fs::remove_file(&unit_path);
            let _ = std::fs::remove_file(&wrapper_path);
            return Err(err);
        }

        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        if self.run_systemctl("is-active").is_ok() {
            return Err(QuantixError::Other(
                "strategy service 仍在运行，请先执行 strategy service stop".to_string(),
            ));
        }

        let unit_path = self.resolved_unit_path()?;
        if unit_path.exists() {
            std::fs::remove_file(unit_path)?;
        }

        let wrapper_path = self.resolved_wrapper_path()?;
        if wrapper_path.exists() {
            std::fs::remove_file(wrapper_path)?;
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
        let summary = self.status_summary()?;
        let mut lines = vec![
            format!("installed: {}", yes_no(summary.installed)),
            format!("enabled: {}", yes_no(summary.enabled)),
            format!("active: {}", summary.active),
            format!("unit_path: {}", summary.unit_path.display()),
            format!("wrapper_path: {}", summary.wrapper_path.display()),
            format!("quantix_bin_path: {}", summary.quantix_bin_path.display()),
        ];

        if let Some(environment_file_path) = summary.environment_file_path {
            lines.push(format!(
                "environment_file_path: {}",
                environment_file_path.display()
            ));
        }

        if let Some(raw_status) = summary.raw_status {
            lines.push(String::new());
            lines.push(raw_status);
        }

        Ok(lines.join("\n"))
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

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| QuantixError::Config("HOME is required for strategy service install".into()))
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
