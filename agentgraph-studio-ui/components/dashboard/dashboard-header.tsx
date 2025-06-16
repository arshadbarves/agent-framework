'use client'

import { Activity, Settings, RefreshCw, Wifi, WifiOff, Zap } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ThemeToggle } from '@/components/ui/theme-toggle'

interface DashboardHeaderProps {
  isConnected: boolean
  isLoading: boolean
  onRefresh: () => void
}

export function DashboardHeader({ isConnected, isLoading, onRefresh }: DashboardHeaderProps) {

  return (
    <header className="bg-white/80 dark:bg-black/80 backdrop-blur-xl border-b border-gray-200 dark:border-gray-800 sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-xl">
                <Activity className="h-6 w-6 text-blue-600 dark:text-blue-400" />
              </div>
              <div>
                <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
                  AgentGraph Studio
                </h1>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Workflow Management
                </p>
              </div>
            </div>

            {/* Connection Status */}
            <div className="hidden lg:flex items-center space-x-2 px-3 py-1.5 bg-gray-100 dark:bg-gray-800 rounded-lg">
              {isConnected ? (
                <>
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                  <span className="text-xs font-medium text-gray-700 dark:text-gray-300">Connected</span>
                </>
              ) : (
                <>
                  <div className="w-2 h-2 bg-red-500 rounded-full" />
                  <span className="text-xs font-medium text-gray-700 dark:text-gray-300">Disconnected</span>
                </>
              )}
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <Button
              variant="outline"
              size="sm"
              onClick={onRefresh}
              disabled={isLoading}
              className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-all duration-200 ease-out"
            >
              <RefreshCw className={`h-4 w-4 mr-2 transition-transform duration-200 ${isLoading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>

            <ThemeToggle />

            <Button
              variant="outline"
              size="sm"
              className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-all duration-200 ease-out"
            >
              <Settings className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </header>
  )
}
