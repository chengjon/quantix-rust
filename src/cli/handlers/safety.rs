use crate::cli::command_types::{SafetyCommands, SafetyKillSwitchCommands};
use crate::core::Result;
use crate::safety::{JsonKillSwitchStore, KillSwitchState};
use chrono::{DateTime, Utc};

pub(crate) fn run_safety_command(cmd: SafetyCommands) -> Result<()> {
    match cmd {
        SafetyCommands::KillSwitch(kill_switch_cmd) => {
            let store = JsonKillSwitchStore::with_default_path()?;
            let state = execute_safety_kill_switch_command_with_store_at(
                kill_switch_cmd,
                &store,
                Utc::now(),
            )?;
            println!("{}", serde_json::to_string_pretty(&state)?);
            Ok(())
        }
    }
}

pub(crate) fn execute_safety_kill_switch_command_with_store_at(
    cmd: SafetyKillSwitchCommands,
    store: &JsonKillSwitchStore,
    now: DateTime<Utc>,
) -> Result<KillSwitchState> {
    match cmd {
        SafetyKillSwitchCommands::Status => store.load_or_default(),
        SafetyKillSwitchCommands::Enable { reason } => {
            let mut state = store.load_or_default()?;
            state.enabled = true;
            state.reason = Some(reason);
            state.enabled_at = Some(now);
            state.disabled_at = None;
            state.updated_by = "cli".to_string();
            store.save(&state)?;
            Ok(state)
        }
        SafetyKillSwitchCommands::Disable => {
            let mut state = store.load_or_default()?;
            state.enabled = false;
            state.disabled_at = Some(now);
            state.updated_by = "cli".to_string();
            store.save(&state)?;
            Ok(state)
        }
    }
}
