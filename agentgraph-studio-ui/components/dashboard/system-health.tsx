'use client'

import { motion } from 'framer-motion'
import { 
  Cpu, 
  Database, 
  Network, 
  HardDrive, 
  Activity, 
  Zap,
  Server,
  Gauge,
  TrendingUp,
  AlertTriangle,
  CheckCircle2,
  Clock
} from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { useAgentGraph } from '@/hooks/use-agentgraph'
import { formatBytes } from '@/lib/utils'

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
}

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.5,
      ease: 'easeOut',
    },
  },
}

export function SystemHealth() {
  const { metrics } = useAgentGraph()

  // Mock system health data (in real implementation, this would come from the backend)
  const systemHealth = {
    cpu_usage: 45,
    memory_usage: 68,
    disk_usage: 32,
    network_latency: 12,
    uptime: '2d 14h 32m',
    active_connections: 156,
    requests_per_second: 1247,
    error_rate: 0.02,
    response_time: 89,
    throughput: 2.4
  }

  const healthStatus = systemHealth.cpu_usage < 70 && systemHealth.memory_usage < 80 && systemHealth.error_rate < 0.05 ? 'healthy' : 'warning'

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* System Overview */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center space-x-2">
                  <Server className="h-5 w-5 text-blue-500" />
                  <span>System Health Overview</span>
                </CardTitle>
                <CardDescription>Real-time system performance and health metrics</CardDescription>
              </div>
              <Badge 
                variant={healthStatus === 'healthy' ? 'success' : 'warning'}
                className="text-sm px-3 py-1"
              >
                {healthStatus === 'healthy' ? (
                  <CheckCircle2 className="w-4 h-4 mr-1" />
                ) : (
                  <AlertTriangle className="w-4 h-4 mr-1" />
                )}
                {healthStatus === 'healthy' ? 'Healthy' : 'Warning'}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-muted-foreground">Uptime</span>
                  <Clock className="w-4 h-4 text-green-500" />
                </div>
                <p className="text-2xl font-bold text-green-600">{systemHealth.uptime}</p>
              </div>
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-muted-foreground">Active Connections</span>
                  <Network className="w-4 h-4 text-blue-500" />
                </div>
                <p className="text-2xl font-bold text-blue-600">{systemHealth.active_connections}</p>
              </div>
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-muted-foreground">Requests/sec</span>
                  <Zap className="w-4 h-4 text-orange-500" />
                </div>
                <p className="text-2xl font-bold text-orange-600">{systemHealth.requests_per_second.toLocaleString()}</p>
              </div>
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-muted-foreground">Error Rate</span>
                  <Activity className="w-4 h-4 text-purple-500" />
                </div>
                <p className="text-2xl font-bold text-purple-600">{(systemHealth.error_rate * 100).toFixed(2)}%</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Resource Usage */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card className="glass-card border-0 hover:scale-105 transition-all duration-300">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center space-x-2 text-sm">
              <Cpu className="h-4 w-4 text-blue-500" />
              <span>CPU Usage</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemHealth.cpu_usage}%</span>
                <Badge variant={systemHealth.cpu_usage < 70 ? 'success' : 'warning'}>
                  {systemHealth.cpu_usage < 70 ? 'Normal' : 'High'}
                </Badge>
              </div>
              <Progress value={systemHealth.cpu_usage} className="h-2" />
              <p className="text-xs text-muted-foreground">4 cores • 2.4 GHz</p>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0 hover:scale-105 transition-all duration-300">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center space-x-2 text-sm">
              <Database className="h-4 w-4 text-green-500" />
              <span>Memory Usage</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemHealth.memory_usage}%</span>
                <Badge variant={systemHealth.memory_usage < 80 ? 'success' : 'warning'}>
                  {systemHealth.memory_usage < 80 ? 'Normal' : 'High'}
                </Badge>
              </div>
              <Progress value={systemHealth.memory_usage} className="h-2" />
              <p className="text-xs text-muted-foreground">5.4 GB / 8 GB</p>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0 hover:scale-105 transition-all duration-300">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center space-x-2 text-sm">
              <HardDrive className="h-4 w-4 text-orange-500" />
              <span>Disk Usage</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemHealth.disk_usage}%</span>
                <Badge variant="success">Normal</Badge>
              </div>
              <Progress value={systemHealth.disk_usage} className="h-2" />
              <p className="text-xs text-muted-foreground">160 GB / 500 GB</p>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0 hover:scale-105 transition-all duration-300">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center space-x-2 text-sm">
              <Network className="h-4 w-4 text-purple-500" />
              <span>Network Latency</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-2xl font-bold">{systemHealth.network_latency}ms</span>
                <Badge variant="success">Excellent</Badge>
              </div>
              <Progress value={100 - systemHealth.network_latency} className="h-2" />
              <p className="text-xs text-muted-foreground">Avg response time</p>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Performance Metrics */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Gauge className="h-5 w-5 text-blue-500" />
              <span>Performance Metrics</span>
            </CardTitle>
            <CardDescription>Key performance indicators and benchmarks</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                <div>
                  <p className="font-medium">Average Response Time</p>
                  <p className="text-sm text-muted-foreground">HTTP requests</p>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold text-blue-600">{systemHealth.response_time}ms</p>
                  <div className="flex items-center space-x-1 text-green-600">
                    <TrendingUp className="w-3 h-3" />
                    <span className="text-xs">-12%</span>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                <div>
                  <p className="font-medium">Throughput</p>
                  <p className="text-sm text-muted-foreground">MB/s processed</p>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold text-green-600">{systemHealth.throughput} MB/s</p>
                  <div className="flex items-center space-x-1 text-green-600">
                    <TrendingUp className="w-3 h-3" />
                    <span className="text-xs">+8%</span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Activity className="h-5 w-5 text-green-500" />
              <span>System Status</span>
            </CardTitle>
            <CardDescription>Current system status and alerts</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center space-x-3 p-3 rounded-lg bg-green-50 dark:bg-green-950/20 border border-green-200 dark:border-green-800">
                <CheckCircle2 className="w-5 h-5 text-green-600" />
                <div>
                  <p className="font-medium text-green-800 dark:text-green-200">All Systems Operational</p>
                  <p className="text-sm text-green-600 dark:text-green-400">No critical issues detected</p>
                </div>
              </div>
              
              <div className="flex items-center space-x-3 p-3 rounded-lg bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800">
                <Activity className="w-5 h-5 text-blue-600" />
                <div>
                  <p className="font-medium text-blue-800 dark:text-blue-200">High Performance Mode</p>
                  <p className="text-sm text-blue-600 dark:text-blue-400">Optimized for maximum throughput</p>
                </div>
              </div>
              
              <div className="flex items-center space-x-3 p-3 rounded-lg bg-orange-50 dark:bg-orange-950/20 border border-orange-200 dark:border-orange-800">
                <Zap className="w-5 h-5 text-orange-600" />
                <div>
                  <p className="font-medium text-orange-800 dark:text-orange-200">Auto-scaling Active</p>
                  <p className="text-sm text-orange-600 dark:text-orange-400">Dynamically adjusting resources</p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  )
}
