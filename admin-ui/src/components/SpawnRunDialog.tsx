'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Loader2, Target } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Textarea } from '@/components/ui/Textarea';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogTrigger } from '@/components/ui/Dialog';

import { api, type NicheResponse, type NicheListResponse, type CreateRunRequest } from '@/api/client';

const SpawnRunSchema = z.object({
  niche_id: z.string().min(1, 'Niche is required'),
  target_count: z.coerce.number().int().min(1, 'Target count must be at least 1').max(1000, 'Target count too high'),
  params: z.string().optional().refine(
    (val) => {
      if (!val || val.trim() === '') return true;
      try {
        JSON.parse(val);
        return true;
      } catch {
        return false;
      }
    },
    { message: 'Params must be valid JSON' }
  ),
});

type SpawnRunFormData = z.infer<typeof SpawnRunSchema>;

interface SpawnRunDialogProps {
  onSuccess?: () => void;
  defaultNicheId?: string;
}

export function SpawnRunDialog({ onSuccess, defaultNicheId }: SpawnRunDialogProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [niches, setNiches] = useState<NicheResponse[]>([]);
  const [isLoadingNiches, setIsLoadingNiches] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { errors, isSubmitting },
    reset,
  } = useForm<SpawnRunFormData>({
    resolver: zodResolver(SpawnRunSchema),
    defaultValues: {
      niche_id: defaultNicheId || '',
      target_count: 10,
      params: '{}',
    },
  });

  const loadNiches = async () => {
    setIsLoadingNiches(true);
    try {
      const response: NicheListResponse = await api.niches.list({ limit: 100, offset: 0 });
      setNiches(response.items);
    } catch (err) {
      console.error('Failed to load niches:', err);
    } finally {
      setIsLoadingNiches(false);
    }
  };

  const onSubmit = async (data: SpawnRunFormData) => {
    setIsLoading(true);
    setError(null);
    try {
      const createRunRequest: CreateRunRequest = {
        niche_id: data.niche_id,
        target_count: data.target_count,
        params: data.params ? JSON.parse(data.params) : {},
      };
      await api.runs.create(createRunRequest);
      reset();
      setIsOpen(false);
      onSuccess?.();
    } catch (err: any) {
      setError(err.message ?? 'Failed to spawn run');
    } finally {
      setIsLoading(false);
    }
  };

  const watchedNicheId = watch('niche_id');

  return (
    <>
      <DialogTrigger asChild>
        <Button onClick={() => { loadNiches(); setIsOpen(true); }}>
          <Target className="mr-2 h-4 w-4" />
          Spawn Run
        </Button>
      </DialogTrigger>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Spawn New Production Run</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
            {error && (
              <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
                {error}
              </div>
            )}

            {/* Niche Selection */}
            <div className="space-y-2">
              <label htmlFor="niche_id" className="text-sm font-medium">
                Niche <span className="text-destructive">*</span>
              </label>
              <Select
                value={watchedNicheId}
                onValueChange={(value) => setValue('niche_id', value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a niche" />
                </SelectTrigger>
                <SelectContent>
                  {isLoadingNiches ? (
                    <SelectItem value="" disabled>
                      Loading niches...
                    </SelectItem>
                  ) : niches.length === 0 ? (
                    <SelectItem value="" disabled>
                      No niches available
                    </SelectItem>
                  ) : (
                    niches.map((niche) => (
                      <SelectItem key={niche.id} value={niche.id}>
                        {niche.name} ({niche.vertical})
                      </SelectItem>
                    ))
                  )}
                </SelectContent>
              </Select>
              {errors.niche_id && (
                <p className="text-sm text-destructive">{errors.niche_id.message}</p>
              )}
            </div>

            {/* Target Count */}
            <div className="space-y-2">
              <label htmlFor="target_count" className="text-sm font-medium">
                Target Count <span className="text-destructive">*</span>
              </label>
              <Input
                id="target_count"
                type="number"
                min="1"
                max="1000"
                {...register('target_count')}
                placeholder="e.g., 10"
              />
              {errors.target_count && (
                <p className="text-sm text-destructive">{errors.target_count.message}</p>
              )}
            </div>

            {/* Params JSON */}
            <div className="space-y-2">
              <label htmlFor="params" className="text-sm font-medium">
                Parameters (JSON)
                <span className="text-muted-foreground ml-1">(optional)</span>
              </label>
              <Textarea
                id="params"
                rows={4}
                {...register('params')}
                placeholder='{"style": "minimalist", "color_palette": "monochrome"}'
                className="font-mono text-sm"
              />
              {errors.params && (
                <p className="text-sm text-destructive">{errors.params.message}</p>
              )}
              <p className="text-xs text-muted-foreground">
                Optional JSON parameters passed to the generation tools.
              </p>
            </div>

            {/* Preview of selected niche */}
            {watchedNicheId && (
              <Card variant="outline" className="bg-muted/50">
                <CardContent className="pt-4">
                  <p className="text-sm font-medium">Selected Niche Details</p>
                  <div className="mt-2 grid gap-1 text-sm">
                    {niches.find(n => n.id === watchedNicheId) && niches.find(n => n.id === watchedNicheId)!.demand_signals && (
                      <>
                        <p className="text-muted-foreground">
                          Competition: {niches.find(n => n.id === watchedNicheId)!.competition_score}
                        </p>
                        <p className="text-muted-foreground">
                          Demand signals: {Object.keys(niches.find(n => n.id === watchedNicheId)!.demand_signals).length} metrics
                        </p>
                      </>
                    )}
                  </div>
                </CardContent>
              </Card>
            )}

            <DialogFooter className="gap-2">
              <Button type="button" variant="outline" onClick={() => { reset(); setIsOpen(false); }}>
                Cancel
              </Button>
              <Button type="submit" disabled={isLoading || isSubmitting}>
                {isLoading || isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Creating...
                  </>
                ) : (
                  <>
                    <Target className="mr-2 h-4 w-4" />
                    Spawn Run
                  </>
                )}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}