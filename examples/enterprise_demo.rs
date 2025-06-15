// Enterprise features demonstration example
// Shows multi-tenancy, security, resource management, audit logging, and monitoring

use agent_graph::enterprise::{
    EnterpriseManager, EnterpriseConfig, EnterpriseContext,
    tenancy::{Tenant, TenantConfig, TenantStatus},
    resources::{ResourceLimits, ResourceUsage},
    security::{Role, Permission, AuthContext},
    audit::{AuditEvent, AuditEventType, AuditLevel},
    monitoring::{PerformanceMetrics, HealthCheck, HealthStatus, Alert, AlertSeverity},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 AgentGraph Enterprise Features Demo");
    println!("=====================================");
    
    // Create enterprise configuration
    let config = EnterpriseConfig::default();
    let enterprise_manager = EnterpriseManager::new(config)?;
    
    // Demo 1: Multi-Tenancy
    println!("\n🏗️  Demo 1: Multi-Tenancy");
    println!("========================");
    
    // Create tenants
    let tenant1 = enterprise_manager.tenant_manager()
        .create_tenant("acme_corp".to_string(), "ACME Corporation".to_string())
        .await?;
    
    let tenant2 = enterprise_manager.tenant_manager()
        .create_tenant("beta_inc".to_string(), "Beta Inc".to_string())
        .await?;
    
    println!("✅ Created tenant: {} ({})", tenant1.name, tenant1.id);
    println!("✅ Created tenant: {} ({})", tenant2.name, tenant2.id);
    
    // Update tenant configuration
    let mut updated_tenant = tenant1.clone();
    let mut tenant_config = TenantConfig::default();
    tenant_config.max_concurrent_executions = 20;
    tenant_config.default_timeout = Duration::from_secs(600);
    updated_tenant.update_config(tenant_config);
    
    enterprise_manager.tenant_manager()
        .update_tenant(updated_tenant)
        .await?;
    
    println!("✅ Updated tenant configuration for {}", tenant1.name);
    
    // List all tenants
    let tenants = enterprise_manager.tenant_manager().list_tenants().await?;
    println!("📋 Total tenants: {}", tenants.len());
    for tenant in &tenants {
        println!("   - {} ({}) - Status: {:?}", tenant.name, tenant.id, tenant.status);
    }
    
    // Demo 2: Security & RBAC
    println!("\n🔐 Demo 2: Security & RBAC");
    println!("==========================");
    
    // Create roles
    let admin_role = Role::admin();
    let user_role = Role::user();
    let tenant_admin_role = Role::tenant_admin("acme_corp".to_string());
    
    println!("✅ Created roles:");
    println!("   - Admin: {} permissions", admin_role.permissions.len());
    println!("   - User: {} permissions", user_role.permissions.len());
    println!("   - Tenant Admin: {} permissions", tenant_admin_role.permissions.len());
    
    // Add users with roles
    enterprise_manager.security_manager()
        .add_user_roles("admin_user".to_string(), vec![admin_role.clone()])
        .await?;
    
    enterprise_manager.security_manager()
        .add_user_roles("regular_user".to_string(), vec![user_role.clone()])
        .await?;
    
    enterprise_manager.security_manager()
        .add_user_roles("tenant_admin".to_string(), vec![tenant_admin_role.clone()])
        .await?;
    
    // Add API keys
    enterprise_manager.security_manager()
        .add_api_key("admin_api_key_123".to_string(), "admin_user".to_string())
        .await?;
    
    enterprise_manager.security_manager()
        .add_api_key("user_api_key_456".to_string(), "regular_user".to_string())
        .await?;
    
    println!("✅ Added users and API keys");
    
    // Test authentication
    let auth_context = enterprise_manager.security_manager()
        .authenticate("admin_api_key_123")
        .await?;
    
    println!("✅ Authenticated user: {}", auth_context.user_id);
    println!("   Roles: {:?}", auth_context.roles.iter().map(|r| &r.name).collect::<Vec<_>>());
    
    // Test authorization
    let graph_execute_permission = Permission::graph_execute();
    match enterprise_manager.security_manager()
        .authorize(&auth_context, &graph_execute_permission)
        .await {
        Ok(()) => println!("✅ User authorized for graph execution"),
        Err(e) => println!("❌ Authorization failed: {}", e),
    }
    
    // Demo 3: Resource Management
    println!("\n📊 Demo 3: Resource Management");
    println!("==============================");
    
    // Set resource limits for tenants
    let basic_limits = ResourceLimits::basic();
    let premium_limits = ResourceLimits::premium();
    
    enterprise_manager.resource_manager()
        .set_tenant_limits("acme_corp".to_string(), premium_limits.clone())
        .await?;
    
    enterprise_manager.resource_manager()
        .set_tenant_limits("beta_inc".to_string(), basic_limits.clone())
        .await?;
    
    println!("✅ Set resource limits:");
    println!("   - ACME Corp: Premium ({}MB memory, {} executions)", 
             premium_limits.max_memory_bytes.unwrap_or(0) / 1024 / 1024,
             premium_limits.max_executions.unwrap_or(0));
    println!("   - Beta Inc: Basic ({}MB memory, {} executions)", 
             basic_limits.max_memory_bytes.unwrap_or(0) / 1024 / 1024,
             basic_limits.max_executions.unwrap_or(0));
    
    // Simulate resource usage
    let mut usage = ResourceUsage::new();
    usage.cpu_time_ms = 5000;
    usage.memory_bytes = 100 * 1024 * 1024; // 100 MB
    usage.execution_count = 5;
    usage.network_bytes_sent = 1024 * 1024; // 1 MB
    usage.network_bytes_received = 2 * 1024 * 1024; // 2 MB
    
    // Record usage for tenant
    enterprise_manager.resource_manager()
        .record_usage(Some("acme_corp"), &usage)
        .await?;
    
    println!("✅ Recorded resource usage for ACME Corp:");
    println!("   - CPU time: {}ms", usage.cpu_time_ms);
    println!("   - Memory: {}MB", usage.memory_bytes / 1024 / 1024);
    println!("   - Executions: {}", usage.execution_count);
    println!("   - Network: {}MB", usage.total_network_bytes() / 1024 / 1024);
    
    // Check quotas
    match enterprise_manager.resource_manager()
        .check_quotas(&usage, Some("acme_corp"))
        .await {
        Ok(()) => println!("✅ Resource usage within limits"),
        Err(e) => println!("❌ Resource quota exceeded: {}", e),
    }
    
    // Demo 4: Audit Logging
    println!("\n📝 Demo 4: Audit Logging");
    println!("========================");
    
    // Create audit events
    let login_event = AuditEvent::user_login("admin_user".to_string(), true)
        .with_tenant("acme_corp".to_string())
        .with_session("session_123".to_string())
        .with_network("192.168.1.100".to_string(), Some("AgentGraph-Client/1.0".to_string()));
    
    let graph_exec_event = AuditEvent::graph_execution(
        "admin_user".to_string(),
        "workflow_456".to_string(),
        true,
    )
    .with_tenant("acme_corp".to_string())
    .with_data("execution_time_ms".to_string(), 2500)
    .with_data("nodes_executed".to_string(), 5);
    
    let security_event = AuditEvent::security_event(
        "api_key_usage".to_string(),
        "API key used for authentication".to_string(),
    )
    .with_user("admin_user".to_string())
    .with_data("api_key_prefix".to_string(), "admin_api_***");
    
    // Log events
    enterprise_manager.audit_logger().log_event(login_event).await?;
    enterprise_manager.audit_logger().log_event(graph_exec_event).await?;
    enterprise_manager.audit_logger().log_event(security_event).await?;
    
    println!("✅ Logged audit events:");
    println!("   - User login event");
    println!("   - Graph execution event");
    println!("   - Security event");
    
    // Get audit statistics
    let audit_stats = enterprise_manager.audit_logger().get_stats();
    println!("📊 Audit statistics:");
    println!("   - Total events: {}", audit_stats.total_events);
    println!("   - Events by type: {:?}", audit_stats.events_by_type);
    println!("   - Events by level: {:?}", audit_stats.events_by_level);
    
    // Demo 5: Monitoring & Alerting
    println!("\n📈 Demo 5: Monitoring & Alerting");
    println!("=================================");
    
    // Collect metrics
    let metrics = enterprise_manager.metrics_collector().collect_metrics().await?;
    println!("✅ Collected system metrics:");
    println!("   - CPU usage: {:.1}%", metrics.cpu_usage_percent);
    println!("   - Memory usage: {:.1}%", metrics.memory_usage_percent());
    println!("   - Request rate: {:.1} req/s", metrics.request_rate);
    println!("   - Error rate: {:.1} err/s", metrics.error_rate);
    
    // Perform health checks
    let health_checks = enterprise_manager.metrics_collector().perform_health_checks().await?;
    println!("✅ Health check results:");
    for (component, check) in &health_checks {
        let status_emoji = match check.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️",
            HealthStatus::Unhealthy => "❌",
            HealthStatus::Unknown => "❓",
        };
        println!("   {} {}: {} ({}ms)", status_emoji, component, check.message, check.response_time_ms);
    }
    
    // Create and send alerts
    let high_cpu_alert = Alert::new(
        AlertSeverity::Warning,
        "High CPU Usage".to_string(),
        "CPU usage is above normal levels".to_string(),
        "system".to_string(),
    )
    .with_metric("cpu_usage_percent".to_string(), 85.0, 80.0);
    
    println!("🚨 Generated alert: {} (Severity: {:?})", high_cpu_alert.title, high_cpu_alert.severity);
    
    // Demo 6: Enterprise Context Integration
    println!("\n🔗 Demo 6: Enterprise Context");
    println!("=============================");
    
    // Create enterprise context
    let mut context = enterprise_manager.create_context(
        Some("acme_corp".to_string()),
        Some("admin_api_key_123".to_string()),
    ).await?;
    
    println!("✅ Created enterprise context:");
    println!("   - Tenant: {}", context.tenant_id().unwrap_or("None"));
    println!("   - User: {}", context.user_id().unwrap_or("None"));
    println!("   - Execution duration: {:?}", context.execution_duration());
    
    // Validate context
    enterprise_manager.validate_context(&context).await?;
    println!("✅ Context validation passed");
    
    // Record usage in context
    enterprise_manager.record_usage(&mut context, usage.clone()).await?;
    println!("✅ Recorded usage in context");
    
    // Add audit event to context
    let context_event = AuditEvent::new(
        AuditEventType::System,
        "context_demo".to_string(),
        "Enterprise context demonstration completed".to_string(),
    )
    .with_level(AuditLevel::Info)
    .with_user(context.user_id().unwrap_or("unknown").to_string())
    .with_tenant(context.tenant_id().unwrap_or("unknown").to_string());
    
    enterprise_manager.audit_event(&mut context, context_event).await?;
    println!("✅ Added audit event to context");
    
    println!("\n🎉 Enterprise Demo Complete!");
    println!("============================");
    println!("The AgentGraph Enterprise platform provides:");
    println!("✅ Multi-tenant isolation with configurable limits");
    println!("✅ Role-based access control with fine-grained permissions");
    println!("✅ Resource quotas and usage tracking per tenant");
    println!("✅ Comprehensive audit logging for compliance");
    println!("✅ Real-time monitoring and health checks");
    println!("✅ Integrated enterprise context for all operations");
    println!("✅ Production-ready security and scalability features");
    
    Ok(())
}
