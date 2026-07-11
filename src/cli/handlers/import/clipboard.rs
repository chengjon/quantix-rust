use super::*;

/// 获取剪贴板内容
pub(super) fn get_clipboard_content() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            && output.status.success()
        {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        if let Ok(output) = std::process::Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
            && output.status.success()
        {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pbpaste").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    Err(QuantixError::Other("无法读取剪贴板内容，请确保已安装 xclip/xsel (Linux)、pbpaste (macOS) 或 PowerShell (Windows)".to_string()))
}
