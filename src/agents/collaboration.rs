// Agent collaboration system for AgentGraph
// Provides multi-agent communication and coordination patterns

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// Enable collaboration
    pub enabled: bool,
    /// Maximum concurrent collaborations
    pub max_concurrent_collaborations: u32,
    /// Collaboration timeout
    pub collaboration_timeout: Duration,
    /// Message queue size
    pub message_queue_size: usize,
    /// Enable delegation
    pub delegation_enabled: bool,
    /// Enable consensus mechanisms
    pub consensus_enabled: bool,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_collaborations: 5,
            collaboration_timeout: Duration::from_secs(300), // 5 minutes
            message_queue_size: 100,
            delegation_enabled: true,
            consensus_enabled: false,
        }
    }
}

/// Message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Request for assistance
    AssistanceRequest {
        task: String,
        context: String,
        urgency: MessageUrgency,
        required_capabilities: Vec<String>,
    },
    /// Response to assistance request
    AssistanceResponse {
        request_id: String,
        response: String,
        success: bool,
        metadata: HashMap<String, serde_json::Value>,
    },
    /// Task delegation
    TaskDelegation {
        task: String,
        deadline: Option<SystemTime>,
        requirements: Vec<String>,
    },
    /// Status update
    StatusUpdate {
        status: String,
        progress: f32, // 0.0 - 1.0
        estimated_completion: Option<SystemTime>,
    },
    /// Information sharing
    InformationShare {
        topic: String,
        information: String,
        relevance_score: f32,
    },
    /// Consensus request
    ConsensusRequest {
        proposal: String,
        options: Vec<String>,
        deadline: SystemTime,
    },
    /// Consensus vote
    ConsensusVote {
        request_id: String,
        vote: String,
        reasoning: String,
    },
}

/// Message urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessageUrgency {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Collaboration patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationPattern {
    /// Sequential execution (one after another)
    Sequential,
    /// Parallel execution (simultaneous)
    Parallel,
    /// Hierarchical (leader-follower)
    Hierarchical,
    /// Peer-to-peer (equal collaboration)
    PeerToPeer,
    /// Consensus-based decision making
    Consensus,
    /// Competitive (best solution wins)
    Competitive,
}

/// Collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// Session ID
    pub id: String,
    /// Participating agents
    pub participants: Vec<String>,
    /// Session pattern
    pub pattern: CollaborationPattern,
    /// Session goal
    pub goal: String,
    /// Session status
    pub status: CollaborationStatus,
    /// Start time
    pub started_at: SystemTime,
    /// End time
    pub ended_at: Option<SystemTime>,
    /// Session results
    pub results: HashMap<String, serde_json::Value>,
    /// Message history
    pub message_history: Vec<(String, AgentMessage, SystemTime)>,
}

impl CollaborationSession {
    /// Create a new collaboration session
    pub fn new(
        participants: Vec<String>,
        pattern: CollaborationPattern,
        goal: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            participants,
            pattern,
            goal,
            status: CollaborationStatus::Active,
            started_at: SystemTime::now(),
            ended_at: None,
            results: HashMap::new(),
            message_history: Vec::new(),
        }
    }
    
    /// Add message to session
    pub fn add_message(&mut self, sender: String, message: AgentMessage) {
        self.message_history.push((sender, message, SystemTime::now()));
    }
    
    /// End the session
    pub fn end_session(&mut self, status: CollaborationStatus) {
        self.status = status;
        self.ended_at = Some(SystemTime::now());
    }
    
    /// Get session duration
    pub fn duration(&self) -> Duration {
        let end_time = self.ended_at.unwrap_or_else(SystemTime::now);
        end_time.duration_since(self.started_at).unwrap_or(Duration::ZERO)
    }
}

/// Collaboration session status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationStatus {
    /// Session is active
    Active,
    /// Session completed successfully
    Completed,
    /// Session failed
    Failed,
    /// Session was cancelled
    Cancelled,
    /// Session timed out
    TimedOut,
}

