'use client'

import { motion } from 'framer-motion'
import {
  TrendingUp,
  BarChart3,
  Activity,
  Clock,
  Zap,
  Target,
  Gauge,
  LineChart
} from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { useAgentGraph } from '@/hooks/use-agentgraph'

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

export function PerformanceCharts() {
  const { metrics } = useAgentGraph()

  // Mock performance data for demonstration
  const performanceData = {
    execution_trends: [
      { time: '00:00', executions: 45, success_rate: 94 },
      { time: '04:00', executions: 67, success_rate: 96 },
      { time: '08:00', executions: 123, success_rate: 92 },
      { time: '12:00', executions: 156, success_rate: 98 },
      { time: '16:00', executions: 134, success_rate: 95 },
      { time: '20:00', executions: 89, success_rate: 97 },
    ],
    response_times: {
      avg: 245,
      p50: 180,
      p95: 450,
      p99: 680
    },
    throughput: {
      current: 1247,
      peak: 1856,
      average: 1123
    },
    resource_usage: {
      cpu: 45,
      memory: 68,
      network: 23,
      disk: 12
    }
  }

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Performance Overview */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <BarChart3 className="h-5 w-5 text-blue-500" />
              <span>Performance Analytics</span>
            </CardTitle>
            <CardDescription>Real-time performance metrics and trend analysis</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                  <Zap className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-blue-600">{performanceData.throughput.current}</p>
                <p className="text-sm text-muted-foreground">Requests/sec</p>
                <Badge variant="success" className="text-xs">+12% vs avg</Badge>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-green-500 to-emerald-600 flex items-center justify-center">
                  <Clock className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-green-600">{performanceData.response_times.avg}ms</p>
                <p className="text-sm text-muted-foreground">Avg Response</p>
                <Badge variant="success" className="text-xs">-8% improvement</Badge>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-orange-500 to-red-600 flex items-center justify-center">
                  <Activity className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-orange-600">{metrics?.success_rate ? Math.round(metrics.success_rate) : 95}%</p>
                <p className="text-sm text-muted-foreground">Success Rate</p>
                <Badge variant="success" className="text-xs">+2.1% today</Badge>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                  <Target className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-purple-600">{performanceData.throughput.peak}</p>
                <p className="text-sm text-muted-foreground">Peak Throughput</p>
                <Badge variant="info" className="text-xs">All-time high</Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Response Time Analysis */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Gauge className="h-5 w-5 text-green-500" />
              <span>Response Time Distribution</span>
            </CardTitle>
            <CardDescription>Latency percentiles and performance benchmarks</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                <span className="font-medium">50th Percentile (Median)</span>
                <div className="flex items-center space-x-3">
                  <Progress value={75} className="w-20 h-2" />
                  <span className="text-sm font-bold text-green-600">{performanceData.response_times.p50}ms</span>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                <span className="font-medium">95th Percentile</span>
                <div className="flex items-center space-x-3">
                  <Progress value={60} className="w-20 h-2" />
                  <span className="text-sm font-bold text-orange-600">{performanceData.response_times.p95}ms</span>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                <span className="font-medium">99th Percentile</span>
                <div className="flex items-center space-x-3">
                  <Progress value={45} className="w-20 h-2" />
                  <span className="text-sm font-bold text-red-600">{performanceData.response_times.p99}ms</span>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800">
                <span className="font-medium text-blue-800 dark:text-blue-200">Average Response Time</span>
                <span className="text-lg font-bold text-blue-600">{performanceData.response_times.avg}ms</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Activity className="h-5 w-5 text-purple-500" />
              <span>Resource Utilization</span>
            </CardTitle>
            <CardDescription>System resource usage and optimization metrics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">CPU Usage</span>
                  <span className="text-sm font-bold">{performanceData.resource_usage.cpu}%</span>
                </div>
                <Progress value={performanceData.resource_usage.cpu} className="h-2" />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Memory Usage</span>
                  <span className="text-sm font-bold">{performanceData.resource_usage.memory}%</span>
                </div>
                <Progress value={performanceData.resource_usage.memory} className="h-2" />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Network I/O</span>
                  <span className="text-sm font-bold">{performanceData.resource_usage.network}%</span>
                </div>
                <Progress value={performanceData.resource_usage.network} className="h-2" />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Disk I/O</span>
                  <span className="text-sm font-bold">{performanceData.resource_usage.disk}%</span>
                </div>
                <Progress value={performanceData.resource_usage.disk} className="h-2" />
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Execution Trends */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <LineChart className="h-5 w-5 text-blue-500" />
              <span>Execution Trends (24h)</span>
            </CardTitle>
            <CardDescription>Workflow execution patterns and success rates over time</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="grid grid-cols-6 gap-4">
                {performanceData.execution_trends.map((data, index) => (
                  <div key={index} className="text-center space-y-2">
                    <div className="h-20 bg-gradient-to-t from-blue-500 to-purple-500 rounded-lg flex items-end justify-center relative overflow-hidden">
                      <div
                        className="w-full bg-gradient-to-t from-blue-600 to-blue-400 rounded-lg transition-all duration-500"
                        style={{ height: `${(data.executions / 200) * 100}%` }}
                      />
                      <span className="absolute bottom-1 text-xs text-white font-bold">{data.executions}</span>
                    </div>
                    <p className="text-xs font-medium">{data.time}</p>
                    <Badge variant="success" className="text-xs">{data.success_rate}%</Badge>
                  </div>
                ))}
              </div>

              <div className="flex items-center justify-center space-x-6 pt-4 border-t">
                <div className="flex items-center space-x-2">
                  <div className="w-3 h-3 rounded-full bg-gradient-to-r from-blue-500 to-purple-500" />
                  <span className="text-sm text-muted-foreground">Executions</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                  <span className="text-sm text-muted-foreground">Success Rate</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  )
}
