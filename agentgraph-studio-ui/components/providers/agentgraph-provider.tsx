'use client'

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react'
import { io, Socket } from 'socket.io-client'
import { 
  SystemMetrics, 
  VisualWorkflow, 
  ExecutionTrace, 
  VisualExecutionEvent,
  ConnectionStatus,
  WebSocketMessage
} from '@/lib/types'
import { createApiUrl, createWebSocketUrl } from '@/lib/utils'
// import { toast } from '@/hooks/use-toast'

interface AgentGraphContextType {
  // Data
  metrics: SystemMetrics | null
  workflows: VisualWorkflow[]
  traces: ExecutionTrace[]
  events: VisualExecutionEvent[]
  
  // Connection
  isConnected: boolean
  connectionStatus: string
  socket: Socket | null
  
  // Actions
  refreshData: () => Promise<void>
  connectWebSocket: () => void
  disconnectWebSocket: () => void
  
  // Loading states
  isLoading: boolean
  error: string | null
}

const AgentGraphContext = createContext<AgentGraphContextType | undefined>(undefined)

interface AgentGraphProviderProps {
  children: React.ReactNode
}

export function AgentGraphProvider({ children }: AgentGraphProviderProps) {
  // State
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [workflows, setWorkflows] = useState<VisualWorkflow[]>([])
  const [traces, setTraces] = useState<ExecutionTrace[]>([])
  const [events, setEvents] = useState<VisualExecutionEvent[]>([])
  
  // Connection state
  const [isConnected, setIsConnected] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState('Disconnected')
  const [socket, setSocket] = useState<Socket | null>(null)
  
  // Loading state
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // API functions with mock data fallback
  const fetchMetrics = useCallback(async (): Promise<SystemMetrics | null> => {
    try {
      const url = createApiUrl('/api/agentgraph/metrics')
      console.log('Fetching metrics from:', url)
      const response = await fetch(url)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      const data = await response.json()
      return data
    } catch (error) {
      console.error('Failed to fetch metrics:', error)
      setError('Failed to connect to backend')
      return null
    }
  }, [])

  const fetchWorkflows = useCallback(async (): Promise<VisualWorkflow[]> => {
    try {
      const response = await fetch(createApiUrl('/api/agentgraph/workflows'))
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      const data = await response.json()
      return Array.isArray(data) ? data : []
    } catch (error) {
      console.error('Failed to fetch workflows:', error)
      return []
    }
  }, [])

  const fetchTraces = useCallback(async (): Promise<ExecutionTrace[]> => {
    try {
      const response = await fetch(createApiUrl('/api/agentgraph/traces'))
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      const data = await response.json()
      return Array.isArray(data) ? data : []
    } catch (error) {
      console.error('Failed to fetch traces:', error)
      return []
    }
  }, [])

  // Refresh all data
  const refreshData = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    
    try {
      const [metricsData, workflowsData, tracesData] = await Promise.all([
        fetchMetrics(),
        fetchWorkflows(),
        fetchTraces(),
      ])
      
      if (metricsData) setMetrics(metricsData)
      setWorkflows(workflowsData)
      setTraces(tracesData)
      
      setConnectionStatus('Connected')
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      setError(errorMessage)
      setConnectionStatus(`Error: ${errorMessage}`)
      
      // toast({
      //   title: 'Connection Error',
      //   description: `Failed to fetch data: ${errorMessage}`,
      //   variant: 'destructive',
      // })
    } finally {
      setIsLoading(false)
    }
  }, [fetchMetrics, fetchWorkflows, fetchTraces])

  // WebSocket connection
  const connectWebSocket = useCallback(() => {
    if (socket?.connected) {
      return
    }

    try {
      const wsUrl = createWebSocketUrl('/api/agentgraph/events')
      const newSocket = io(wsUrl, {
        transports: ['websocket'],
        timeout: 5000,
        reconnection: true,
        reconnectionAttempts: 5,
        reconnectionDelay: 1000,
      })

      newSocket.on('connect', () => {
        console.log('WebSocket connected')
        setIsConnected(true)
        setConnectionStatus('Connected (Real-time)')
        
        // toast({
        //   title: 'Connected',
        //   description: 'Real-time updates are now active',
        // })
      })

      newSocket.on('disconnect', (reason) => {
        console.log('WebSocket disconnected:', reason)
        setIsConnected(false)
        setConnectionStatus(`Disconnected: ${reason}`)
      })

      newSocket.on('connect_error', (error) => {
        console.error('WebSocket connection error:', error)
        setIsConnected(false)
        setConnectionStatus(`Connection Error: ${error.message}`)
      })

      // Handle real-time events
      newSocket.on('event', (message: WebSocketMessage) => {
        if (message.type === 'event' && message.payload) {
          const event = message.payload as VisualExecutionEvent
          setEvents(prev => [event, ...prev.slice(0, 99)]) // Keep last 100 events
        }
      })

      // Handle metrics updates
      newSocket.on('metrics', (message: WebSocketMessage) => {
        if (message.type === 'metrics' && message.payload) {
          setMetrics(message.payload as SystemMetrics)
        }
      })

      // Handle workflow updates
      newSocket.on('workflow', (message: WebSocketMessage) => {
        if (message.type === 'workflow' && message.payload) {
          const workflow = message.payload as VisualWorkflow
          setWorkflows(prev => {
            const index = prev.findIndex(w => w.id === workflow.id)
            if (index >= 0) {
              const updated = [...prev]
              updated[index] = workflow
              return updated
            } else {
              return [workflow, ...prev]
            }
          })
        }
      })

      // Handle trace updates
      newSocket.on('trace', (message: WebSocketMessage) => {
        if (message.type === 'trace' && message.payload) {
          const trace = message.payload as ExecutionTrace
          setTraces(prev => {
            const index = prev.findIndex(t => t.id === trace.id)
            if (index >= 0) {
              const updated = [...prev]
              updated[index] = trace
              return updated
            } else {
              return [trace, ...prev.slice(0, 49)] // Keep last 50 traces
            }
          })
        }
      })

      setSocket(newSocket)
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error)
      setConnectionStatus('WebSocket Error')
    }
  }, [socket])

  const disconnectWebSocket = useCallback(() => {
    if (socket) {
      socket.disconnect()
      setSocket(null)
      setIsConnected(false)
      setConnectionStatus('Disconnected')
    }
  }, [socket])

  // Initialize connection on mount
  useEffect(() => {
    refreshData()
    connectWebSocket()

    // Cleanup on unmount
    return () => {
      disconnectWebSocket()
    }
  }, []) // Only run once on mount

  // Auto-refresh data every 30 seconds if not connected via WebSocket
  useEffect(() => {
    if (!isConnected) {
      const interval = setInterval(() => {
        refreshData()
      }, 30000)

      return () => clearInterval(interval)
    }
  }, [isConnected, refreshData])

  // Context value
  const value: AgentGraphContextType = {
    // Data
    metrics,
    workflows,
    traces,
    events,
    
    // Connection
    isConnected,
    connectionStatus,
    socket,
    
    // Actions
    refreshData,
    connectWebSocket,
    disconnectWebSocket,
    
    // Loading states
    isLoading,
    error,
  }

  return (
    <AgentGraphContext.Provider value={value}>
      {children}
    </AgentGraphContext.Provider>
  )
}

export function useAgentGraph() {
  const context = useContext(AgentGraphContext)
  if (context === undefined) {
    throw new Error('useAgentGraph must be used within an AgentGraphProvider')
  }
  return context
}
