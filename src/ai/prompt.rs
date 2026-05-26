//! Prompt template system
//!
//! Manage and render prompt templates for different analysis scenarios

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt template registry
#[derive(Debug, Clone, Default)]
pub struct PromptRegistry {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptRegistry {
    /// Create a new registry with default templates
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default templates
    fn register_defaults(&mut self) {
        // Stock analysis template
        self.register(PromptTemplate {
            name: "stock_analysis".to_string(),
            description: "Analyze a stock based on technical and fundamental data".to_string(),
            system_prompt: include_str!("templates/stock_analysis_system.md").to_string(),
            user_template: include_str!("templates/stock_analysis_user.md").to_string(),
            variables: vec![
                "code".to_string(),
                "name".to_string(),
                "price_data".to_string(),
                "indicators".to_string(),
                "news".to_string(),
            ],
        });

        // Decision template
        self.register(PromptTemplate {
            name: "trading_decision".to_string(),
            description: "Make a trading decision (buy/sell/hold)".to_string(),
            system_prompt: include_str!("templates/decision_system.md").to_string(),
            user_template: include_str!("templates/decision_user.md").to_string(),
            variables: vec![
                "code".to_string(),
                "current_position".to_string(),
                "analysis".to_string(),
                "risk_level".to_string(),
            ],
        });

        // Market overview template
        self.register(PromptTemplate {
            name: "market_overview".to_string(),
            description: "Generate market overview analysis".to_string(),
            system_prompt: include_str!("templates/market_overview_system.md").to_string(),
            user_template: include_str!("templates/market_overview_user.md").to_string(),
            variables: vec![
                "date".to_string(),
                "index_data".to_string(),
                "sector_performance".to_string(),
                "north_flow".to_string(),
            ],
        });
    }

    /// Register a new template
    pub fn register(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// Render a template with variables
    pub fn render(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Option<(String, String)> {
        self.templates.get(name).map(|t| {
            let system = t.render_system(variables);
            let user = t.render_user(variables);
            (system, user)
        })
    }

    /// List available templates
    pub fn list(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

/// A prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// System prompt template
    pub system_prompt: String,
    /// User message template
    pub user_template: String,
    /// Required variables
    pub variables: Vec<String>,
}

impl PromptTemplate {
    /// Render the system prompt with variables
    pub fn render_system(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.system_prompt.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }

    /// Render the user message with variables
    pub fn render_user(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.user_template.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_registry() {
        let registry = PromptRegistry::new();
        let templates = registry.list();
        assert!(templates.contains(&"stock_analysis"));
        assert!(templates.contains(&"trading_decision"));
        assert!(templates.contains(&"market_overview"));
    }
}
