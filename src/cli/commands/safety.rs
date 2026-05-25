use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum SafetyCommands {
    /// 系统 kill switch
    #[command(subcommand)]
    KillSwitch(SafetyKillSwitchCommands),
}

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
