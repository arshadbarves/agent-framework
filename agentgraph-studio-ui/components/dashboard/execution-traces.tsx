'use client'

import { Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { useAgentGraph } from '@/hooks/use-agentgraph'
import { getStatusColor, getRelativeTime } from '@/lib/utils'

export function ExecutionTraces() {
  const { traces } = useAgentGraph()

  return (
    <Card className="bg-white dark:bg-neutral-800 border border-gray-200 dark:border-neutral-700 shadow-sm">
      <CardHeader>
        <CardTitle className="text-gray-900 dark:text-neutral-100">Execution Traces</CardTitle>
        <CardDescription className="text-gray-600 dark:text-neutral-400">Recent workflow execution details and debugging information</CardDescription>
      </CardHeader>
      <CardContent>
        {traces.length === 0 ? (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-neutral-700 flex items-center justify-center">
              <Activity className="w-8 h-8 text-gray-400" />
            </div>
            <p className="text-gray-600 dark:text-neutral-400 font-medium">No execution traces available</p>
            <p className="text-sm text-gray-500 dark:text-neutral-500 mt-2">
              Execute workflows to see traces here
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {traces.slice(0, 10).map((trace) => (
              <div key={trace.id} className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <div>
                    <h4 className="font-medium">Execution {trace.execution_id.slice(0, 8)}</h4>
                    <p className="text-sm text-muted-foreground">
                      Workflow: {trace.workflow_id}
                    </p>
                  </div>
                  <div className="text-right">
                    <Badge className={getStatusColor(trace.status)}>
                      {trace.status}
                    </Badge>
                    <p className="text-xs text-muted-foreground mt-1">
                      {getRelativeTime(trace.start_time)}
                    </p>
                  </div>
                </div>
                
                <div className="text-sm">
                  <p className="text-muted-foreground">
                    Events: {trace.events.length}
                  </p>
                  {trace.end_time && (
                    <p className="text-muted-foreground">
                      Duration: {Math.round(
                        (new Date(trace.end_time).getTime() - new Date(trace.start_time).getTime())
                      )}ms
                    </p>
                  )}
                  {trace.error && (
                    <p className="text-red-600 text-xs mt-2">
                      Error: {trace.error}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
