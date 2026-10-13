'use client';

import { useState, useCallback } from 'react';
import { Loader2, RefreshCw, Play, Square, Clock, CheckCircle, XCircle } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Input } from '@/components/ui/Input';
import { api, type RunResponse, type RunListResponse, type ListRunsQuery } from '@/api/client';

const statusVariants: Record<string, string> = {
  pending: 'warning',
  running: 'info',
  completed: 'success',
  cancelled: 'secondary',
  failed: 'destructive',
  queued: 'draft',
  claimed: 'publishing',
};

const statusIcons: Record<string, React.ReactNode> = {
  pending: <Clock className="h-3 w-3" />,
  running: <Play className="h-3 w-3" />,
  completed: <CheckCircle className="h-3 w-3" />,
  cancelled: <Square className="h-3 w-3" />,
  failed: <XCircle className="h-3 w-3" />,
  queued: <Clock className="h-3 w-3" />,
  claimed: <Play className="h-3 w-3" />,
};

interface RunListProps {
  initialQuery?: ListRunsQuery;
}

export function RunList({ initialQuery }: RunListProps) {
  const [query, setQuery] = useState<ListRunsQuery>({
    limit: 20,
    offset: 0,
    ...initialQuery,
  });
  const [runs, setRuns] = useState<RunResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const fetchRuns = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: RunListResponse = await api.runs.list(query);
      setRuns(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch runs:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  const handleFilterChange = (newQuery: Partial<ListRunsQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  const formatStatus = (status: string) => {
    return status.charAt(0).toUpperCase() + status.slice(1).replace('_', ' ');
  };

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Production Runs</CardTitle>
        <div className="flex items-center gap-4">
          <Select
            value={query.status ?? ''}
            onValueChange={(value) => handleFilterChange({ status: value || undefined })}
          >
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="All statuses" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Statuses</SelectItem>
              <SelectItem value="pending">Pending</SelectItem>
              <SelectItem value="running">Running</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
              <SelectItem value="cancelled">Cancelled</SelectItem>
              <SelectItem value="failed">Failed</SelectItem>
            </SelectContent>
          </Select>
          <Input
            placeholder="Filter by niche ID..."
            value={query.niche_id ?? ''}
            onChange={(e) => handleFilterChange({ niche_id: e.target.value || undefined })}
            className="w-[200px]"
          />
          <Button
            variant="outline"
            size="icon"
            onClick={fetchRuns}
            disabled={isLoading}
            aria-label="Refresh runs"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && runs.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : runs.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-muted-foreground">No runs found</p>
          </div>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="w-full" role="table">
                <thead>
                  <tr className="border-b text-left text-sm text-muted-foreground">
                    <th className="pb-3 font-medium">Run ID</th>
                    <th className="pb-3 font-medium">Niche</th>
                    <th className="pb-3 font-medium">Status</th>
                    <th className="pb-3 font-medium">Target</th>
                    <th className="pb-3 font-medium">Created</th>
                    <th className="pb-3 font-medium">Started</th>
                    <th className="pb-3 font-medium">Completed</th>
                    <th className="pb-3 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.map((run) => (
                    <tr key={run.id} className="border-b hover:bg-muted/50 transition-colors">
                      <td className="py-3 font-mono text-xs">
                        {run.id.slice(0, 12)}...
                      </td>
                      <td className="py-3 text-sm">{run.niche_id}</td>
                      <td className="py-3">
                        <Badge variant={statusVariants[run.status] || 'outline'} className="gap-1">
                          {statusIcons[run.status]}
                          {formatStatus(run.status)}
                        </Badge>
                      </td>
                      <td className="py-3 text-sm">{run.target_count}</td>
                      <td className="py-3 text-sm text-muted-foreground">
                        {new Date(run.created_at).toLocaleDateString()}
                      </td>
                      <td className="py-3 text-sm text-muted-foreground">
                        {run.started_at ? new Date(run.started_at).toLocaleDateString() : '—'}
                      </td>
                      <td className="py-3 text-sm text-muted-foreground">
                        {run.completed_at ? new Date(run.completed_at).toLocaleDateString() : '—'}
                      </td>
                      <td className="py-3">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => window.location.href = `/runs/${run.id}`}
                        >
                          View
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            {total > query.limit && (
              <div className="flex items-center justify-between mt-6 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  Showing {query.offset + 1} to {Math.min(query.offset + runs.length, total)} of {total}
                </p>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(Math.max(0, query.offset - query.limit))}
                    disabled={query.offset === 0}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(query.offset + query.limit)}
                    disabled={query.offset + runs.length >= total}
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}