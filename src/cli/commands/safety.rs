use clap::Subcommand;

/// safety 命令族 clap 枚举（容器）：KillSwitch 系统 kill switch 子命令族（启用后阻止所有实盘执行路径）。
#[derive(Subcommand, Debug)]
pub enum SafetyCommands {
    /// 系统 kill switch
    #[command(subcommand)]
    KillSwitch(SafetyKillSwitchCommands),
}

/// safety kill-switch 子命令枚举：Enable 启用（阻止 target_mode 实盘执行）、Disable 关闭、Status 查看当前状态。
#[derive(Subcommand, Debug)]
pub enum SafetyKillSwitchCommands {
    /// 启用 kill switch
    Enable {
        /// 启用原因
        #[arg(long)]
        reason: String,
    },

    /// 关闭 kill switch
    Disable,

    /// 查看 kill switch 状态
    Status,
}
