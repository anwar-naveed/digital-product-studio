// Niche Board page component
import { useNiches } from '../hooks/useNiches';
import { useCreateRun } from '../hooks/useRuns';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatNumber } from '../lib/utils';
import { TrendingDown, Play, TrendingUp, Target, Loader2, Zap } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const spawnRunSchema = z.object({
  niche_id: z.string().min(1, 'Niche is required'),
  target_count: z.number().min(1, 'Must be at least 1').max(100),
  params: z.record(z.unknown()).optional(),
});

type SpawnRunFormData = z.infer<typeof spawnRunSchema>;

interface SpawnRunDialogProps {
  niche: {
    id: string;
    name: string;
    vertical: string;
    demand_signals: Record<string, unknown>;
    competition_score: string;
  };
  onClose: () => void;
}

function SpawnRunDialog({ niche, onClose }: SpawnRunDialogProps) {
  const createRun = useCreateRun();
  
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<SpawnRunFormData>({
    resolver: zodResolver(spawnRunSchema),
    defaultValues: {
      niche_id: niche.id,
      target_count: 10,
      params: {},
    },
  });

  const onSubmit = async (data: SpawnRunFormData) => {
    try {
      await createRun.mutateAsync(data);
      reset();
      onClose();
    } catch (err) {
      console.error('Failed to spawn run:', err);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 overflow-y-auto">
      <div className="w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6">
        <div className="mb-4">
          <h2 className="text-xl font-semibold">Spawn Production Run</h2>
          <p className="text-gray-600 dark:text-gray-400 text-sm">{niche.name}</p>
          <p className="text-gray-500 dark:text-gray-400 text-sm">{niche.vertical}</p>
        </div>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Target Count</label>
            <input
              {...register('target_count', { valueAsNumber: true })}
              type="number"
              min="1"
              max="100"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
            />
            {errors.target_count && <p className="text-red-500 text-sm mt-1">{errors.target_count.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Generation Parameters (JSON)</label>
            <textarea
              {...register('params', )}
              rows={4}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 font-mono text-sm"
              placeholder='{"style": "gothic", "colorway": "dark", "count": 3}'
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
                  Spawning...
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

export function NicheBoardPage() {
  const [spawnDialog, setSpawnDialog] = useState<SpawnRunDialogProps['niche'] | null>(null);
  
  const { data, isLoading, error } = useNiches({ limit: 50, offset: 0 });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-red-500">Failed to load niches</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Niche Board</h1>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Ranked niches with demand signals — pick a niche to spawn a production run
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {data?.items.map((niche) => {
          const demandSignals = niche.demand_signals as Record<string, unknown> | undefined;
          const searchVolume = demandSignals?.search_volume as number | undefined;
          const trend = demandSignals?.trend as string | undefined;
          const competition = parseFloat(niche.competition_score);
          
          return (
            <Card key={niche.id} className="hover:shadow-lg transition-shadow cursor-pointer">
              <CardContent className="p-6">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <h3 className="font-semibold text-gray-900 dark:text-white text-lg">{niche.name}</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 capitalize">{niche.vertical}</p>
                  </div>
                  <span className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs font-mono text-gray-600 dark:text-gray-300">
                    #{niche.id.slice(0, 6)}
                  </span>
                </div>

                <div className="grid grid-cols-3 gap-4 mb-4">
                  <div className="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
                    <p className="text-xs text-gray-500 dark:text-gray-400">Search Volume</p>
                    <p className="font-bold text-gray-900 dark:text-white">
                      {searchVolume ? formatNumber(searchVolume) : '—'}
                    </p>
                  </div>
                  <div className="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
                    <p className="text-xs text-gray-500 dark:text-gray-400">Competition</p>
                    <p className="font-bold text-gray-900 dark:text-white">
                      {competition.toFixed(2)}
                    </p>
                  </div>
                  <div className="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
                    <p className="text-xs text-gray-500 dark:text-gray-400">Trend</p>
                    <p className="font-bold text-gray-900 dark:text-white flex items-center gap-1">
                      {trend === 'rising' && <TrendingUp className="h-3 w-3 text-green-500" />}
                      {trend === 'falling' && <TrendingDown className="h-3 w-3 text-red-500" />}
                      {trend === 'stable' && <Target className="h-3 w-3 text-gray-500" />}
                      <span className="capitalize">{trend ?? '—'}</span>
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setSpawnDialog(niche)}
                    className="flex-1 btn-primary justify-center"
                  >
                    <Play className="h-4 w-4 mr-2" />
                    Spawn Run
                  </button>
                  <button
                    className="btn-outline"
                    title="View niche details"
                  >
                    <Zap className="h-4 w-4" />
                  </button>
                </div>
              </CardContent>
            </Card>
          );
        })}
        {(!data?.items.length) && (
          <Card className="md:col-span-3">
            <CardContent className="p-12 text-center">
              <Target className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">No niches found</h3>
              <p className="text-gray-500 dark:text-gray-400 mt-1">
                Create niches in the Niches page to see them here.
              </p>
            </CardContent>
          </Card>
        )}
      </div>

      {spawnDialog && (
        <SpawnRunDialog
          niche={spawnDialog}
          onClose={() => setSpawnDialog(null)}
        />
      )}
    </div>
  );
}