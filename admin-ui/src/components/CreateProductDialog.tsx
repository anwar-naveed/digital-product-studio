'use client';

import { useState } from 'react';
import { useForm, useFieldArray } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Loader2, X, Plus, Trash2, GripVertical } from 'lucide-react';
//import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';

import { Input } from '@/components/ui/Input';
//import { Textarea } from '@/components/ui/Textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { api, type LicenseTier, type CreateProductRequest } from '@/api/client';

const CreateProductSchema = z.object({
  sku: z.string().min(1, 'SKU is required').max(100, 'SKU too long'),
  title: z.string().min(1, 'Title is required').max(200, 'Title too long'),
  license_tier: z.enum(['personal', 'small_commercial', 'extended_commercial']),
  price: z.string().min(1, 'Price is required').regex(/^\d+(\.\d{1,2})?$/, 'Invalid price format'),
  s3_prefix: z.string().min(1, 'S3 prefix is required'),
  items: z.array(z.object({
    variant_id: z.string().min(1, 'Variant ID is required'),
    position: z.coerce.number().int().min(0, 'Position must be >= 0'),
  })).min(1, 'At least one item is required'),
});

type CreateProductFormData = z.infer<typeof CreateProductSchema>;

interface CreateProductDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  product?: { id: string; sku: string; title: string; license_tier: string; price: string; s3_prefix: string } | null;
}

export function CreateProductDialog({ product: _product, open, onClose, onSuccess }: CreateProductDialogProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
    watch,
    reset,
    setValue,
  } = useForm<CreateProductFormData>({
    resolver: zodResolver(CreateProductSchema),
    defaultValues: {
      sku: '',
      title: '',
      license_tier: 'personal',
      price: '',
      s3_prefix: '',
      items: [{ variant_id: '', position: 0 }],
    },
  });

  const { fields, append, remove } = useFieldArray({ control, name: 'items' });

  const onSubmit = async (data: CreateProductFormData) => {
    setIsLoading(true);
    setError(null);
    try {
      const createRequest: CreateProductRequest = {
        sku: data.sku,
        title: data.title,
        license_tier: data.license_tier as LicenseTier,
        price: data.price,
        s3_prefix: data.s3_prefix,
        items: data.items.map(item => ({
          variant_id: item.variant_id,
          position: item.position,
        })),
      };
      await api.catalog.create(createRequest);
      reset();
      onClose();
      onSuccess?.();
    } catch (err: any) {
      setError(err.message ?? 'Failed to create product');
    } finally {
      setIsLoading(false);
    }
  };

  if (!open) return null;

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-start justify-between">
            <DialogTitle>Create New Product</DialogTitle>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
          {error && (
            <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
              {error}
            </div>
          )}

          {/* SKU */}
          <div className="space-y-2">
            <label htmlFor="sku" className="text-sm font-medium">
              SKU <span className="text-destructive">*</span>
            </label>
            <Input
              id="sku"
              {...register('sku')}
              placeholder="e.g., WALL-ART-001"
            />
            {errors.sku && <p className="text-sm text-destructive">{errors.sku.message}</p>}
          </div>

          {/* Title */}
          <div className="space-y-2">
            <label htmlFor="title" className="text-sm font-medium">
              Title <span className="text-destructive">*</span>
            </label>
            <Input
              id="title"
              {...register('title')}
              placeholder="e.g., Minimalist Mountain Landscape"
            />
            {errors.title && <p className="text-sm text-destructive">{errors.title.message}</p>}
          </div>

          {/* License Tier */}
          <div className="space-y-2">
            <label htmlFor="license_tier" className="text-sm font-medium">
              License Tier <span className="text-destructive">*</span>
            </label>
            <Select
              value={watch('license_tier')}
              onValueChange={(value) => setValue('license_tier', value as any)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select license tier" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="personal">Personal Use</SelectItem>
                <SelectItem value="small_commercial">Small Commercial (up to 500 units)</SelectItem>
                <SelectItem value="extended_commercial">Extended Commercial (unlimited)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Price */}
          <div className="space-y-2">
            <label htmlFor="price" className="text-sm font-medium">
              Price (USD) <span className="text-destructive">*</span>
            </label>
            <Input
              id="price"
              type="number"
              step="0.01"
              min="0.01"
              {...register('price')}
              placeholder="e.g., 19.99"
            />
            {errors.price && <p className="text-sm text-destructive">{errors.price.message}</p>}
          </div>

          {/* S3 Prefix */}
          <div className="space-y-2">
            <label htmlFor="s3_prefix" className="text-sm font-medium">
              S3 Prefix <span className="text-destructive">*</span>
            </label>
            <Input
              id="s3_prefix"
              {...register('s3_prefix')}
              placeholder="e.g., products/wall-art-001/"
            />
            {errors.s3_prefix && <p className="text-sm text-destructive">{errors.s3_prefix.message}</p>}
          </div>

          {/* Items */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Product Items <span className="text-destructive">*</span></label>
              <Button type="button" variant="outline" size="sm" onClick={() => append({ variant_id: '', position: fields.length })}>
                <Plus className="mr-2 h-4 w-4" />
                Add Item
              </Button>
            </div>

            {fields.map((field, index) => (
              <div key={field.id} className="flex items-start gap-2 p-3 border rounded-lg bg-muted/50">
                <GripVertical className="h-5 w-5 text-muted-foreground mt-1 cursor-grab" />
                <div className="flex-1 space-y-2">
                  <div className="grid gap-2 sm:grid-cols-2">
                    <div className="space-y-1">
                      <label className="text-xs font-medium text-muted-foreground">Variant ID</label>
                      <Input
                        {...register(`items.${index}.variant_id`)}
                        placeholder="Variant UUID"
                        className="text-sm"
                      />
                    </div>
                    <div className="space-y-1">
                      <label className="text-xs font-medium text-muted-foreground">Position</label>
                      <Input
                        type="number"
                        min="0"
                        {...register(`items.${index}.position`)}
                        placeholder="0"
                        className="text-sm"
                      />
                    </div>
                  </div>
                  {errors.items && errors.items[index] && (
                    <p className="text-xs text-destructive">
                      {JSON.stringify(errors.items[index])}
                    </p>
                  )}
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => remove(index)}
                  disabled={fields.length === 1}
                >
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </div>
            ))}

            {errors.items && <p className="text-sm text-destructive">{errors.items.message}</p>}
          </div>

          <DialogFooter className="gap-2 border-t pt-4">
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading || isSubmitting}>
              {isLoading || isSubmitting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                'Create Product'
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}