/// Agent collaboration manager
#[derive(Debug)]
pub struct CollaborationManager {
    /// Configuration
    config: CollaborationConfig,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, CollaborationSession>>>,
    /// Message channels for agents
    message_channels: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    /// Agent capabilities registry
    agent_capabilities: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl CollaborationManager {
    /// Create a new collaboration manager
    pub fn new(config: CollaborationConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_channels: Arc::new(RwLock::new(HashMap::new())),
            agent_capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an agent for collaboration
    pub async fn register_agent(
        &self,
        agent_id: String,
        capabilities: Vec<String>,
    ) -> Result<mpsc::UnboundedReceiver<AgentMessage>, CollaborationError> {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        {
            let mut channels = self.message_channels.write().await;
            channels.insert(agent_id.clone(), sender);
        }
        
        {
            let mut caps = self.agent_capabilities.write().await;
            caps.insert(agent_id, capabilities);
        }
        
        Ok(receiver)
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<(), CollaborationError> {
        {
            let mut channels = self.message_channels.write().await;
            channels.remove(agent_id);
        }
        
        {
            let mut caps = self.agent_capabilities.write().await;
            caps.remove(agent_id);
        }
        
        Ok(())
    }
    
    /// Start a collaboration session
    pub async fn start_collaboration(
        &self,
        participants: Vec<String>,
        pattern: CollaborationPattern,
        goal: String,
    ) -> Result<String, CollaborationError> {
        if !self.config.enabled {
            return Err(CollaborationError::CollaborationDisabled);
        }
        
        // Check if all participants are registered
        let channels = self.message_channels.read().await;
        for participant in &participants {
            if !channels.contains_key(participant) {
                return Err(CollaborationError::AgentNotRegistered {
                    agent_id: participant.clone(),
                });
            }
        }
        
        // Check concurrent collaboration limit
        let sessions = self.sessions.read().await;
        let active_sessions = sessions.values()
            .filter(|s| s.status == CollaborationStatus::Active)
            .count();
        
        if active_sessions >= self.config.max_concurrent_collaborations as usize {
            return Err(CollaborationError::TooManyConcurrentCollaborations {
                current: active_sessions as u32,
                limit: self.config.max_concurrent_collaborations,
            });
        }
        drop(sessions);
        
        // Create new session
        let session = CollaborationSession::new(participants, pattern, goal);
        let session_id = session.id.clone();
        
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }
        
        Ok(session_id)
    }
    
    /// Send message to agent
    pub async fn send_message(
        &self,
        recipient: &str,
        message: AgentMessage,
    ) -> Result<(), CollaborationError> {
        let channels = self.message_channels.read().await;
        
        if let Some(sender) = channels.get(recipient) {
            sender.send(message)
                .map_err(|_| CollaborationError::MessageDeliveryFailed {
                    recipient: recipient.to_string(),
                })?;
            Ok(())
        } else {
            Err(CollaborationError::AgentNotRegistered {
                agent_id: recipient.to_string(),
            })
        }
    }
    
    /// Broadcast message to multiple agents
    pub async fn broadcast_message(
        &self,
        recipients: &[String],
        message: AgentMessage,
    ) -> Result<(), CollaborationError> {
        for recipient in recipients {
            self.send_message(recipient, message.clone()).await?;
        }
        Ok(())
    }
    
    /// Find agents with specific capabilities
    pub async fn find_agents_with_capabilities(
        &self,
        required_capabilities: &[String],
    ) -> Result<Vec<String>, CollaborationError> {
        let capabilities = self.agent_capabilities.read().await;
        
        let matching_agents = capabilities.iter()
            .filter(|(_, agent_caps)| {
                required_capabilities.iter()
                    .all(|req_cap| agent_caps.contains(req_cap))
            })
            .map(|(agent_id, _)| agent_id.clone())
            .collect();
        
        Ok(matching_agents)
    }
    
    /// Request assistance from other agents
    pub async fn request_assistance(
        &self,
        requester: &str,
        task: String,
        context: String,
        urgency: MessageUrgency,
        required_capabilities: Vec<String>,
    ) -> Result<Vec<String>, CollaborationError> {
        // Find suitable agents
        let suitable_agents = self.find_agents_with_capabilities(&required_capabilities).await?;
        
        // Filter out the requester
        let candidates: Vec<String> = suitable_agents.into_iter()
            .filter(|agent_id| agent_id != requester)
            .collect();
        
        if candidates.is_empty() {
            return Err(CollaborationError::NoSuitableAgents {
                capabilities: required_capabilities,
            });
        }
        
        // Send assistance request
        let message = AgentMessage::AssistanceRequest {
            task,
            context,
            urgency,
            required_capabilities,
        };
        
        self.broadcast_message(&candidates, message).await?;
        
        Ok(candidates)
    }
    
    /// Delegate task to another agent
    pub async fn delegate_task(
        &self,
        _delegator: &str,
        delegate: &str,
        task: String,
        deadline: Option<SystemTime>,
        requirements: Vec<String>,
    ) -> Result<(), CollaborationError> {
        if !self.config.delegation_enabled {
            return Err(CollaborationError::DelegationDisabled);
        }
        
        let message = AgentMessage::TaskDelegation {
            task,
            deadline,
            requirements,
        };
        
        self.send_message(delegate, message).await
    }
    
    /// End a collaboration session
    pub async fn end_collaboration(
        &self,
        session_id: &str,
        status: CollaborationStatus,
    ) -> Result<(), CollaborationError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.end_session(status);
            Ok(())
        } else {
            Err(CollaborationError::SessionNotFound {
                session_id: session_id.to_string(),
            })
        }
    }
    
