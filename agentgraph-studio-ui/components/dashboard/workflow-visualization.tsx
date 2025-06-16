'use client'

import { Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { useAgentGraph } from '@/hooks/use-agentgraph'
import { getStatusColor, getNodeTypeColor } from '@/lib/utils'

export function WorkflowVisualization() {
  const { workflows } = useAgentGraph()

  return (
    <Card className="bg-white dark:bg-neutral-800 border border-gray-200 dark:border-neutral-700 shadow-sm">
      <CardHeader>
        <CardTitle className="text-gray-900 dark:text-neutral-100">Workflow Visualization</CardTitle>
        <CardDescription className="text-gray-600 dark:text-neutral-400">Interactive workflow graphs and execution status</CardDescription>
      </CardHeader>
      <CardContent>
        {workflows.length === 0 ? (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-neutral-700 flex items-center justify-center">
              <Activity className="w-8 h-8 text-gray-400" />
            </div>
            <p className="text-gray-600 dark:text-neutral-400 font-medium">No workflows available</p>
            <p className="text-sm text-gray-500 dark:text-neutral-500 mt-2">
              Start the AgentGraph backend to see workflows
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {workflows.map((workflow) => (
              <div key={workflow.id} className="border border-gray-200 dark:border-neutral-700 rounded-lg p-4 bg-gray-50 dark:bg-neutral-900/50">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-semibold text-gray-900 dark:text-neutral-100">{workflow.name}</h3>
                  <Badge className={getStatusColor(workflow.status)}>
                    {workflow.status}
                  </Badge>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {workflow.nodes.map((node) => (
                    <div
                      key={node.id}
                      className={`workflow-node ${node.node_type} ${node.status} p-3 rounded-lg text-sm`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-medium">{node.name}</span>
                        <div className={`status-indicator ${node.status}`} />
                      </div>
                      <p className="text-xs opacity-75 mt-1">{node.node_type}</p>
                      {node.execution_time_ms && (
                        <p className="text-xs opacity-75">
                          {Math.round(node.execution_time_ms)}ms
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
