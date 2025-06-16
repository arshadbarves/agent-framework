'use client'

import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import {
  Activity,
  BarChart3,
  Clock,
  Cpu,
  Database,
  HardDrive,
  MemoryStick,
  Network,
  Server,
  TrendingUp,
  Users,
  Workflow,
  Zap,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Pause,
  Play,
  RotateCcw,
  Eye,
  Brain,
  Target,
  Gauge,
  Monitor,
  Settings
} from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { DashboardHeader } from '@/components/dashboard/dashboard-header'
import { MetricsOverview } from '@/components/dashboard/metrics-overview'
import { WorkflowVisualization } from '@/components/dashboard/workflow-visualization'
import { ExecutionTraces } from '@/components/dashboard/execution-traces'
import { PerformanceCharts } from '@/components/dashboard/performance-charts'
import { AgentMonitoring } from '@/components/dashboard/agent-monitoring'
import { RealTimeEvents } from '@/components/dashboard/real-time-events'
import { SystemHealth } from '@/components/dashboard/system-health'
import { AdvancedAnalytics } from '@/components/dashboard/advanced-analytics'
import { useAgentGraph } from '@/hooks/use-agentgraph'
import { formatNumber, formatDuration, getRelativeTime } from '@/lib/utils'

export default function ProfessionalDashboard() {
  const { metrics, workflows, traces, isConnected, isLoading, refreshData, error } = useAgentGraph()
  const [activeTab, setActiveTab] = useState('overview')

  // Apple-inspired animation variants
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.08,
        delayChildren: 0.1,
        ease: [0.25, 0.46, 0.45, 0.94] // Apple's signature easing
      }
    }
  }

  const itemVariants = {
    hidden: {
      opacity: 0,
      y: 24,
      scale: 0.95
    },
    visible: {
      opacity: 1,
      y: 0,
      scale: 1,
      transition: {
        duration: 0.6,
        ease: [0.25, 0.46, 0.45, 0.94]
      }
    }
  }

  if (error) {
    return (
      <div className="min-h-screen bg-white dark:bg-neutral-900 flex items-center justify-center">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/20 flex items-center justify-center">
              <XCircle className="w-8 h-8 text-red-500" />
            </div>
            <CardTitle className="text-red-600 dark:text-red-400">Connection Failed</CardTitle>
            <CardDescription>
              Unable to connect to AgentGraph backend. Please ensure the server is running on port 8081.
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center">
            <Button onClick={refreshData} disabled={isLoading}>
              <RotateCcw className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
              Retry Connection
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black">
      {/* Apple-style Header */}
      <DashboardHeader
        isConnected={isConnected}
        isLoading={isLoading}
        onRefresh={refreshData}
      />

      {/* Main Content - Apple spacing and layout */}
      <main className="max-w-7xl mx-auto px-6 py-8">
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          className="space-y-12"
        >
          {/* Apple-style Dashboard */}
          <motion.div variants={itemVariants}>
            <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-8">
              {/* Clean Header Section */}
              <div className="space-y-6">
                <div>
                  <h1 className="text-4xl font-semibold text-gray-900 dark:text-white tracking-tight">
                    Dashboard
                  </h1>
                  <p className="text-lg text-gray-600 dark:text-gray-400 mt-2 font-normal">
                    Monitor your workflows and system performance
                  </p>
                </div>

                {/* Apple-style Tab Navigation */}
                <TabsList className="inline-flex bg-gray-100 dark:bg-gray-800 p-1 rounded-xl">
                  <TabsTrigger
                    value="overview"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <Gauge className="h-4 w-4 mr-2" />
                    Overview
                  </TabsTrigger>
                  <TabsTrigger
                    value="workflows"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <Workflow className="h-4 w-4 mr-2" />
                    Workflows
                  </TabsTrigger>
                  <TabsTrigger
                    value="agents"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <Brain className="h-4 w-4 mr-2" />
                    Agents
                  </TabsTrigger>
                  <TabsTrigger
                    value="performance"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <BarChart3 className="h-4 w-4 mr-2" />
                    Performance
                  </TabsTrigger>
                  <TabsTrigger
                    value="monitoring"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <Monitor className="h-4 w-4 mr-2" />
                    Monitoring
                  </TabsTrigger>
                  <TabsTrigger
                    value="analytics"
                    className="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ease-out data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-white text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <Target className="h-4 w-4 mr-2" />
                    Analytics
                  </TabsTrigger>
                </TabsList>
              </div>

              {/* Tab Content with Apple animations */}
              <TabsContent value="overview" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                >
                  <MetricsOverview
                    metrics={metrics}
                    workflows={workflows}
                    traces={traces}
                  />
                </motion.div>
                <motion.div
                  className="grid grid-cols-1 lg:grid-cols-2 gap-6"
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.1, ease: [0.25, 0.46, 0.45, 0.94] }}
                >
                  <SystemHealth metrics={metrics} />
                  <RealTimeEvents traces={traces} />
                </motion.div>
              </TabsContent>

              <TabsContent value="workflows" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                  className="space-y-6"
                >
                  <WorkflowVisualization workflows={workflows} />
                  <ExecutionTraces traces={traces} />
                </motion.div>
              </TabsContent>

              <TabsContent value="agents" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                >
                  <AgentMonitoring workflows={workflows} metrics={metrics} />
                </motion.div>
              </TabsContent>

              <TabsContent value="performance" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                >
                  <PerformanceCharts metrics={metrics} traces={traces} />
                </motion.div>
              </TabsContent>

              <TabsContent value="monitoring" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                  className="space-y-6"
                >
                  <SystemHealth metrics={metrics} />
                  <RealTimeEvents traces={traces} />
                </motion.div>
              </TabsContent>

              <TabsContent value="analytics" className="space-y-8">
                <motion.div
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -16 }}
                  transition={{ duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] }}
                >
                  <AdvancedAnalytics
                    metrics={metrics}
                    workflows={workflows}
                    traces={traces}
                  />
                </motion.div>
              </TabsContent>
            </Tabs>
          </motion.div>

        </motion.div>
      </main>
    </div>
  )
}