    /// Get collaboration session
    pub async fn get_session(&self, session_id: &str) -> Result<CollaborationSession, CollaborationError> {
        let sessions = self.sessions.read().await;
        
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| CollaborationError::SessionNotFound {
                session_id: session_id.to_string(),
            })
    }
    
    /// Get active sessions
    pub async fn get_active_sessions(&self) -> Result<Vec<CollaborationSession>, CollaborationError> {
        let sessions = self.sessions.read().await;
        
        let active_sessions = sessions.values()
            .filter(|s| s.status == CollaborationStatus::Active)
            .cloned()
            .collect();
        
        Ok(active_sessions)
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u32, CollaborationError> {
        let mut sessions = self.sessions.write().await;
        let now = SystemTime::now();
        let mut cleaned_count = 0;
        
        sessions.retain(|_, session| {
            if session.status == CollaborationStatus::Active {
                let duration = now.duration_since(session.started_at).unwrap_or(Duration::ZERO);
                if duration > self.config.collaboration_timeout {
                    cleaned_count += 1;
                    false // Remove expired session
                } else {
                    true // Keep active session
                }
            } else {
                true // Keep completed sessions for history
            }
        });
        
        Ok(cleaned_count)
    }
    
    /// Get collaboration statistics
    pub async fn get_stats(&self) -> CollaborationStats {
        let sessions = self.sessions.read().await;
        let channels = self.message_channels.read().await;
        
        let total_sessions = sessions.len();
        let active_sessions = sessions.values()
            .filter(|s| s.status == CollaborationStatus::Active)
            .count();
        let completed_sessions = sessions.values()
            .filter(|s| s.status == CollaborationStatus::Completed)
            .count();
        let failed_sessions = sessions.values()
            .filter(|s| s.status == CollaborationStatus::Failed)
            .count();
        
        CollaborationStats {
            registered_agents: channels.len(),
            total_sessions,
            active_sessions,
            completed_sessions,
            failed_sessions,
            average_session_duration: self.calculate_average_duration(&sessions),
        }
    }
    
    /// Calculate average session duration
    fn calculate_average_duration(&self, sessions: &HashMap<String, CollaborationSession>) -> Duration {
        let completed_sessions: Vec<&CollaborationSession> = sessions.values()
            .filter(|s| s.ended_at.is_some())
            .collect();
        
        if completed_sessions.is_empty() {
            return Duration::ZERO;
        }
        
        let total_duration: Duration = completed_sessions.iter()
            .map(|s| s.duration())
            .sum();
        
        total_duration / completed_sessions.len() as u32
    }
    
    /// Get configuration
    pub fn config(&self) -> &CollaborationConfig {
        &self.config
    }
}

