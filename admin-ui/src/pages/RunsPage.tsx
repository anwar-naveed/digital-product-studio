// Runs page component
import { useRuns } from '../hooks/useRuns';
import { useCreateRun, useCancelRun } from '../hooks/useRuns';
import { useNiches } from '../hooks/useNiches';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDateTime, formatNumber } from '../lib/utils';
import { Plus, XCircle, Loader2, Play } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const runSchema = z.object({
  niche_id: z.string().min(1, 'Niche is required'),
  target_count: z.number().min(1, 'Must be at least 1').max(1000),
  params: z.record(z.unknown()).optional(),
});

type RunFormData = z.infer<typeof runSchema>;

interface RunDialogProps {
  niches: { id: string; name: string }[];
  onClose: () => void;
}

function RunDialog({ niches, onClose }: RunDialogProps) {
  const createRun = useCreateRun();
  
  const { register, handleSubmit, reset, watch: _watch, formState: { errors, isSubmitting } } = useForm<RunFormData>({
    resolver: zodResolver(runSchema),
    defaultValues: {
      niche_id: '',
      target_count: 10,
      params: {},
    },
  });

  const onSubmit = async (data: RunFormData) => {
    try {
      await createRun.mutateAsync(data);
      reset();
      onClose();
    } catch (err) {
      console.error('Failed to create run:', err);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6">
        <h2 className="text-xl font-semibold mb-4">Spawn Production Run</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Niche</label>
            <select
              {...register('niche_id')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
            >
              <option value="">Select a niche</option>
              {niches.map((niche) => (
                <option key={niche.id} value={niche.id}>{niche.name}</option>
              ))}
            </select>
            {errors.niche_id && <p className="text-red-500 text-sm mt-1">{errors.niche_id.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Target Count</label>
            <input
              {...register('target_count', { valueAsNumber: true })}
              type="number"
              min="1"
              max="1000"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
            />
            {errors.target_count && <p className="text-red-500 text-sm mt-1">{errors.target_count.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Parameters (JSON)</label>
            <textarea
              {...register('params')}
              rows={4}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 font-mono text-sm"
              placeholder='{"style": "gothic", "count": 3}'
            />
          </div>
          <div className="flex justify-end gap-2 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border rounded-md text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting || createRun.isPending}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90 disabled:opacity-50"
            >
              {isSubmitting || createRun.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  Creating...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Spawn Run
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

const statusColors: Record<string, string> = {
  pending: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
  running: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
  completed: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
  cancelled: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400',
  failed: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
};

export function RunsPage() {
  const [showDialog, setShowDialog] = useState(false);
  const cancelRun = useCancelRun();
  
  const { data: nichesData } = useNiches({ limit: 100, offset: 0 });
  const { data, isLoading, error } = useRuns();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-red-500">Failed to load runs</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Production Runs</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">Manage and monitor production runs</p>
        </div>
        <button onClick={() => setShowDialog(true)} className="btn-primary">
          <Plus className="h-4 w-4 mr-2" /> Spawn Run
        </button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-gray-50 dark:bg-gray-800/50">
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Niche</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Target</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Progress</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Created</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {data?.items?.map((run) => {
                  const niche = nichesData?.items?.find((n) => n.id === run.niche_id);
                  const totalJobs = run.target_count;
                  const completedJobs = 0; // Would need job count from detail
                  const progress = totalJobs > 0 ? (completedJobs / totalJobs) * 100 : 0;
                  
                  return (
                    <tr key={run.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                      <td className="px-6 py-4 font-mono text-sm text-gray-900 dark:text-white">
                        {run.id.slice(0, 8)}...
                      </td>
                      <td className="px-6 py-4 text-gray-600 dark:text-gray-300">
                        {niche?.name ?? run.niche_id.slice(0, 8)}
                      </td>
                      <td className="px-6 py-4">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${statusColors[run.status] ?? 'bg-gray-100 text-gray-800'}`}>
                          {run.status}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-gray-600 dark:text-gray-300">{formatNumber(run.target_count)}</td>
                      <td className="px-6 py-4">
                        <div className="w-32 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-primary transition-all"
                            style={{ width: `${progress}%` }}
                          />
                        </div>
                        <span className="text-xs text-gray-500 ml-1">{progress.toFixed(0)}%</span>
                      </td>
                      <td className="px-6 py-4 text-gray-600 dark:text-gray-300">{formatDateTime(run.created_at)}</td>
                      <td className="px-6 py-4 text-right">
                        {(run.status === 'pending' || run.status === 'running') && (
                          <button
                            onClick={() => cancelRun.mutate(run.id)}
                            disabled={cancelRun.isPending}
                            className="p-2 text-gray-500 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md disabled:opacity-50"
                          >
                            <XCircle className="h-4 w-4" />
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
                {(!data?.items?.length) && (
                  <tr>
                    <td colSpan={7} className="px-6 py-12 text-center text-gray-500 dark:text-gray-400">
                      No runs found. Spawn your first production run to get started.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {showDialog && (
        <RunDialog
          niches={nichesData?.items ?? []}
          onClose={() => setShowDialog(false)}
        />
      )}
    </div>
  );
}