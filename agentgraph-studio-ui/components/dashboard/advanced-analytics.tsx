'use client'

import { motion } from 'framer-motion'
import { 
  Sparkles, 
  TrendingUp, 
  BarChart3, 
  PieChart, 
  LineChart,
  Target,
  Zap,
  Brain,
  Users,
  Clock,
  DollarSign,
  Award,
  Rocket,
  Shield
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

export function AdvancedAnalytics() {
  const { metrics } = useAgentGraph()

  // Mock advanced analytics data
  const analytics = {
    efficiency_score: 94,
    cost_optimization: 87,
    user_satisfaction: 96,
    innovation_index: 89,
    security_score: 98,
    scalability_rating: 92,
    total_cost_saved: 45600,
    performance_improvement: 156,
    automation_rate: 78,
    prediction_accuracy: 94.5,
    workflow_complexity: 'Medium',
    optimization_opportunities: 12
  }

  const trends = [
    { name: 'Execution Speed', value: 89, change: '+12%', trend: 'up' },
    { name: 'Resource Efficiency', value: 94, change: '+8%', trend: 'up' },
    { name: 'Error Reduction', value: 76, change: '+23%', trend: 'up' },
    { name: 'Cost Optimization', value: 87, change: '+15%', trend: 'up' },
  ]

  const insights = [
    {
      title: 'Peak Performance Hours',
      description: 'System performs 34% better between 2-6 AM',
      icon: Clock,
      color: 'blue'
    },
    {
      title: 'Agent Collaboration',
      description: 'Multi-agent workflows show 67% higher success rates',
      icon: Users,
      color: 'green'
    },
    {
      title: 'Optimization Potential',
      description: '12 workflows can be optimized for better performance',
      icon: Target,
      color: 'orange'
    },
    {
      title: 'Security Excellence',
      description: 'Zero security incidents in the last 30 days',
      icon: Shield,
      color: 'purple'
    }
  ]

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Analytics Overview */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Sparkles className="h-5 w-5 text-purple-500" />
              <span>Advanced Analytics Dashboard</span>
            </CardTitle>
            <CardDescription>AI-powered insights and predictive analytics for your workflows</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                  <Award className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-blue-600">{analytics.efficiency_score}%</p>
                <p className="text-sm text-muted-foreground">Efficiency Score</p>
              </div>
              
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-green-500 to-emerald-600 flex items-center justify-center">
                  <DollarSign className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-green-600">${analytics.total_cost_saved.toLocaleString()}</p>
                <p className="text-sm text-muted-foreground">Cost Saved</p>
              </div>
              
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-orange-500 to-red-600 flex items-center justify-center">
                  <Rocket className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-orange-600">{analytics.performance_improvement}%</p>
                <p className="text-sm text-muted-foreground">Performance Boost</p>
              </div>
              
              <div className="text-center space-y-2">
                <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                  <Brain className="w-8 h-8 text-white" />
                </div>
                <p className="text-2xl font-bold text-purple-600">{analytics.prediction_accuracy}%</p>
                <p className="text-sm text-muted-foreground">Prediction Accuracy</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Performance Trends */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <TrendingUp className="h-5 w-5 text-green-500" />
              <span>Performance Trends</span>
            </CardTitle>
            <CardDescription>Key performance indicators over time</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {trends.map((trend, index) => (
                <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-muted/50 hover:bg-muted/70 transition-colors">
                  <div className="flex items-center space-x-3">
                    <div className="w-2 h-2 rounded-full bg-gradient-to-r from-blue-500 to-purple-500" />
                    <span className="font-medium">{trend.name}</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <Progress value={trend.value} className="w-20 h-2" />
                    <span className="text-sm font-medium w-12">{trend.value}%</span>
                    <Badge variant="success" className="text-xs">
                      {trend.change}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <BarChart3 className="h-5 w-5 text-blue-500" />
              <span>Workflow Analytics</span>
            </CardTitle>
            <CardDescription>Detailed workflow performance metrics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center p-3 rounded-lg bg-blue-50 dark:bg-blue-950/20">
                  <p className="text-2xl font-bold text-blue-600">{analytics.automation_rate}%</p>
                  <p className="text-sm text-muted-foreground">Automation Rate</p>
                </div>
                <div className="text-center p-3 rounded-lg bg-green-50 dark:bg-green-950/20">
                  <p className="text-2xl font-bold text-green-600">{analytics.user_satisfaction}%</p>
                  <p className="text-sm text-muted-foreground">User Satisfaction</p>
                </div>
              </div>
              
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Workflow Complexity</span>
                  <Badge variant="outline">{analytics.workflow_complexity}</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Security Score</span>
                  <div className="flex items-center space-x-2">
                    <Progress value={analytics.security_score} className="w-16 h-2" />
                    <span className="text-sm font-bold text-green-600">{analytics.security_score}%</span>
                  </div>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Scalability Rating</span>
                  <div className="flex items-center space-x-2">
                    <Progress value={analytics.scalability_rating} className="w-16 h-2" />
                    <span className="text-sm font-bold text-blue-600">{analytics.scalability_rating}%</span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* AI Insights */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Brain className="h-5 w-5 text-purple-500" />
              <span>AI-Powered Insights</span>
            </CardTitle>
            <CardDescription>Intelligent recommendations and actionable insights</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {insights.map((insight, index) => {
                const IconComponent = insight.icon
                const colorClasses = {
                  blue: 'bg-blue-50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-800 text-blue-600',
                  green: 'bg-green-50 dark:bg-green-950/20 border-green-200 dark:border-green-800 text-green-600',
                  orange: 'bg-orange-50 dark:bg-orange-950/20 border-orange-200 dark:border-orange-800 text-orange-600',
                  purple: 'bg-purple-50 dark:bg-purple-950/20 border-purple-200 dark:border-purple-800 text-purple-600'
                }
                
                return (
                  <div 
                    key={index} 
                    className={`p-4 rounded-lg border transition-all duration-200 hover:scale-105 ${colorClasses[insight.color as keyof typeof colorClasses]}`}
                  >
                    <div className="flex items-start space-x-3">
                      <IconComponent className="w-5 h-5 mt-0.5" />
                      <div>
                        <h4 className="font-medium mb-1">{insight.title}</h4>
                        <p className="text-sm opacity-80">{insight.description}</p>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Optimization Recommendations */}
      <motion.div variants={itemVariants}>
        <Card className="glass-card border-0">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Target className="h-5 w-5 text-orange-500" />
              <span>Optimization Recommendations</span>
            </CardTitle>
            <CardDescription>AI-generated suggestions to improve your workflows</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-4 rounded-lg bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-950/20 dark:to-purple-950/20 border border-blue-200 dark:border-blue-800">
                <div className="flex items-center space-x-3">
                  <Zap className="w-5 h-5 text-blue-600" />
                  <div>
                    <p className="font-medium">Parallel Processing Opportunity</p>
                    <p className="text-sm text-muted-foreground">3 workflows can benefit from parallel execution</p>
                  </div>
                </div>
                <Badge variant="info">High Impact</Badge>
              </div>
              
              <div className="flex items-center justify-between p-4 rounded-lg bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-950/20 dark:to-emerald-950/20 border border-green-200 dark:border-green-800">
                <div className="flex items-center space-x-3">
                  <DollarSign className="w-5 h-5 text-green-600" />
                  <div>
                    <p className="font-medium">Cost Optimization</p>
                    <p className="text-sm text-muted-foreground">Switch to more efficient LLM models for 23% cost reduction</p>
                  </div>
                </div>
                <Badge variant="success">Medium Impact</Badge>
              </div>
              
              <div className="flex items-center justify-between p-4 rounded-lg bg-gradient-to-r from-orange-50 to-red-50 dark:from-orange-950/20 dark:to-red-950/20 border border-orange-200 dark:border-orange-800">
                <div className="flex items-center space-x-3">
                  <Clock className="w-5 h-5 text-orange-600" />
                  <div>
                    <p className="font-medium">Caching Strategy</p>
                    <p className="text-sm text-muted-foreground">Implement response caching for 40% faster execution</p>
                  </div>
                </div>
                <Badge variant="warning">High Impact</Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  )
}
