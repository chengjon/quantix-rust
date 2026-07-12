use super::commands::parse_account_type_input;
use crate::account::AccountType;
use crate::core::QuantixError;

#[test]
fn parse_account_type_prefers_qmt_live_wording_but_keeps_live_alias() {
    assert_eq!(
        parse_account_type_input("paper").unwrap(),
        AccountType::Paper
    );
    assert_eq!(
        parse_account_type_input("mock_live").unwrap(),
        AccountType::MockLive
    );
    assert_eq!(
        parse_account_type_input("qmt_live").unwrap(),
        AccountType::Live
    );
    assert_eq!(parse_account_type_input("live").unwrap(), AccountType::Live);
}

#[test]
fn parse_account_type_error_mentions_qmt_live_boundary() {
    let err = parse_account_type_input("invalid").unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("无效的账户类型: invalid"));
    assert!(message.contains("paper, mock_live, qmt_live"));
    assert!(message.contains("live"));
}
