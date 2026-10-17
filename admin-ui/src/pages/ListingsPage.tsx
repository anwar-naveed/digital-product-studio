// Listings page component
import { useListings } from '../hooks/useListings';
import { usePublishListing, useDelistListing, useRepriceListing } from '../hooks/useListings';
import { useProducts } from '../hooks/useProducts';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDateTime, formatCurrency } from '../lib/utils';
import { Send, X, DollarSign, Loader2, RefreshCw, Image as ImageIcon } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { assetUrl } from '../api/client';

const PLATFORMS = [
  { value: 'etsy', label: 'Etsy' },
  { value: 'lemon-squeezy', label: 'Lemon Squeezy' },
  { value: 'printify', label: 'Printify' },
  { value: 'gelato', label: 'Gelato' },
] as const;

const stateColors: Record<string, string> = {
  draft: 'bg-muted text-foreground dark:bg-gray-700 dark:text-gray-300',
  publishing: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
  live: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
  updating: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
  failed: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
  suspended: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400',
  delisting: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
  delisted: 'bg-muted text-foreground dark:bg-gray-700 dark:text-gray-400',
};

const publishSchema = z.object({
  platform: z.enum(['etsy', 'lemon-squeezy', 'printify', 'gelato']),
  license_tier: z.enum(['personal', 'small_commercial', 'extended_commercial']).optional(),
});

type PublishFormData = z.infer<typeof publishSchema>;

const delistSchema = z.object({
  platform: z.enum(['etsy', 'lemon-squeezy', 'printify', 'gelato']),
});

type DelistFormData = z.infer<typeof delistSchema>;

const repriceSchema = z.object({
  platform: z.enum(['etsy', 'lemon-squeezy', 'printify', 'gelato']),
  base_price: z.string().min(1, 'Base price is required'),
});

type RepriceFormData = z.infer<typeof repriceSchema>;

interface ActionDialogProps {
  type: 'publish' | 'delist' | 'reprice';
  productId: string;
  productTitle: string;
  listing?: {
    id: string;
    platform: string;
    state: string;
  };
  onClose: () => void;
}

