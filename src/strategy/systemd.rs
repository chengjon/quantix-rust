use std::path::PathBuf;
use std::process::Command;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::strategy::{JsonStrategyServiceConfigStore, StrategyServiceConfig};

const SERVICE_NAME: &str = "quantix-strategy.service";

/// `systemctl status` 等命令聚合的状态摘要，用于 CLI 输出与测试断言。
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

/// 负责将 strategy 守护进程安装为 systemd `--user` unit 的安装器，
/// 持有 CLI 运行时路径与服务配置。
#[derive(Debug, Clone)]
pub struct StrategyUserServiceInstaller {
    runtime: CliRuntime,
    service_config: StrategyServiceConfig,
}

impl StrategyUserServiceInstaller {
    /// 用给定的 CLI 运行时与服务配置构造安装器。
    pub fn new(runtime: CliRuntime, service_config: StrategyServiceConfig) -> Self {
        Self {
            runtime,
            service_config,
        }
    }

    /// 返回用于展示的 wrapper 脚本路径（带 `~` 前缀，未展开）。
    pub fn wrapper_path(&self) -> PathBuf {
        PathBuf::from("~/.local/bin/quantix-strategy-run")
    }

    /// 返回用于展示的 unit 文件路径（带 `~` 前缀，未展开）。
    pub fn unit_path(&self) -> PathBuf {
        PathBuf::from("~/.config/systemd/user").join(SERVICE_NAME)
    }

    /// 解析 `$HOME` 后返回 wrapper 脚本的实际路径；`HOME` 缺失时返回 `Config` 错误。
    fn resolved_wrapper_path(&self) -> Result<PathBuf> {
        Ok(home_dir()?
            .join(".local")
            .join("bin")
            .join("quantix-strategy-run"))
    }

    /// 解析 `$HOME` 后返回 unit 文件的实际路径；`HOME` 缺失时返回 `Config` 错误。
    fn resolved_unit_path(&self) -> Result<PathBuf> {
        Ok(home_dir()?
            .join(".config")
            .join("systemd")
            .join("user")
            .join(SERVICE_NAME))
    }

    /// 渲染 wrapper shell 脚本内容：`exec <quantix_bin> strategy daemon run`。
    pub fn render_wrapper_script(&self) -> String {
        format!(
            "#!/bin/sh\nexec \"{}\" strategy daemon run\n",
            self.service_config.quantix_bin_path.display()
        )
    }

    /// 渲染 systemd unit 文件内容，注入 strategy_config_path、strategy_runtime_db_path 以及可选的 environment_file。
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

    /// 生成调用 `systemctl --user` 的参数列表：`daemon-reload` 不附带 unit 名，其余 action 附加 `SERVICE_NAME`。
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

    /// 汇总 unit 当前状态：是否已安装/已启用/已激活，环境文件路径，以及原始 `systemctl status` 文本。
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

    /// 写入 wrapper 脚本与 unit 文件并执行 `daemon-reload`，随后校验 `LoadState`；失败时回滚已写入文件。
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
            if let Err(cleanup_err) = std::fs::remove_file(&wrapper_path) {
                tracing::warn!("回滚删除 wrapper 失败: {}", cleanup_err);
            }
            return Err(err.into());
        }

        if let Err(err) = self.run_systemctl("daemon-reload") {
            if let Err(cleanup_err) = std::fs::remove_file(&unit_path) {
                tracing::warn!("回滚删除 unit 失败: {}", cleanup_err);
            }
            if let Err(cleanup_err) = std::fs::remove_file(&wrapper_path) {
                tracing::warn!("回滚删除 wrapper 失败: {}", cleanup_err);
            }
            return Err(err);
        }

        let load_state = self.run_systemctl_show_load_state()?;
        validate_unit_load_state(&load_state, &unit_path)?;

        Ok(())
    }

    /// 删除 wrapper 与 unit，然后 `daemon-reload`；若服务仍在运行则拒绝并返回错误。
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

    /// 执行 `systemctl --user start`。
    pub fn start(&self) -> Result<()> {
        self.run_systemctl("start")
    }

    /// 执行 `systemctl --user stop`。
    pub fn stop(&self) -> Result<()> {
        self.run_systemctl("stop")
    }

    /// 执行 `systemctl --user enable`，让开机自启动生效。
    pub fn enable(&self) -> Result<()> {
        self.run_systemctl("enable")
    }

    /// 执行 `systemctl --user disable`，取消开机自启动。
    pub fn disable(&self) -> Result<()> {
        self.run_systemctl("disable")
    }

    /// 生成供 CLI 打印的多行状态文本，包含 installed/enabled/active 等关键字段与原始 status 输出。
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

    fn run_systemctl_show_load_state(&self) -> Result<String> {
        let output = Command::new("systemctl")
            .args([
                "--user",
                "show",
                SERVICE_NAME,
                "--property=LoadState",
                "--value",
            ])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(QuantixError::Other(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }
}

fn validate_unit_load_state(load_state: &str, unit_path: &std::path::Path) -> Result<()> {
    if load_state == "not-found" {
        return Err(QuantixError::Other(format!(
            "strategy service 文件已写入 {}，但当前 systemd --user 会话没有识别该 unit (LoadState=not-found)",
            unit_path.display()
        )));
    }

    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| QuantixError::Config("HOME is required for strategy service install".into()))
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::validate_unit_load_state;

    #[test]
    fn validate_unit_load_state_accepts_loaded_unit() {
        let result = validate_unit_load_state(
            "loaded",
            std::path::Path::new("/tmp/quantix-strategy.service"),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn validate_unit_load_state_reports_not_found_unit_with_path() {
        let path = std::path::Path::new("/tmp/quantix-strategy.service");
        let err = validate_unit_load_state("not-found", path).unwrap_err();

        assert!(err.to_string().contains("LoadState=not-found"));
        assert!(err.to_string().contains(path.to_string_lossy().as_ref()));
    }
}
