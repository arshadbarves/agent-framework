// Specialized agent roles for AgentGraph
// Provides pre-configured agent roles with specific capabilities and behaviors

#![allow(missing_docs)]

use super::{AgentConfig, AgentRole};
use crate::agents::memory::MemoryConfig;
use crate::agents::collaboration::CollaborationConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Role template for creating specialized agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplate {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// System prompt template
    pub system_prompt: String,
    /// Recommended tools
    pub tools: Vec<String>,
    /// Recommended model
    pub model: String,
    /// Temperature setting
    pub temperature: f32,
    /// Max tokens
    pub max_tokens: u32,
    /// Memory configuration
    pub memory_config: MemoryConfig,
    /// Collaboration configuration
    pub collaboration_config: CollaborationConfig,
}

impl RoleTemplate {
    /// Create agent config from template
    pub fn to_agent_config(&self, name: String, provider: String) -> AgentConfig {
        AgentConfig {
            name,
            role: AgentRole::Custom(self.description.clone()),
            model: self.model.clone(),
            provider,
            system_prompt: self.system_prompt.clone(),
            max_tokens: Some(self.max_tokens),
            temperature: Some(self.temperature),
            available_tools: self.tools.clone(),
            memory_config: self.memory_config.clone(),
            collaboration_config: self.collaboration_config.clone(),
        }
    }
}

/// Predefined role templates
pub struct RoleTemplates;

