# AgentGraph Deployment Guide

This guide covers deploying AgentGraph in production environments with enterprise-grade security, monitoring, and scalability.

## Overview

AgentGraph supports multiple deployment patterns:
- **Single Node**: Development and small-scale production
- **Multi-Node**: High availability and load distribution
- **Containerized**: Docker and Kubernetes deployments
- **Cloud Native**: AWS, GCP, Azure optimized deployments
- **Hybrid**: On-premises with cloud integration

## Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 4 cores, 2.4GHz
- **Memory**: 8GB RAM
- **Storage**: 50GB SSD
- **Network**: 1Gbps connection
- **OS**: Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)

#### Recommended Requirements
- **CPU**: 16 cores, 3.0GHz
- **Memory**: 32GB RAM
- **Storage**: 500GB NVMe SSD
- **Network**: 10Gbps connection
- **OS**: Linux with container runtime

#### Enterprise Requirements
- **CPU**: 32+ cores, 3.5GHz
- **Memory**: 128GB+ RAM
- **Storage**: 2TB+ NVMe SSD with backup
- **Network**: 25Gbps+ with redundancy
- **OS**: Enterprise Linux with security hardening

### Software Dependencies

```bash
# Required
- Rust 1.70+
- OpenSSL 1.1.1+
- PostgreSQL 13+ (for persistence)
- Redis 6+ (for caching)

# Optional
- Docker 20.10+
- Kubernetes 1.24+
- Prometheus (monitoring)
- Grafana (dashboards)
- Nginx (load balancing)
```

## Installation Methods

### 1. Binary Installation

```bash
# Download latest release
wget https://github.com/agent-graph/releases/latest/agent-graph-linux-x64.tar.gz

# Extract and install
tar -xzf agent-graph-linux-x64.tar.gz
sudo mv agent-graph /usr/local/bin/
sudo chmod +x /usr/local/bin/agent-graph

# Verify installation
agent-graph --version
```

### 2. Source Installation

```bash
# Clone repository
git clone https://github.com/agent-graph/agent-graph.git
cd agent-graph

# Build optimized release
cargo build --release --features production

# Install binary
sudo cp target/release/agent-graph /usr/local/bin/
```

### 3. Docker Installation

```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features production

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/agent-graph /usr/local/bin/
EXPOSE 8080
CMD ["agent-graph", "serve"]
```

```bash
# Build and run
docker build -t agent-graph:latest .
docker run -d -p 8080:8080 agent-graph:latest
```

## Configuration

### Environment Variables

```bash
# Core Configuration
export AGENT_GRAPH_ENV=production
export AGENT_GRAPH_LOG_LEVEL=info
export AGENT_GRAPH_PORT=8080
export AGENT_GRAPH_HOST=0.0.0.0

# Database Configuration
export DATABASE_URL=postgresql://user:pass@localhost:5432/agentgraph
export REDIS_URL=redis://localhost:6379

# LLM Provider Configuration
export OPENAI_API_KEY=your_openai_key
export ANTHROPIC_API_KEY=your_anthropic_key
export GOOGLE_API_KEY=your_google_key
export OPENROUTER_API_KEY=your_openrouter_key

# Security Configuration
export JWT_SECRET=your_jwt_secret_key
export ENCRYPTION_KEY=your_encryption_key
export CORS_ORIGINS=https://yourdomain.com

# Monitoring Configuration
export PROMETHEUS_ENDPOINT=http://prometheus:9090
export JAEGER_ENDPOINT=http://jaeger:14268
export LOG_ENDPOINT=http://elasticsearch:9200

# Resource Limits
export MAX_AGENTS_PER_NODE=100
export MAX_MEMORY_PER_AGENT=512MB
export MAX_CPU_PER_AGENT=1.0
export REQUEST_TIMEOUT=30s
```

### Configuration File

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 16
max_connections = 1000

[database]
url = "postgresql://user:pass@localhost:5432/agentgraph"
pool_size = 20
timeout = 30

[redis]
url = "redis://localhost:6379"
pool_size = 10
timeout = 5

[llm]
default_provider = "openai"
timeout = 30
max_retries = 3
retry_delay = 1000

[security]
jwt_expiry = 3600
session_timeout = 7200
max_login_attempts = 5
password_policy = "strong"

[monitoring]
metrics_enabled = true
tracing_enabled = true
health_check_interval = 30
log_level = "info"

[resources]
max_agents_per_node = 100
max_memory_per_agent = "512MB"
max_cpu_per_agent = 1.0
cleanup_interval = 3600
```

## Deployment Patterns

### 1. Single Node Deployment

```bash
# Simple single-node setup
agent-graph serve --config config/production.toml
```

```yaml
# systemd service
[Unit]
Description=AgentGraph Service
After=network.target

[Service]
Type=simple
User=agentgraph
WorkingDirectory=/opt/agentgraph
ExecStart=/usr/local/bin/agent-graph serve --config /etc/agentgraph/production.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 2. Multi-Node Deployment

