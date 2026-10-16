// Runs page component
import { useRuns } from '../hooks/useRuns';
import { api, assetUrl } from '../api/client';
import type { RunVariantItem as RunVariant } from '../types/api';
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

/** Thumbnail strip of every image generated for a run (live previews). */
function RunVariantsStrip({ runId }: { runId: string }) {
  const [variants, setVariants] = useState<RunVariant[] | null>(null);
  const [open, setOpen] = useState(false);

  const load = async () => {
    setOpen((o) => !o);
    if (!variants) {
      try {
        const res = await api.runs.variants(runId);
        setVariants(res.items ?? []);
      } catch {
        setVariants([]);
      }
    }
  };

  return (
    <div className="mt-2">
      <button
        onClick={load}
        className="inline-flex items-center gap-1 rounded-md border border-input bg-background px-2 py-1 text-xs text-foreground transition-colors hover:bg-accent"
      >
        {open ? 'Hide' : 'Preview'} images
      </button>
      {open && (
        <div className="mt-2 flex flex-wrap gap-2">
          {variants === null && <LoadingSpinner />}
          {variants?.length === 0 && (
            <span className="text-xs text-muted-foreground">
              No generated images yet for this run.
            </span>
          )}
          {variants?.map((v) => (
            <a key={v.variant_id} href={assetUrl(v.s3_key)} target="_blank" rel="noreferrer" title={v.s3_key}>
              <img
                src={assetUrl(v.s3_key)}
                alt={`variant ${v.variant_id.slice(0, 8)}`}
                className="h-16 w-16 rounded-md border border-border object-cover"
              />
            </a>
          ))}
        </div>
      )}
    </div>
  );
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 overflow-y-auto">
      <div className="w-full max-w-md bg-card text-card-foreground rounded-lg shadow-xl p-6 border border-border">
        <h2 className="text-xl font-semibold mb-4">Spawn Production Run</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Niche</label>
            <select
              {...register('niche_id')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
            >
              <option value="">Select a niche</option>
              {niches.map((niche) => (
                <option key={niche.id} value={niche.id}>{niche.name}</option>
              ))}
            </select>
            {errors.niche_id && <p className="text-destructive text-sm mt-1">{errors.niche_id.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Target Count</label>
            <input
              {...register('target_count', { valueAsNumber: true })}
              type="number"
              min="1"
              max="1000"
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
            />
            {errors.target_count && <p className="text-destructive text-sm mt-1">{errors.target_count.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Prompt Template</label>
            <textarea
              {...register('params.prompt_template')}
              rows={3}
              className="input font-mono text-sm"
              placeholder={'Beautiful {name} design, {VARIANT} — {name} = niche name, {VARIANT} = per-variant suffix'}
            />
            <p className="text-xs text-muted-foreground mt-1">
              Supports <code className="font-mono">{'{name}'}</code> and <code className="font-mono">{'{VARIANT}'}</code>.
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Style Keywords</label>
            <input
              {...register('params.style')}
              type="text"
              className="input"
              placeholder="minimalist, earth tones, high detail (optional)"
            />
          </div>
          <div className="flex justify-end gap-2 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border rounded-md text-sm hover:bg-accent"
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
  cancelled: 'bg-muted text-foreground dark:bg-gray-700 dark:text-gray-400',
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
    return <div className="text-destructive">Failed to load runs</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Production Runs</h1>
          <p className="text-muted-foreground mt-1">Manage and monitor production runs</p>
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
                <tr className="border-b bg-muted">
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Niche</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Status</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Target</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Progress</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Created</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-muted-foreground uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {data?.items?.map((run) => {
                  const niche = nichesData?.items?.find((n) => n.id === run.niche_id);
                  const totalJobs = run.target_count;
                  const completedJobs = 0; // Would need job count from detail
                  const progress = totalJobs > 0 ? (completedJobs / totalJobs) * 100 : 0;
                  
                  return (
                    <tr key={run.id} className="hover:bg-accent">
                      <td className="px-6 py-4 font-mono text-sm text-foreground">
                        {run.id.slice(0, 8)}...
                      </td>
                      <td className="px-6 py-4 text-muted-foreground">
                        <div>{niche?.name ?? run.niche_id.slice(0, 8)}</div>
                        <RunVariantsStrip runId={run.id} />
                      </td>
                      <td className="px-6 py-4">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${statusColors[run.status] ?? 'bg-muted text-foreground'}`}>
                          {run.status}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-muted-foreground">{formatNumber(run.target_count)}</td>
                      <td className="px-6 py-4">
                        <div className="w-32 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-primary transition-all"
                            style={{ width: `${progress}%` }}
                          />
                        </div>
                        <span className="text-xs text-gray-500 ml-1">{progress.toFixed(0)}%</span>
                      </td>
                      <td className="px-6 py-4 text-muted-foreground">{formatDateTime(run.created_at)}</td>
                      <td className="px-6 py-4 text-right">
                        {(run.status === 'pending' || run.status === 'generating') && (
                          <button
                            onClick={() => cancelRun.mutate(run.id)}
                            disabled={cancelRun.isPending}
                            className="p-2 text-gray-500 hover:text-destructive hover:bg-accent rounded-md disabled:opacity-50"
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
                    <td colSpan={7} className="px-6 py-12 text-center text-muted-foreground">
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