/// Collaboration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationStats {
    /// Number of registered agents
    pub registered_agents: usize,
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Number of completed sessions
    pub completed_sessions: usize,
    /// Number of failed sessions
    pub failed_sessions: usize,
    /// Average session duration
    pub average_session_duration: Duration,
}

/// Errors that can occur in collaboration operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum CollaborationError {
    /// Collaboration is disabled
    #[error("Collaboration is disabled")]
    CollaborationDisabled,
    
    /// Delegation is disabled
    #[error("Task delegation is disabled")]
    DelegationDisabled,
    
    /// Agent not registered
    #[error("Agent not registered: {agent_id}")]
    AgentNotRegistered { agent_id: String },
    
    /// Session not found
    #[error("Collaboration session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    /// Too many concurrent collaborations
    #[error("Too many concurrent collaborations: {current}/{limit}")]
    TooManyConcurrentCollaborations { current: u32, limit: u32 },
    
    /// Message delivery failed
    #[error("Failed to deliver message to agent: {recipient}")]
    MessageDeliveryFailed { recipient: String },
    
    /// No suitable agents found
    #[error("No agents found with required capabilities: {capabilities:?}")]
    NoSuitableAgents { capabilities: Vec<String> },
    
    /// Configuration error
    #[error("Collaboration configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("Collaboration system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_config_default() {
        let config = CollaborationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_concurrent_collaborations, 5);
        assert!(config.delegation_enabled);
    }

    #[test]
    fn test_collaboration_session_creation() {
        let participants = vec!["agent1".to_string(), "agent2".to_string()];
        let session = CollaborationSession::new(
            participants.clone(),
            CollaborationPattern::PeerToPeer,
            "Test collaboration".to_string(),
        );
        
        assert_eq!(session.participants, participants);
        assert_eq!(session.pattern, CollaborationPattern::PeerToPeer);
        assert_eq!(session.goal, "Test collaboration");
        assert_eq!(session.status, CollaborationStatus::Active);
    }

    #[tokio::test]
    async fn test_collaboration_manager_agent_registration() {
        let config = CollaborationConfig::default();
        let manager = CollaborationManager::new(config);
        
        let capabilities = vec!["coding".to_string(), "testing".to_string()];
        let _receiver = manager.register_agent("agent1".to_string(), capabilities.clone()).await.unwrap();
        
        // Check if agent can be found by capabilities
        let found_agents = manager.find_agents_with_capabilities(&["coding".to_string()]).await.unwrap();
        assert!(found_agents.contains(&"agent1".to_string()));
        
        // Unregister agent
        manager.unregister_agent("agent1").await.unwrap();
        
        let found_agents_after = manager.find_agents_with_capabilities(&["coding".to_string()]).await.unwrap();
        assert!(!found_agents_after.contains(&"agent1".to_string()));
    }

    #[test]
    fn test_message_urgency_ordering() {
        assert!(MessageUrgency::Critical > MessageUrgency::High);
        assert!(MessageUrgency::High > MessageUrgency::Normal);
        assert!(MessageUrgency::Normal > MessageUrgency::Low);
    }

    #[tokio::test]
    async fn test_collaboration_session_management() {
        let config = CollaborationConfig::default();
        let manager = CollaborationManager::new(config);
        
        // Register agents
        let _receiver1 = manager.register_agent("agent1".to_string(), vec!["coding".to_string()]).await.unwrap();
        let _receiver2 = manager.register_agent("agent2".to_string(), vec!["testing".to_string()]).await.unwrap();
        
        // Start collaboration
        let participants = vec!["agent1".to_string(), "agent2".to_string()];
        let session_id = manager.start_collaboration(
            participants,
            CollaborationPattern::Sequential,
            "Test project".to_string(),
        ).await.unwrap();
        
        // Get session
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.pattern, CollaborationPattern::Sequential);
        assert_eq!(session.goal, "Test project");
        
        // End collaboration
        manager.end_collaboration(&session_id, CollaborationStatus::Completed).await.unwrap();
        
        let ended_session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(ended_session.status, CollaborationStatus::Completed);
    }
}
