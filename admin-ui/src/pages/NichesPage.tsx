// Niches page component
import { useState } from 'react';
import { useNiches } from '../hooks/useNiches';
import { useCreateNiche, useUpdateNiche, useDeleteNiche } from '../hooks/useNiches';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDate } from '../lib/utils';
import { Plus, Edit, Trash2, Loader2 } from 'lucide-react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const nicheSchema = z.object({
  name: z.string().min(1, 'Name is required').max(200),
  vertical: z.string().min(1, 'Vertical is required').max(100),
  demand_signals: z.record(z.unknown()).optional(),
  competition_score: z.string().optional(),
});

type NicheFormData = z.infer<typeof nicheSchema>;

interface NicheDialogProps {
  niche?: {
    id: string;
    name: string;
    vertical: string;
    demand_signals: Record<string, unknown>;
    competition_score: string;
  } | null;
  onClose: () => void;
}

function NicheDialog({ niche, onClose }: NicheDialogProps) {
  const isEditing = !!niche;
  const createNiche = useCreateNiche();
  const updateNiche = useUpdateNiche();
  
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<NicheFormData>({
    resolver: zodResolver(nicheSchema),
    defaultValues: niche ? {
      name: niche.name,
      vertical: niche.vertical,
      demand_signals: niche.demand_signals as Record<string, unknown>,
      competition_score: niche.competition_score,
    } : {
      name: '',
      vertical: '',
      demand_signals: {},
      competition_score: '',
    },
  });

  const onSubmit = async (data: NicheFormData) => {
    try {
      if (isEditing && niche) {
        await updateNiche.mutateAsync({ id: niche.id, data });
      } else {
        await createNiche.mutateAsync(data);
      }
      reset();
      onClose();
    } catch (err) {
      console.error('Failed to save niche:', err);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-md bg-card rounded-lg shadow-xl p-6">
        <h2 className="text-xl font-semibold mb-4">{isEditing ? 'Edit Niche' : 'Create Niche'}</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Name</label>
            <input
              {...register('name')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="Niche name"
            />
            {errors.name && <p className="text-destructive text-sm mt-1">{errors.name.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Vertical</label>
            <select
              {...register('vertical')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
            >
              <option value="">Select a vertical…</option>
              <option value="printable_wall_art">Printable Wall Art</option>
              <option value="planner_journal_bundles">Planner / Journal Bundles</option>
              <option value="party_wedding_printables">Party / Wedding Printables</option>
            </select>
            {errors.vertical && <p className="text-destructive text-sm mt-1">{errors.vertical.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Competition Score</label>
            <input
              {...register('competition_score')}
              type="number"
              step="0.01"
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="0.0 - 1.0"
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
              disabled={isSubmitting || createNiche.isPending || updateNiche.isPending}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90 disabled:opacity-50"
            >
              {isSubmitting || createNiche.isPending || updateNiche.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                'Save'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export function NichesPage() {
  const [showDialog, setShowDialog] = useState(false);
  const [editingNiche, setEditingNiche] = useState<NicheDialogProps['niche'] | null>(null);
  const deleteNiche = useDeleteNiche();
  
  const { data, isLoading, error } = useNiches();

  const handleCreate = () => {
    setEditingNiche(null);
    setShowDialog(true);
  };

  const handleEdit = (niche: NicheDialogProps['niche']) => {
    setEditingNiche(niche);
    setShowDialog(true);
  };

  const handleDelete = async (id: string) => {
    if (window.confirm('Are you sure you want to delete this niche?')) {
      await deleteNiche.mutateAsync(id);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-destructive">Failed to load niches</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Niches</h1>
          <p className="text-muted-foreground mt-1">Manage niche verticals for production runs</p>
        </div>
        <button onClick={handleCreate} className="btn-primary">
          <Plus className="h-4 w-4 mr-2" /> Add Niche
        </button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-muted">
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Name</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Vertical</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Competition</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Created</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-muted-foreground uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {data?.items.map((niche) => (
                  <tr key={niche.id} className="hover:bg-accent">
                    <td className="px-6 py-4 font-medium text-foreground">{niche.name}</td>
                    <td className="px-6 py-4 text-muted-foreground">{niche.vertical}</td>
                    <td className="px-6 py-4 text-muted-foreground">{niche.competition_score}</td>
                    <td className="px-6 py-4 text-muted-foreground">{formatDate(niche.created_at)}</td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleEdit(niche)}
                          className="p-2 text-gray-500 hover:text-primary hover:bg-accent rounded-md"
                        >
                          <Edit className="h-4 w-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(niche.id)}
                          className="p-2 text-gray-500 hover:text-destructive hover:bg-accent rounded-md"
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
                {(!data?.items.length) && (
                  <tr>
                    <td colSpan={5} className="px-6 py-12 text-center text-muted-foreground">
                      No niches found. Create your first niche to get started.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {showDialog && (
        <NicheDialog
          niche={editingNiche}
          onClose={() => {
            setShowDialog(false);
            setEditingNiche(null);
          }}
        />
      )}
    </div>
  );
}