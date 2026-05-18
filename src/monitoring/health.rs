//! Health check module for monitoring system
//!
//! Provides health status reporting for system components:
//! - Database connections
//! - Data sources
//! - Execution adapters
//! - Strategy runtime

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// When this health check was performed
    pub checked_at: DateTime<Utc>,
    /// Overall status
    pub status: HealthStatus,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Health status level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// All components healthy
    Healthy,
    /// Some components degraded but operational
    Degraded,
    /// Critical components unhealthy
    Unhealthy,
}

impl HealthStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }

    /// Combine multiple statuses, taking the worst
    pub fn combine(iter: impl Iterator<Item = Self>) -> Self {
        let mut has_degraded = false;
        for status in iter {
            match status {
                Self::Unhealthy => return Self::Unhealthy,
                Self::Degraded => has_degraded = true,
                Self::Healthy => {}
            }
        }
        if has_degraded {
            Self::Degraded
        } else {
            Self::Healthy
        }
    }
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Last successful operation
    pub last_success_at: Option<DateTime<Utc>>,
    /// Last error message if any
    pub last_error: Option<String>,
    /// Additional details
    pub details: HashMap<String, String>,
    /// Response time in milliseconds (if applicable)
    pub response_time_ms: Option<u64>,
}

impl ComponentHealth {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            last_success_at: Some(Utc::now()),
            last_error: None,
            details: HashMap::new(),
            response_time_ms: None,
        }
    }

    pub fn degraded(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            last_success_at: None,
            last_error: Some(error.into()),
            details: HashMap::new(),
            response_time_ms: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            last_success_at: None,
            last_error: Some(error.into()),
            details: HashMap::new(),
            response_time_ms: None,
        }
    }

    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = Some(ms);
        self
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Health checker trait for components
pub trait HealthCheck: Send + Sync {
    /// Component name
    fn name(&self) -> &str;

    /// Perform health check
    fn check(&self) -> impl std::future::Future<Output = ComponentHealth> + Send;
}

/// Health check registry
pub struct HealthRegistry {
    components: HashMap<String, ComponentHealth>,
    start_time: DateTime<Utc>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            start_time: Utc::now(),
        }
    }

    /// Register a component's health status
    pub fn register(&mut self, health: ComponentHealth) {
        self.components.insert(health.name.clone(), health);
    }

    /// Update a component's health status
    pub fn update(&mut self, health: ComponentHealth) {
        self.components.insert(health.name.clone(), health);
    }

    /// Get overall system health
    pub fn system_health(&self) -> SystemHealth {
        let status = HealthStatus::combine(self.components.values().map(|c| c.status));
        let uptime = (Utc::now() - self.start_time).num_seconds() as u64;

        SystemHealth {
            checked_at: Utc::now(),
            status,
            components: self.components.clone(),
            uptime_seconds: uptime,
        }
    }

    /// Get health for a specific component
    pub fn get(&self, name: &str) -> Option<&ComponentHealth> {
        self.components.get(name)
    }

    /// List all component names
    pub fn component_names(&self) -> Vec<&str> {
        self.components.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_combine() {
        assert_eq!(
            HealthStatus::combine([HealthStatus::Healthy, HealthStatus::Healthy].into_iter()),
            HealthStatus::Healthy
        );

        assert_eq!(
            HealthStatus::combine([HealthStatus::Healthy, HealthStatus::Degraded].into_iter()),
            HealthStatus::Degraded
        );

        assert_eq!(
            HealthStatus::combine([HealthStatus::Healthy, HealthStatus::Unhealthy].into_iter()),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_component_health_builder() {
        let health = ComponentHealth::healthy("test")
            .with_response_time(50)
            .with_detail("version", "1.0.0");

        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.response_time_ms, Some(50));
        assert_eq!(health.details.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_health_registry() {
        let mut registry = HealthRegistry::new();
        registry.register(ComponentHealth::healthy("db"));
        registry.register(ComponentHealth::degraded("api", "timeout"));

        let system = registry.system_health();
        assert_eq!(system.status, HealthStatus::Degraded);
    }
}