```yaml
# docker-compose.yml
version: '3.8'
services:
  agentgraph-node1:
    image: agentgraph:latest
    environment:
      - NODE_ID=node1
      - CLUSTER_PEERS=node2:8080,node3:8080
    ports:
      - "8080:8080"
    
  agentgraph-node2:
    image: agentgraph:latest
    environment:
      - NODE_ID=node2
      - CLUSTER_PEERS=node1:8080,node3:8080
    ports:
      - "8081:8080"
    
  agentgraph-node3:
    image: agentgraph:latest
    environment:
      - NODE_ID=node3
      - CLUSTER_PEERS=node1:8080,node2:8080
    ports:
      - "8082:8080"
    
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
```

### 3. Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentgraph
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentgraph
  template:
    metadata:
      labels:
        app: agentgraph
    spec:
      containers:
      - name: agentgraph
        image: agentgraph:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: agentgraph-secrets
              key: database-url
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: agentgraph-service
spec:
  selector:
    app: agentgraph
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

## Security Configuration

### 1. TLS/SSL Setup

```bash
# Generate certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Configure TLS
export TLS_CERT_PATH=/etc/ssl/certs/agentgraph.crt
export TLS_KEY_PATH=/etc/ssl/private/agentgraph.key
```

### 2. Authentication Setup

```toml
[auth]
providers = ["jwt", "oauth2", "ldap"]
jwt_secret = "your-secret-key"
jwt_expiry = 3600

[oauth2]
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://yourdomain.com/auth/callback"

[ldap]
server = "ldap://ldap.company.com"
bind_dn = "cn=admin,dc=company,dc=com"
bind_password = "admin-password"
user_base = "ou=users,dc=company,dc=com"
```

### 3. Network Security

```bash
# Firewall rules
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 8080/tcp  # AgentGraph
sudo ufw enable

# Fail2ban configuration
sudo apt install fail2ban
sudo systemctl enable fail2ban
```

## Monitoring and Observability

### 1. Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'agentgraph'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

### 2. Grafana Dashboards

```json
{
  "dashboard": {
    "title": "AgentGraph Metrics",
    "panels": [
      {
        "title": "Agent Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(agentgraph_requests_total[5m])",
            "legendFormat": "Requests/sec"
          }
        ]
      }
    ]
  }
}
```

### 3. Logging Configuration

```toml
[logging]
level = "info"
format = "json"
output = "stdout"

[logging.structured]
service = "agentgraph"
version = "0.7.0"
environment = "production"

[logging.exporters]
elasticsearch = "http://elasticsearch:9200"
fluentd = "http://fluentd:24224"
```

## Performance Tuning

### 1. System Optimization

```bash
# Kernel parameters
echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf
echo 'fs.file-max = 1000000' >> /etc/sysctl.conf

# Apply changes
sysctl -p
```

### 2. Application Tuning

```toml
[performance]
worker_threads = 16
max_blocking_threads = 512
thread_stack_size = "2MB"
thread_keep_alive = "60s"

[cache]
memory_limit = "4GB"
ttl_default = "1h"
cleanup_interval = "10m"

[database]
connection_pool_size = 50
statement_cache_size = 1000
query_timeout = "30s"
```

## Backup and Recovery

### 1. Database Backup

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump agentgraph > /backups/agentgraph_$DATE.sql
aws s3 cp /backups/agentgraph_$DATE.sql s3://backups/agentgraph/
```

### 2. State Backup

```bash
# Backup agent states and memory
agent-graph backup --output /backups/state_$DATE.tar.gz
```

### 3. Recovery Procedures

```bash
# Database recovery
psql agentgraph < /backups/agentgraph_latest.sql

# State recovery
agent-graph restore --input /backups/state_latest.tar.gz
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   ```bash
   # Check memory usage
   agent-graph stats --memory
   
   # Adjust limits
   export MAX_MEMORY_PER_AGENT=256MB
   ```

2. **Connection Timeouts**
   ```bash
   # Increase timeouts
   export REQUEST_TIMEOUT=60s
   export DATABASE_TIMEOUT=30s
   ```

3. **Performance Issues**
   ```bash
   # Enable profiling
   export RUST_LOG=debug
   agent-graph serve --profile
   ```

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed status
curl http://localhost:8080/status

# Metrics endpoint
curl http://localhost:8080/metrics
```

## Scaling Strategies

### Horizontal Scaling
- Add more nodes to the cluster
- Use load balancers for distribution
- Implement auto-scaling based on metrics

### Vertical Scaling
- Increase CPU and memory resources
- Optimize database connections
- Tune garbage collection settings

### Database Scaling
- Read replicas for query distribution
- Connection pooling optimization
- Query optimization and indexing

This deployment guide provides comprehensive instructions for production-ready AgentGraph deployments. Regular monitoring and maintenance are essential for optimal performance.
