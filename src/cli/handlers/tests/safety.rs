use super::*;
use crate::safety::JsonKillSwitchStore;

#[test]
fn test_execute_safety_kill_switch_status_defaults_to_disabled_when_missing() {
    let dir = tempdir().unwrap();
    let store = JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    let now = Utc.with_ymd_and_hms(2026, 5, 6, 10, 0, 0).unwrap();

    let state = execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Status,
        &store,
        now,
    )
    .unwrap();

    assert!(!state.enabled);
    assert_eq!(state.reason, None);
    assert_eq!(state.enabled_at, None);
    assert_eq!(state.disabled_at, None);
    assert_eq!(state.updated_by, "cli");
}

#[test]
fn test_execute_safety_kill_switch_enable_persists_reason_and_timestamp() {
    let dir = tempdir().unwrap();
    let store = JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    let now = Utc.with_ymd_and_hms(2026, 5, 6, 10, 0, 0).unwrap();

    let state = execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &store,
        now,
    )
    .unwrap();

    assert!(state.enabled);
    assert_eq!(state.reason.as_deref(), Some("broker instability"));
    assert_eq!(state.enabled_at, Some(now));
    assert_eq!(state.disabled_at, None);
    assert_eq!(state.updated_by, "cli");

    let saved = store.load().unwrap();
    assert_eq!(saved, state);
}

#[test]
fn test_execute_safety_kill_switch_disable_turns_off_but_preserves_last_enable_context() {
    let dir = tempdir().unwrap();
    let store = JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    let enabled_at = Utc.with_ymd_and_hms(2026, 5, 6, 10, 0, 0).unwrap();
    let disabled_at = Utc.with_ymd_and_hms(2026, 5, 6, 10, 5, 0).unwrap();

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &store,
        enabled_at,
    )
    .unwrap();

    let state = execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Disable,
        &store,
        disabled_at,
    )
    .unwrap();

    assert!(!state.enabled);
    assert_eq!(state.reason.as_deref(), Some("broker instability"));
    assert_eq!(state.enabled_at, Some(enabled_at));
    assert_eq!(state.disabled_at, Some(disabled_at));
    assert_eq!(state.updated_by, "cli");

    let saved = store.load().unwrap();
    assert_eq!(saved, state);
}
