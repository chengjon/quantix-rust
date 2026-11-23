//! FinError 收敛点：`QuantixError::Fin` 的 From 实现与验证。

#[cfg(test)]
mod tests {
    use crate::core::QuantixError;

    #[test]
    fn fin_error_maps_into_quantix_error() {
        let err = fin_primitives::FinError::InvalidInput("bad".to_string());
        let mapped: QuantixError = err.into();
        let text = mapped.to_string();
        assert!(
            text.contains("fin-primitives"),
            "display 应保留前缀: {text}"
        );
        assert!(text.contains("bad"));
        assert!(
            matches!(mapped, QuantixError::Fin(fin_primitives::FinError::InvalidInput(s)) if s == "bad")
        );
    }
}
