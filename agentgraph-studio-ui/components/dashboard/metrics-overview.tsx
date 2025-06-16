'use client'

import { Activity, TrendingUp, Clock, CheckCircle, Workflow, Users } from 'lucide-react'
import { formatNumber } from '@/lib/utils'
import { SystemMetrics, VisualWorkflow, ExecutionTrace } from '@/lib/types'

interface MetricsOverviewProps {
  metrics: SystemMetrics | null
  workflows: VisualWorkflow[] | null
  traces: ExecutionTrace[] | null
}

export function MetricsOverview({ metrics, workflows, traces }: MetricsOverviewProps) {

  if (!metrics) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {[1, 2, 3, 4].map((i) => (
          <div key={i} className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
            <div className="animate-pulse">
              <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded-lg w-3/4 mb-3"></div>
              <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded-lg w-1/2 mb-4"></div>
              <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-lg w-2/3"></div>
            </div>
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Apple-style Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Executions - Apple Card Style */}
        <div className="group bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700 transition-all duration-200 ease-out hover:shadow-lg hover:shadow-gray-900/5 dark:hover:shadow-black/20">
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Total Executions</p>
              <p className="text-3xl font-semibold text-gray-900 dark:text-white tracking-tight">
                {formatNumber(metrics.total_executions)}
              </p>
            </div>
            <div className="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-xl">
              <Activity className="h-5 w-5 text-blue-600 dark:text-blue-400" />
            </div>
          </div>
          <div className="mt-4 flex items-center space-x-1">
            <TrendingUp className="h-3 w-3 text-green-600 dark:text-green-400" />
            <span className="text-xs font-medium text-green-600 dark:text-green-400">+12.5%</span>
            <span className="text-xs text-gray-500 dark:text-gray-500">from last week</span>
          </div>
        </div>

        {/* Active Workflows */}
        <div className="group bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700 transition-all duration-200 ease-out hover:shadow-lg hover:shadow-gray-900/5 dark:hover:shadow-black/20">
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Active Workflows</p>
              <p className="text-3xl font-semibold text-gray-900 dark:text-white tracking-tight">
                {metrics.active_executions}
              </p>
            </div>
            <div className="p-2 bg-green-100 dark:bg-green-900/30 rounded-xl">
              <Workflow className="h-5 w-5 text-green-600 dark:text-green-400" />
            </div>
          </div>
          <div className="mt-4 flex items-center space-x-1">
            <Users className="h-3 w-3 text-blue-600 dark:text-blue-400" />
            <span className="text-xs font-medium text-blue-600 dark:text-blue-400">{workflows?.length || 0}</span>
            <span className="text-xs text-gray-500 dark:text-gray-500">total workflows</span>
          </div>
        </div>

        {/* Success Rate */}
        <div className="group bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700 transition-all duration-200 ease-out hover:shadow-lg hover:shadow-gray-900/5 dark:hover:shadow-black/20">
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Success Rate</p>
              <p className="text-3xl font-semibold text-gray-900 dark:text-white tracking-tight">
                {Math.round(metrics.success_rate)}%
              </p>
            </div>
            <div className="p-2 bg-emerald-100 dark:bg-emerald-900/30 rounded-xl">
              <CheckCircle className="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
            </div>
          </div>
          <div className="mt-4">
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className="bg-emerald-500 h-2 rounded-full transition-all duration-300 ease-out"
                style={{ width: `${metrics.success_rate}%` }}
              />
            </div>
          </div>
        </div>

        {/* Avg Response Time */}
        <div className="group bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700 transition-all duration-200 ease-out hover:shadow-lg hover:shadow-gray-900/5 dark:hover:shadow-black/20">
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Avg Response Time</p>
              <p className="text-3xl font-semibold text-gray-900 dark:text-white tracking-tight">
                {Math.round(metrics.avg_execution_time_ms)}ms
              </p>
            </div>
            <div className="p-2 bg-orange-100 dark:bg-orange-900/30 rounded-xl">
              <Clock className="h-5 w-5 text-orange-600 dark:text-orange-400" />
            </div>
          </div>
          <div className="mt-4 flex items-center space-x-1">
            <TrendingUp className="h-3 w-3 text-green-600 dark:text-green-400 rotate-180" />
            <span className="text-xs font-medium text-green-600 dark:text-green-400">-15%</span>
            <span className="text-xs text-gray-500 dark:text-gray-500">improvement</span>
          </div>
        </div>
      </div>

      {/* Apple-style Additional Metrics */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Execution Summary */}
        <div className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">Execution Summary</h3>
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-green-500 rounded-full" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Completed</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {formatNumber(metrics.completed_executions)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-red-500 rounded-full" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Failed</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {formatNumber(metrics.failed_executions)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Running</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {formatNumber(metrics.active_executions)}
              </span>
            </div>
          </div>
        </div>

        {/* Resource Usage */}
        <div className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">Resource Usage</h3>
          <div className="space-y-6">
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">CPU</span>
                <span className="text-sm font-semibold text-gray-900 dark:text-white">{metrics.resource_metrics.cpu_usage}%</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${metrics.resource_metrics.cpu_usage}%` }}
                />
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Memory</span>
                <span className="text-sm font-semibold text-gray-900 dark:text-white">{metrics.resource_metrics.memory_usage_percent}%</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div
                  className="bg-green-500 h-2 rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${metrics.resource_metrics.memory_usage_percent}%` }}
                />
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Network</span>
                <span className="text-sm font-semibold text-gray-900 dark:text-white">{metrics.resource_metrics.network_rps} req/s</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div
                  className="bg-purple-500 h-2 rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${Math.min(metrics.resource_metrics.network_rps / 10, 100)}%` }}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Recent Activity */}
        <div className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">Recent Activity</h3>
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-cyan-500 rounded-full animate-pulse" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Recent Traces</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {traces?.length || 0}
              </span>
            </div>

            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-indigo-500 rounded-full" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Active Threads</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {metrics.resource_metrics.active_threads}
              </span>
            </div>

            <div className="flex justify-between items-center">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 bg-purple-500 rounded-full" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Disk IOPS</span>
              </div>
              <span className="text-sm font-semibold text-gray-900 dark:text-white">
                {metrics.resource_metrics.disk_iops}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
