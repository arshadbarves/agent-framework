'use client'

import { motion } from 'framer-motion'
import {
  Bot,
  Brain,
  Zap,
  DollarSign,
  Clock,
  Target,
  TrendingUp,
  Activity,
  Users,
  MessageSquare,
  CheckCircle2,
  AlertCircle,
  Cpu,
  BarChart3
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

export function AgentMonitoring() {
  const { metrics } = useAgentGraph()

  // Mock agent data for demonstration
  const agents = [
    {
      id: 'research-agent',
      name: 'Research Agent',
      type: 'Information Gathering',
      status: 'active',
      tasks_completed: 156,
      success_rate: 94,
      avg_response_time: 2.3,
      tokens_used: 45600,
      cost: 12.45,
      efficiency: 89,
      last_activity: '2 minutes ago'
    },
    {
      id: 'writing-agent',
      name: 'Writing Agent',
      type: 'Content Creation',
      status: 'active',
      tasks_completed: 89,
      success_rate: 97,
      avg_response_time: 3.1,
      tokens_used: 67800,
      cost: 18.92,
      efficiency: 92,
      last_activity: '5 minutes ago'
    },
    {
      id: 'analysis-agent',
      name: 'Analysis Agent',
      type: 'Data Processing',
      status: 'idle',
      tasks_completed: 234,
      success_rate: 91,
      avg_response_time: 1.8,
      tokens_used: 34200,
      cost: 9.87,
      efficiency: 87,
      last_activity: '15 minutes ago'
    },
    {
      id: 'quality-agent',
      name: 'Quality Agent',
      type: 'Review & Validation',
      status: 'active',
      tasks_completed: 67,
      success_rate: 99,
      avg_response_time: 1.2,
      tokens_used: 23400,
      cost: 6.78,
      efficiency: 96,
      last_activity: '1 minute ago'
    }
  ]

  const totalStats = {
    total_agents: agents.length,
    active_agents: agents.filter(a => a.status === 'active').length,
    total_tasks: agents.reduce((sum, a) => sum + a.tasks_completed, 0),
    avg_success_rate: agents.reduce((sum, a) => sum + a.success_rate, 0) / agents.length,
    total_cost: agents.reduce((sum, a) => sum + a.cost, 0),
    total_tokens: agents.reduce((sum, a) => sum + a.tokens_used, 0)
  }

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Agent Overview */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Bot className="h-5 w-5 text-blue-500" />
              <span>Agent Monitoring Dashboard</span>
            </CardTitle>
            <CardDescription>Real-time monitoring of AI agent performance and behavior</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                  <Users className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-blue-600">{totalStats.active_agents}/{totalStats.total_agents}</p>
                <p className="text-sm text-muted-foreground">Active Agents</p>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-green-500 to-emerald-600 flex items-center justify-center">
                  <CheckCircle2 className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-green-600">{totalStats.total_tasks}</p>
                <p className="text-sm text-muted-foreground">Tasks Completed</p>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-orange-500 to-red-600 flex items-center justify-center">
                  <Target className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-orange-600">{Math.round(totalStats.avg_success_rate)}%</p>
                <p className="text-sm text-muted-foreground">Avg Success Rate</p>
              </div>

              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                  <DollarSign className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-purple-600">${totalStats.total_cost.toFixed(2)}</p>
                <p className="text-sm text-muted-foreground">Total Cost</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Individual Agent Cards */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {agents.map((agent, index) => (
          <motion.div key={agent.id} variants={itemVariants}>
            <Card className="glass-card border-0 hover:scale-105 transition-all duration-300">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      agent.status === 'active'
                        ? 'bg-green-100 dark:bg-green-900/20'
                        : 'bg-gray-100 dark:bg-gray-900/20'
                    }`}>
                      <Brain className={`w-5 h-5 ${
                        agent.status === 'active' ? 'text-green-600' : 'text-gray-600'
                      }`} />
                    </div>
                    <div>
                      <CardTitle className="text-lg">{agent.name}</CardTitle>
                      <CardDescription className="text-sm">{agent.type}</CardDescription>
                    </div>
                  </div>
                  <Badge
                    variant={agent.status === 'active' ? 'success' : 'secondary'}
                    className="text-xs"
                  >
                    {agent.status === 'active' ? (
                      <Activity className="w-3 h-3 mr-1" />
                    ) : (
                      <Clock className="w-3 h-3 mr-1" />
                    )}
                    {agent.status}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-1">
                      <p className="text-sm font-medium text-muted-foreground">Tasks Completed</p>
                      <p className="text-xl font-bold">{agent.tasks_completed}</p>
                    </div>
                    <div className="space-y-1">
                      <p className="text-sm font-medium text-muted-foreground">Success Rate</p>
                      <p className="text-xl font-bold text-green-600">{agent.success_rate}%</p>
                    </div>
                  </div>

                  <div className="space-y-3">
                    <div className="flex justify-between items-center">
                      <span className="text-sm font-medium">Efficiency Score</span>
                      <div className="flex items-center space-x-2">
                        <Progress value={agent.efficiency} className="w-16 h-2" />
                        <span className="text-sm font-bold">{agent.efficiency}%</span>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div className="flex items-center space-x-2">
                        <Clock className="w-4 h-4 text-blue-500" />
                        <span>{agent.avg_response_time}s avg</span>
                      </div>
                      <div className="flex items-center space-x-2">
                        <MessageSquare className="w-4 h-4 text-purple-500" />
                        <span>{agent.tokens_used.toLocaleString()} tokens</span>
                      </div>
                    </div>

                    <div className="flex justify-between items-center pt-2 border-t">
                      <span className="text-sm text-muted-foreground">Cost: ${agent.cost}</span>
                      <span className="text-sm text-muted-foreground">{agent.last_activity}</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </motion.div>

      {/* Agent Performance Comparison */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <BarChart3 className="h-5 w-5 text-green-500" />
              <span>Agent Performance Comparison</span>
            </CardTitle>
            <CardDescription>Comparative analysis of agent efficiency and performance metrics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {agents.map((agent, index) => (
                <div key={agent.id} className="flex items-center justify-between p-3 rounded-lg bg-muted/50 hover:bg-muted/70 transition-colors">
                  <div className="flex items-center space-x-3">
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-white ${
                      index === 0 ? 'bg-blue-500' :
                      index === 1 ? 'bg-green-500' :
                      index === 2 ? 'bg-orange-500' : 'bg-purple-500'
                    }`}>
                      {agent.name.charAt(0)}
                    </div>
                    <div>
                      <p className="font-medium">{agent.name}</p>
                      <p className="text-sm text-muted-foreground">{agent.type}</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-6">
                    <div className="text-center">
                      <p className="text-sm font-medium">{agent.efficiency}%</p>
                      <p className="text-xs text-muted-foreground">Efficiency</p>
                    </div>
                    <div className="text-center">
                      <p className="text-sm font-medium">{agent.success_rate}%</p>
                      <p className="text-xs text-muted-foreground">Success</p>
                    </div>
                    <div className="text-center">
                      <p className="text-sm font-medium">${agent.cost}</p>
                      <p className="text-xs text-muted-foreground">Cost</p>
                    </div>
                    <Progress value={agent.efficiency} className="w-20 h-2" />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  )
}
