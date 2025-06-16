// AgentGraph Studio Type Definitions

export interface SystemMetrics {
  total_executions: number
  active_executions: number
  completed_executions: number
  failed_executions: number
  avg_execution_time_ms: number
  success_rate: number
  node_metrics: Record<string, NodeMetrics>
  agent_metrics: Record<string, AgentMetrics>
  tool_metrics: Record<string, ToolMetrics>
  resource_metrics: ResourceMetrics
  last_updated: string
}

export interface NodeMetrics {
  node_id: string
  node_type: string
  total_executions: number
  successful_executions: number
  failed_executions: number
  avg_execution_time_ms: number
  min_execution_time_ms: number
  max_execution_time_ms: number
  success_rate: number
  last_execution?: string
}

export interface AgentMetrics {
  agent_name: string
  total_tasks: number
  total_tokens: number
  avg_tokens_per_task: number
  avg_response_time_ms: number
  success_rate: number
  total_cost: number
  last_activity?: string
}

export interface ToolMetrics {
  tool_name: string
  total_executions: number
  successful_executions: number
  failed_executions: number
  avg_execution_time_ms: number
  success_rate: number
  last_used?: string
}

export interface ResourceMetrics {
  cpu_usage: number
  memory_usage_mb: number
  memory_usage_percent: number
  active_threads: number
  network_rps: number
  disk_iops: number
}

export interface VisualWorkflow {
  id: string
  name: string
  nodes: VisualNode[]
  edges: VisualEdge[]
  current_execution?: string
  status: WorkflowStatus
  metadata: Record<string, any>
}

export interface VisualNode {
  id: string
  name: string
  node_type: string
  position: [number, number]
  status: NodeStatus
  execution_time_ms?: number
  metadata: Record<string, any>
  stats: NodeStats
}

export interface VisualEdge {
  id: string
  source: string
  target: string
  edge_type: string
  condition?: string
  execution_count: number
}

export interface NodeStats {
  total_executions: number
  successful_executions: number
  failed_executions: number
  avg_execution_time_ms: number
  last_execution?: string
}

export interface ExecutionTrace {
  id: string
  execution_id: string
  workflow_id: string
  start_time: string
  end_time?: string
  events: VisualExecutionEvent[]
  status: ExecutionStatus
  error?: string
}

export interface VisualExecutionEvent {
  id: string
  execution_id: string
  event_type: EventType
  node_id?: string
  timestamp: string
  data: Record<string, any>
  context: Record<string, any>
}

export type WorkflowStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export type NodeStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped'

export type ExecutionStatus = 'running' | 'completed' | 'failed' | 'cancelled'

export type EventType = 
  | 'execution_started'
  | 'node_started'
  | 'node_completed'
  | 'node_failed'
  | 'agent_response'
  | 'tool_execution'
  | 'command_routing'
  | 'state_update'
  | 'execution_completed'
  | 'execution_failed'
  | 'custom'

export type NodeType = 
  | 'agent'
  | 'tool'
  | 'routing'
  | 'quality_gate'
  | 'start'
  | 'end'
  | 'custom'

export type EdgeType = 
  | 'default'
  | 'conditional'
  | 'parallel'
  | 'tool_call'
  | 'custom'

// API Response Types
export interface ApiResponse<T> {
  data: T
  success: boolean
  message?: string
  timestamp: string
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  limit: number
  has_next: boolean
  has_prev: boolean
}

// WebSocket Message Types
export interface WebSocketMessage {
  type: 'event' | 'metrics' | 'workflow' | 'trace' | 'error'
  payload: any
  timestamp: string
}

// Dashboard Configuration
export interface DashboardConfig {
  refresh_interval: number
  auto_refresh: boolean
  theme: 'light' | 'dark' | 'system'
  layout: 'grid' | 'list'
  show_advanced_metrics: boolean
  notifications_enabled: boolean
}

// Chart Data Types
export interface ChartDataPoint {
  timestamp: string
  value: number
  label?: string
}

export interface TimeSeriesData {
  name: string
  data: ChartDataPoint[]
  color?: string
}

// Filter and Search Types
export interface FilterOptions {
  status?: NodeStatus[]
  node_type?: NodeType[]
  date_range?: {
    start: string
    end: string
  }
  search_query?: string
}

export interface SortOptions {
  field: string
  direction: 'asc' | 'desc'
}

// Component Props Types
export interface BaseComponentProps {
  className?: string
  children?: React.ReactNode
}

export interface MetricCardProps extends BaseComponentProps {
  title: string
  value: string | number
  change?: number
  icon?: React.ComponentType<any>
  trend?: 'up' | 'down' | 'neutral'
}

export interface StatusBadgeProps extends BaseComponentProps {
  status: NodeStatus | ExecutionStatus | WorkflowStatus
  size?: 'sm' | 'md' | 'lg'
}

// Error Types
export interface AgentGraphError {
  code: string
  message: string
  details?: Record<string, any>
  timestamp: string
}

// Connection Types
export interface ConnectionStatus {
  connected: boolean
  last_ping?: string
  latency_ms?: number
  reconnect_attempts: number
  error?: string
}

// Export utility type for component refs
export type ComponentRef<T> = React.RefObject<T>

// Export utility type for event handlers
export type EventHandler<T = any> = (event: T) => void

// Export utility type for async functions
export type AsyncFunction<T = any, R = any> = (args: T) => Promise<R>

// Layout Types
export interface LayoutPosition {
  x: number
  y: number
  width?: number
  height?: number
}

export interface LayoutConfig {
  algorithm: 'force' | 'hierarchical' | 'circular' | 'grid' | 'manual'
  spacing: number
  direction: 'horizontal' | 'vertical'
  animate: boolean
}

// Theme Types
export interface ThemeConfig {
  mode: 'light' | 'dark' | 'system'
  primary_color: string
  accent_color: string
  font_size: 'sm' | 'md' | 'lg'
  animations_enabled: boolean
}

// Performance Types
export interface PerformanceMetrics {
  render_time_ms: number
  memory_usage_mb: number
  fps: number
  bundle_size_kb: number
}

// Export all types as a namespace for easier imports
export namespace AgentGraphTypes {
  export type Metrics = SystemMetrics
  export type Workflow = VisualWorkflow
  export type Node = VisualNode
  export type Edge = VisualEdge
  export type Trace = ExecutionTrace
  export type Event = VisualExecutionEvent
  export type Error = AgentGraphError
  export type Connection = ConnectionStatus
}
