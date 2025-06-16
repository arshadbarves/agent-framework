'use client'

import { Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { useAgentGraph } from '@/hooks/use-agentgraph'
import { getRelativeTime } from '@/lib/utils'

export function RealTimeEvents() {
  const { events, isConnected } = useAgentGraph()

  return (
    <Card className="bg-white dark:bg-neutral-800 border border-gray-200 dark:border-neutral-700 shadow-sm">
      <CardHeader>
        <CardTitle className="flex items-center justify-between text-gray-900 dark:text-neutral-100">
          Real-time Events
          <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`} />
        </CardTitle>
        <CardDescription className="text-gray-600 dark:text-neutral-400">Live event stream from AgentGraph workflows</CardDescription>
      </CardHeader>
      <CardContent>
        {events.length === 0 ? (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-neutral-700 flex items-center justify-center">
              <Activity className="w-8 h-8 text-gray-400" />
            </div>
            <p className="text-gray-600 dark:text-neutral-400 font-medium">
              {isConnected ? 'No recent events' : 'Connecting to event stream...'}
            </p>
            <p className="text-sm text-gray-500 dark:text-neutral-500 mt-2">
              Events will appear here when workflows are executed
            </p>
          </div>
        ) : (
          <div className="space-y-3 max-h-96 overflow-y-auto custom-scrollbar">
            {events.slice(0, 20).map((event) => (
              <div key={event.id} className="border rounded-lg p-3">
                <div className="flex items-center justify-between mb-2">
                  <Badge variant="outline" className="text-xs">
                    {event.event_type}
                  </Badge>
                  <span className="text-xs text-muted-foreground">
                    {getRelativeTime(event.timestamp)}
                  </span>
                </div>
                
                <div className="text-sm">
                  <p className="font-medium">Execution: {event.execution_id.slice(0, 8)}</p>
                  {event.node_id && (
                    <p className="text-muted-foreground">Node: {event.node_id}</p>
                  )}
                  {event.data && Object.keys(event.data).length > 0 && (
                    <div className="mt-2 p-2 bg-muted rounded text-xs">
                      <pre className="whitespace-pre-wrap">
                        {JSON.stringify(event.data, null, 2)}
                      </pre>
                    </div>
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