impl RoleTemplates {
    /// Software Developer Agent
    pub fn software_developer() -> RoleTemplate {
        RoleTemplate {
            name: "Software Developer".to_string(),
            description: "Expert software developer specializing in code generation, review, and debugging".to_string(),
            system_prompt: r#"You are an expert software developer with deep knowledge across multiple programming languages and frameworks. Your responsibilities include:

1. Writing clean, efficient, and well-documented code
2. Reviewing code for bugs, security issues, and best practices
3. Debugging complex issues and providing solutions
4. Suggesting architectural improvements
5. Following coding standards and best practices

You have access to file operations, web research, and calculation tools. Always:
- Write secure and maintainable code
- Include proper error handling
- Add meaningful comments and documentation
- Consider performance implications
- Follow language-specific conventions"#.to_string(),
            tools: vec![
                "file_read".to_string(),
                "file_write".to_string(),
                "file_list".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
                "calculator".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.3, // Lower temperature for more consistent code
            max_tokens: 2000,
            memory_config: MemoryConfig {
                max_short_term_entries: 100,
                max_long_term_entries: 2000,
                retention_period: Duration::from_secs(86400 * 7), // 1 week
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: true,
                ..Default::default()
            },
        }
    }

    /// Research Analyst Agent
    pub fn research_analyst() -> RoleTemplate {
        RoleTemplate {
            name: "Research Analyst".to_string(),
            description: "Expert researcher specializing in information gathering, analysis, and synthesis".to_string(),
            system_prompt: r#"You are a skilled research analyst with expertise in gathering, analyzing, and synthesizing information from various sources. Your responsibilities include:

1. Conducting thorough research on given topics
2. Analyzing data and identifying patterns and trends
3. Synthesizing information from multiple sources
4. Creating comprehensive reports and summaries
5. Fact-checking and verifying information accuracy

You have access to web research, file operations, and text analysis tools. Always:
- Verify information from multiple sources
- Cite your sources and provide references
- Present balanced and objective analysis
- Identify potential biases or limitations
- Structure information clearly and logically"#.to_string(),
            tools: vec![
                "http_get".to_string(),
                "http_post".to_string(),
                "text_search".to_string(),
                "text_summarize".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
                "calculator".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.5,
            max_tokens: 2500,
            memory_config: MemoryConfig {
                max_short_term_entries: 150,
                max_long_term_entries: 3000,
                retention_period: Duration::from_secs(86400 * 14), // 2 weeks
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: false, // Researchers typically work independently
                ..Default::default()
            },
        }
    }

    /// Data Scientist Agent
    pub fn data_scientist() -> RoleTemplate {
        RoleTemplate {
            name: "Data Scientist".to_string(),
            description: "Expert data scientist specializing in data analysis, modeling, and insights".to_string(),
            system_prompt: r#"You are an expert data scientist with deep knowledge in statistics, machine learning, and data analysis. Your responsibilities include:

1. Analyzing datasets to extract meaningful insights
2. Building and validating predictive models
3. Creating data visualizations and reports
4. Identifying data quality issues and cleaning data
5. Communicating findings to stakeholders

You have access to database queries, file operations, and calculation tools. Always:
- Validate data quality and handle missing values appropriately
- Use appropriate statistical methods and models
- Interpret results in business context
- Consider ethical implications of data use
- Document methodology and assumptions clearly"#.to_string(),
            tools: vec![
                "database_query".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
                "calculator".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.4,
            max_tokens: 2000,
            memory_config: MemoryConfig {
                max_short_term_entries: 80,
                max_long_term_entries: 1500,
                retention_period: Duration::from_secs(86400 * 10), // 10 days
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: true,
                consensus_enabled: true, // Data scientists often need consensus on methodologies
                ..Default::default()
            },
        }
    }

    /// Content Writer Agent
    pub fn content_writer() -> RoleTemplate {
        RoleTemplate {
            name: "Content Writer".to_string(),
            description: "Creative writer specializing in engaging, well-structured content across various formats".to_string(),
            system_prompt: r#"You are a skilled content writer with expertise in creating engaging, well-structured content across various formats and styles. Your responsibilities include:

1. Writing compelling and engaging content for different audiences
2. Adapting tone and style to match brand voice and target audience
3. Researching topics to ensure accuracy and relevance
4. Optimizing content for readability and engagement
5. Proofreading and editing for grammar, style, and clarity

You have access to research, file operations, and text analysis tools. Always:
- Write in clear, engaging language appropriate for the audience
- Structure content with proper headings and flow
- Include relevant examples and supporting details
- Maintain consistency in tone and style
- Fact-check information and cite sources when needed"#.to_string(),
            tools: vec![
                "text_search".to_string(),
                "text_summarize".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
                "http_get".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.8, // Higher temperature for creativity
            max_tokens: 2500,
            memory_config: MemoryConfig {
                max_short_term_entries: 60,
                max_long_term_entries: 1000,
                retention_period: Duration::from_secs(86400 * 5), // 5 days
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: false, // Writers typically work independently
                ..Default::default()
            },
        }
    }

    /// Project Manager Agent
    pub fn project_manager() -> RoleTemplate {
        RoleTemplate {
            name: "Project Manager".to_string(),
            description: "Strategic planner focused on project coordination, timeline management, and resource allocation".to_string(),
            system_prompt: r#"You are an experienced project manager with expertise in planning, coordinating, and managing complex projects. Your responsibilities include:

1. Breaking down complex projects into manageable tasks
2. Creating realistic timelines and milestones
3. Coordinating resources and team members
4. Monitoring progress and identifying risks
5. Communicating status updates to stakeholders

You have access to calculation, file operations, and research tools. Always:
- Create detailed project plans with clear deliverables
- Identify dependencies and critical path items
- Consider resource constraints and availability
- Plan for risks and contingencies
- Maintain clear communication with all stakeholders"#.to_string(),
            tools: vec![
                "calculator".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.4,
            max_tokens: 2000,
            memory_config: MemoryConfig {
                max_short_term_entries: 120,
                max_long_term_entries: 2500,
                retention_period: Duration::from_secs(86400 * 21), // 3 weeks
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: true,
                max_concurrent_collaborations: 10, // PMs coordinate many people
                consensus_enabled: true,
                ..Default::default()
            },
        }
    }

    /// Quality Assurance Agent
    pub fn quality_assurance() -> RoleTemplate {
        RoleTemplate {
            name: "Quality Assurance".to_string(),
            description: "QA specialist focused on testing, validation, and ensuring high quality standards".to_string(),
            system_prompt: r#"You are a meticulous quality assurance specialist with expertise in testing, validation, and quality control. Your responsibilities include:

1. Reviewing deliverables for quality and compliance
2. Creating and executing test plans and test cases
3. Identifying bugs, issues, and areas for improvement
4. Ensuring adherence to standards and best practices
5. Providing detailed feedback and recommendations

You have access to file operations, research, and analysis tools. Always:
- Be thorough and systematic in your reviews
- Document all findings clearly and objectively
- Provide specific, actionable feedback
- Consider edge cases and potential failure scenarios
- Verify that requirements are fully met"#.to_string(),
            tools: vec![
                "file_read".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
                "calculator".to_string(),
            ],
            model: "mock-gpt-4".to_string(),
            temperature: 0.2, // Very low temperature for consistency
            max_tokens: 1500,
            memory_config: MemoryConfig {
                max_short_term_entries: 100,
                max_long_term_entries: 2000,
                retention_period: Duration::from_secs(86400 * 14), // 2 weeks
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: false, // QA typically reviews independently
                ..Default::default()
            },
        }
    }

    /// Customer Support Agent
    pub fn customer_support() -> RoleTemplate {
        RoleTemplate {
            name: "Customer Support".to_string(),
            description: "Customer service specialist focused on helping users and resolving issues".to_string(),
            system_prompt: r#"You are a friendly and knowledgeable customer support specialist dedicated to helping users and resolving their issues. Your responsibilities include:

1. Understanding customer problems and providing solutions
2. Explaining complex concepts in simple, accessible terms
3. Escalating issues when necessary
4. Following up to ensure customer satisfaction
5. Maintaining a positive and professional demeanor

You have access to research, file operations, and knowledge base tools. Always:
- Listen carefully to understand the customer's needs
- Respond with empathy and professionalism
- Provide clear, step-by-step solutions
- Follow up to ensure the issue is resolved
- Document interactions for future reference"#.to_string(),
            tools: vec![
                "text_search".to_string(),
                "http_get".to_string(),
                "file_read".to_string(),
            ],
            model: "gpt-3.5-turbo".to_string(), // Faster model for real-time support
            temperature: 0.6,
            max_tokens: 1000,
            memory_config: MemoryConfig {
                max_short_term_entries: 200, // High volume of interactions
                max_long_term_entries: 1000,
                retention_period: Duration::from_secs(86400 * 7), // 1 week
                ..Default::default()
            },
            collaboration_config: CollaborationConfig {
                delegation_enabled: true, // May need to escalate to specialists
                ..Default::default()
            },
        }
    }

    /// Get all available role templates
    pub fn all_templates() -> Vec<RoleTemplate> {
        vec![
            Self::software_developer(),
            Self::research_analyst(),
            Self::data_scientist(),
            Self::content_writer(),
            Self::project_manager(),
            Self::quality_assurance(),
            Self::customer_support(),
        ]
    }

    /// Get role template by name
    pub fn get_template(name: &str) -> Option<RoleTemplate> {
        match name.to_lowercase().as_str() {
            "software_developer" | "developer" => Some(Self::software_developer()),
            "research_analyst" | "researcher" => Some(Self::research_analyst()),
            "data_scientist" | "analyst" => Some(Self::data_scientist()),
            "content_writer" | "writer" => Some(Self::content_writer()),
            "project_manager" | "manager" => Some(Self::project_manager()),
            "quality_assurance" | "qa" => Some(Self::quality_assurance()),
            "customer_support" | "support" => Some(Self::customer_support()),
            _ => None,
        }
    }

    /// Get template names
    pub fn template_names() -> Vec<String> {
        vec![
            "software_developer".to_string(),
            "research_analyst".to_string(),
            "data_scientist".to_string(),
            "content_writer".to_string(),
            "project_manager".to_string(),
            "quality_assurance".to_string(),
            "customer_support".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_template_creation() {
        let template = RoleTemplates::software_developer();
        assert_eq!(template.name, "Software Developer");
        assert!(template.tools.contains(&"file_read".to_string()));
        assert!(template.tools.contains(&"file_write".to_string()));
        assert_eq!(template.temperature, 0.3);
    }

    #[test]
    fn test_template_to_agent_config() {
        let template = RoleTemplates::research_analyst();
        let config = template.to_agent_config("MyResearcher".to_string(), "openai".to_string());
        
        assert_eq!(config.name, "MyResearcher");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, template.model);
        assert_eq!(config.available_tools, template.tools);
    }

    #[test]
    fn test_get_template_by_name() {
        let template = RoleTemplates::get_template("developer");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Software Developer");
        
        let invalid_template = RoleTemplates::get_template("invalid_role");
        assert!(invalid_template.is_none());
    }

    #[test]
    fn test_all_templates() {
        let templates = RoleTemplates::all_templates();
        assert_eq!(templates.len(), 7);
        
        let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"Software Developer".to_string()));
        assert!(names.contains(&"Research Analyst".to_string()));
        assert!(names.contains(&"Data Scientist".to_string()));
    }

    #[test]
    fn test_template_names() {
        let names = RoleTemplates::template_names();
        assert_eq!(names.len(), 7);
        assert!(names.contains(&"software_developer".to_string()));
        assert!(names.contains(&"customer_support".to_string()));
    }

    #[test]
    fn test_role_specific_configurations() {
        let dev_template = RoleTemplates::software_developer();
        assert_eq!(dev_template.temperature, 0.3); // Low for consistency
        
        let writer_template = RoleTemplates::content_writer();
        assert_eq!(writer_template.temperature, 0.8); // High for creativity
        
        let qa_template = RoleTemplates::quality_assurance();
        assert_eq!(qa_template.temperature, 0.2); // Very low for consistency
    }
}
