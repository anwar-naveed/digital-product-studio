'use client';

import { useState, useEffect } from 'react';
import { Loader2, Play, Square, Clock, CheckCircle, XCircle, AlertTriangle, Settings, FileText } from 'lucide-react';
//import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';

import { api, type RunDetailResponse, type JobSummary } from '@/api/client';

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

const jobStatusVariants: Record<string, string> = {
  queued: 'draft',
  claimed: 'publishing',
  running: 'info',
  done: 'success',
  failed: 'destructive',
  dead: 'secondary',
};

const jobStatusIcons: Record<string, React.ReactNode> = {
  queued: <Clock className="h-3 w-3" />,
  claimed: <Play className="h-3 w-3" />,
  running: <Settings className="h-3 w-3 animate-spin" />,
  done: <CheckCircle className="h-3 w-3" />,
  failed: <XCircle className="h-3 w-3" />,
  dead: <AlertTriangle className="h-3 w-3" />,
};

const formatStatus = (status: string) => {
  return status.charAt(0).toUpperCase() + status.slice(1).replace('_', ' ');
};

const formatDate = (dateStr: string | null) => {
  return dateStr ? new Date(dateStr).toLocaleString() : '—';
}

interface RunDetailProps {
  runId: string;
}

export function RunDetail({ runId }: RunDetailProps) {
  const [run, setRun] = useState<RunDetailResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchRun = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const data = await api.runs.get(runId);
        setRun(data);
      } catch (err) {
        setError('Failed to load run details');
        console.error(err);
      } finally {
        setIsLoading(false);
      }
    };

    if (runId) {
      fetchRun();
    }
  }, [runId]);

  if (isLoading && !run) {
    return (
      <Card className="w-full">
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  if (error || !run) {
    return (
      <Card className="w-full">
        <CardContent className="flex flex-col items-center justify-center py-12 text-center">
          <AlertTriangle className="h-8 w-8 text-destructive mb-4" />
          <p className="text-muted-foreground">{error || 'Run not found'}</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Run Header */}
      <Card className="w-full">
        <CardHeader>
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <CardTitle className="flex items-center gap-2">
                Run Details
                <Badge variant={statusVariants[run.status] || 'outline'} className="gap-1">
                  {statusIcons[run.status]}
                  {formatStatus(run.status)}
                </Badge>
              </CardTitle>
              <CardDescription>Production run {run.id.slice(0, 12)}...</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              {run.status === 'pending' && (
                <Button onClick={() => window.location.reload()}>Refresh</Button>
              )}
              <Button variant="outline" onClick={() => window.location.href = '/runs'}>
                Back to List
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Run ID</p>
            <p className="font-mono text-sm">{run.id}</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Niche</p>
            <p className="font-medium">{run.niche_id}</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Target Count</p>
            <p className="font-medium">{run.target_count}</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Progress</p>
            <p className="font-medium">
              {run.jobs?.filter(j => j.status === 'done').length ?? 0} / {run.target_count}
            </p>
          </div>
          <div className="space-y-1 sm:col-span-2">
            <p className="text-sm text-muted-foreground">Created</p>
            <p className="font-medium">{formatDate(run.created_at)}</p>
          </div>
          <div className="space-y-1 sm:col-span-2">
            <p className="text-sm text-muted-foreground">Started</p>
            <p className="font-medium">{formatDate(run.started_at)}</p>
          </div>
          <div className="space-y-1 sm:col-span-2">
            <p className="text-sm text-muted-foreground">Completed</p>
            <p className="font-medium">{formatDate(run.completed_at)}</p>
          </div>
          <div className="space-y-1 sm:col-span-2">
            <p className="text-sm text-muted-foreground">Parameters</p>
            <pre className="text-xs bg-muted p-3 rounded overflow-auto max-h-32">
              {JSON.stringify(run.params, null, 2)}
            </pre>
          </div>
        </CardContent>
      </Card>

      {/* Jobs Breakdown */}
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Job Breakdown ({run.jobs?.length ?? 0} jobs)
          </CardTitle>
        </CardHeader>
        <CardContent>
          {run.jobs && run.jobs.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full" role="table">
                <thead>
                  <tr className="border-b text-left text-sm text-muted-foreground">
                    <th className="pb-3 font-medium">Job ID</th>
                    <th className="pb-3 font-medium">Tool</th>
                    <th className="pb-3 font-medium">Status</th>
                    <th className="pb-3 font-medium">Attempts</th>
                    <th className="pb-3 font-medium">Queued</th>
                    <th className="pb-3 font-medium">Started</th>
                    <th className="pb-3 font-medium">Finished</th>
                    <th className="pb-3 font-medium">Last Error</th>
                  </tr>
                </thead>
                <tbody>
                  {run.jobs.map((job: JobSummary) => (
                    <tr key={job.id} className="border-b hover:bg-muted/50 transition-colors">
                      <td className="py-3 font-mono text-xs">{job.id.slice(0, 12)}...</td>
                      <td className="py-3 text-sm font-medium">{job.tool}</td>
                      <td className="py-3">
                        <Badge variant={jobStatusVariants[job.status] || 'outline'} className="gap-1">
                          {jobStatusIcons[job.status]}
                          {formatStatus(job.status)}
                        </Badge>
                      </td>
                      <td className="py-3 text-sm">{job.attempts}</td>
                      <td className="py-3 text-sm text-muted-foreground">{formatDate(job.queued_at)}</td>
                      <td className="py-3 text-sm text-muted-foreground">{formatDate(job.started_at)}</td>
                      <td className="py-3 text-sm text-muted-foreground">{formatDate(job.finished_at)}</td>
                      <td className="py-3 text-sm text-muted-foreground max-w-xs truncate">
                        {job.last_error ?? '—'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <p className="text-muted-foreground text-center py-8">No jobs found for this run</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}