function ActionDialog({ type, productId, productTitle, listing, onClose }: ActionDialogProps) {
  const publishListing = usePublishListing();
  const delistListing = useDelistListing();
  const repriceListing = useRepriceListing();
  
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<
    PublishFormData | DelistFormData | RepriceFormData
  >({
    resolver: type === 'publish' ? zodResolver(publishSchema) :
            type === 'delist' ? zodResolver(delistSchema) :
            zodResolver(repriceSchema),
    defaultValues: (type === 'publish' ? {
      platform: 'etsy',
    } : type === 'delist' ? {
      platform: listing?.platform ?? 'etsy',
    } : {
      platform: listing?.platform ?? 'etsy',
      base_price: '',
    }) as any,
  });

  const onSubmit = async (data: PublishFormData | DelistFormData | RepriceFormData) => {
    try {
      if (type === 'publish') {
        await publishListing.mutateAsync({ productId, data: data as PublishFormData });
      } else if (type === 'delist') {
        await delistListing.mutateAsync({ productId, data: data as DelistFormData });
      } else {
        await repriceListing.mutateAsync({ productId, data: data as RepriceFormData });
      }
      reset();
      onClose();
    } catch (err) {
      console.error(`Failed to ${type}:`, err);
    }
  };

  const titles = {
    publish: 'Publish to Platform',
    delist: 'Delist from Platform',
    reprice: 'Reprice Listing',
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 overflow-y-auto">
      <div className="w-full max-w-md bg-card rounded-lg shadow-xl p-6">
        <h2 className="text-xl font-semibold mb-2">{titles[type]}</h2>
        <p className="text-muted-foreground mb-4">{productTitle}</p>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Platform</label>
            <select
              {...register('platform')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
            >
              {PLATFORMS.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
          </div>

          {type === 'publish' && (
            <div>
              <label className="block text-sm font-medium mb-1">License Tier (optional)</label>
              <select
                {...register('license_tier')}
                className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              >
                <option value="">Use product default</option>
                <option value="personal">Personal</option>
                <option value="small_commercial">Small Commercial</option>
                <option value="extended_commercial">Extended Commercial</option>
              </select>
            </div>
          )}

          {type === 'reprice' && (
            <div>
              <label className="block text-sm font-medium mb-1">Base Price</label>
              <input
                {...register('base_price')}
                type="number"
                step="0.01"
                className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
                placeholder="29.99"
              />
              {(errors as any).base_price && <p className="text-destructive text-sm mt-1">{(errors as any).base_price.message}</p>}
            </div>
          )}

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
              disabled={isSubmitting || publishListing.isPending || delistListing.isPending || repriceListing.isPending}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90 disabled:opacity-50"
            >
              {(isSubmitting || publishListing.isPending || delistListing.isPending || repriceListing.isPending) ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : type === 'publish' ? (
                <>
                  <Send className="h-4 w-4 mr-2" />
                  Publish
                </>
              ) : type === 'delist' ? (
                <>
                  <X className="h-4 w-4 mr-2" />
                  Delist
                </>
              ) : (
                <>
                  <DollarSign className="h-4 w-4 mr-2" />
                  Reprice
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export function ListingsPage() {
  const [actionDialog, setActionDialog] = useState<{
    type: 'publish' | 'delist' | 'reprice';
    productId: string;
    productTitle: string;
    listing?: { id: string; platform: string; state: string };
  } | null>(null);
  
  const { data: productsData } = useProducts({ limit: 1000, offset: 0 });
  const { data, isLoading, error } = useListings();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-destructive">Failed to load listings</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Listings</h1>
        <p className="text-muted-foreground mt-1">Manage platform listings and pricing</p>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-muted">
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Product</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Platform</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">State</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Price</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Platform ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">Updated</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-muted-foreground uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {data?.items?.map((listing) => {
                  const product = productsData?.items.find((p) => p.id === listing.product_id);
                  return (
                    <tr key={listing.id} className="hover:bg-accent">
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-3">
                          {product?.cover_s3_key ? (
                            <img
                              src={assetUrl(product.cover_s3_key)}
                              alt={product.title}
                              className="w-12 h-12 rounded object-cover border border-border shrink-0"
                              loading="lazy"
                            />
                          ) : (
                            <div className="w-12 h-12 rounded bg-muted flex items-center justify-center shrink-0">
                              <ImageIcon className="h-5 w-5 text-muted-foreground" />
                            </div>
                          )}
                          <span className="font-medium text-foreground">
                            {product?.title ?? listing.product_id.slice(0, 8)}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4 capitalize text-muted-foreground">
                        {listing.platform.replace('-', ' ')}
                      </td>
                      <td className="px-6 py-4">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${stateColors[listing.state] ?? 'bg-muted text-foreground'}`}>
                          {listing.state}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-foreground">{formatCurrency(listing.price)}</td>
                      <td className="px-6 py-4 font-mono text-sm text-muted-foreground">
                        {listing.platform_listing_id || '—'}
                      </td>
                      <td className="px-6 py-4 text-muted-foreground">{formatDateTime(listing.updated_at)}</td>
                      <td className="px-6 py-4 text-right">
                        <div className="flex items-center justify-end gap-1">
                          {listing.state === 'draft' && (
                            <button
                              onClick={() => setActionDialog({ type: 'publish', productId: listing.product_id, productTitle: product?.title ?? '', listing })}
                              className="px-2 py-1 text-xs bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 rounded hover:bg-green-200"
                            >
                              <Send className="h-3 w-3 mr-1 inline" /> Publish
                            </button>
                          )}
                          {(listing.state === 'live' || listing.state === 'updating') && (
                            <button
                              onClick={() => setActionDialog({ type: 'delist', productId: listing.product_id, productTitle: product?.title ?? '', listing })}
                              className="px-2 py-1 text-xs bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 rounded hover:bg-red-200"
                            >
                              <X className="h-3 w-3 mr-1 inline" /> Delist
                            </button>
                          )}
                          {(listing.state === 'live' || listing.state === 'updating' || listing.state === 'failed') && (
                            <button
                              onClick={() => setActionDialog({ type: 'reprice', productId: listing.product_id, productTitle: product?.title ?? '', listing })}
                              className="px-2 py-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 rounded hover:bg-blue-200"
                            >
                              <DollarSign className="h-3 w-3 mr-1 inline" /> Reprice
                            </button>
                          )}
                          {(listing.state === 'failed') && (
                            <button
                              onClick={() => setActionDialog({ type: 'publish', productId: listing.product_id, productTitle: product?.title ?? '', listing })}
                              className="px-2 py-1 text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 rounded hover:bg-yellow-200"
                            >
                              <RefreshCw className="h-3 w-3 mr-1 inline" /> Retry
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  );
                })}
                {(!data?.items.length) && (
                  <tr>
                    <td colSpan={7} className="px-6 py-12 text-center">
                      <ImageIcon className="h-10 w-10 mx-auto text-muted-foreground mb-3" />
                      <p className="text-muted-foreground">No platform listings yet.</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        Add Etsy / Lemon Squeezy credentials in Settings → Credentials, then publish a product from the Final Products page.
                      </p>
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {actionDialog && (
        <ActionDialog
          {...actionDialog}
          onClose={() => setActionDialog(null)}
        />
      )}
    </div>
  );
}