'use client';

import { useState, useEffect, useMemo } from 'react';
import {
  Loader2, X, Package, Layers, Eye, FileText, Tag, Download, Image as ImageIcon, Store, Globe,
} from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/Dialog';
import { api, assetUrl, type ProductResponse, type ProductDetailResponse, type LicenseTier } from '@/api/client';

const licenseVariants: Record<LicenseTier | string, string> = {
  personal: 'info',
  small_commercial: 'warning',
  extended_commercial: 'success',
};

const formatLicense = (license: string) => {
  return license.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
};

const PRINT_RATIOS = ['2x3', '3x4', '4x5', '11x14', 'iso-a'];

type Tab = 'overview' | 'prints' | 'package' | 'publish';

interface ProductDetailDialogProps {
  product: ProductResponse;
  onClose: () => void;
  onEdit?: (product: ProductResponse) => void;
}

export function ProductDetailDialog({ product, onClose, onEdit }: ProductDetailDialogProps) {
  const [detail, setDetail] = useState<ProductDetailResponse | null>(null);
  const [tab, setTab] = useState<Tab>('prints');
  const [zoom, setZoom] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setIsLoading(true);
      try {
        const data = await api.catalog.get(product.id);
        if (!cancelled) setDetail(data);
      } catch (err) {
        if (!cancelled) setError('Failed to load product details');
        console.error(err);
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [product.id]);

  const meta = detail?.product.listing_metadata ?? null;
  const etsy = meta?.etsy ?? null;
  const lsq = meta?.lemonsqueezy ?? null;
  const items = useMemo(() => detail?.items ?? [], [detail]);

  const bundleUrl = (designId: string) => assetUrl(`products/${designId}/bundle.zip`);

  const tabs: { id: Tab; label: string; icon: import('react').ReactNode }[] = [
    { id: 'prints', label: `Prints (${items.length})`, icon: <ImageIcon className="h-4 w-4" /> },
    { id: 'package', label: 'Package', icon: <Package className="h-4 w-4" /> },
    { id: 'publish', label: 'Publish Preview', icon: <Globe className="h-4 w-4" /> },
    { id: 'overview', label: 'Overview', icon: <FileText className="h-4 w-4" /> },
  ];

  return (
    <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-5xl max-h-[92vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <DialogTitle className="pr-8">{product.title}</DialogTitle>
              <div className="flex items-center gap-2 mt-2 flex-wrap">
                <Badge variant="outline">{product.sku}</Badge>
                <Badge variant={licenseVariants[product.license_tier] || 'outline'}>
                  {formatLicense(product.license_tier)}
                </Badge>
                <span className="text-lg font-bold text-foreground">${product.price}</span>
              </div>
            </div>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        </DialogHeader>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center py-12 text-center text-destructive">
            {error}
          </div>
        ) : detail ? (
          <div className="space-y-4 py-2">
            {/* Tab bar */}
            <div className="flex gap-1 border-b border-border overflow-x-auto">
              {tabs.map((t) => (
                <button
                  key={t.id}
                  onClick={() => setTab(t.id)}
                  className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 -mb-px whitespace-nowrap transition-colors ${
                    tab === t.id
                      ? 'border-primary text-foreground'
                      : 'border-transparent text-muted-foreground hover:text-foreground'
                  }`}
                >
                  {t.icon}
                  {t.label}
                </button>
              ))}
            </div>

            {/* PRINTS TAB — the actual final artwork */}
            {tab === 'prints' && (
              <div className="grid gap-4 sm:grid-cols-3">
                {items.map((item, idx) => (
                  <Card key={item.id} className="overflow-hidden">
                    <div
                      className="aspect-[3/4] bg-muted cursor-zoom-in relative"
                      onClick={() => item.s3_key && setZoom(item.s3_key)}
                    >
                      {item.s3_key ? (
                        <img src={assetUrl(item.s3_key)} alt={`Print ${idx + 1}`} className="w-full h-full object-cover" loading="lazy" />
                      ) : (
                        <div className="w-full h-full flex items-center justify-center text-muted-foreground text-sm">No preview</div>
                      )}
                      <span className="absolute top-2 left-2 bg-black/60 text-white text-xs px-2 py-0.5 rounded">PRINT {idx + 1}</span>
                    </div>
                    <CardContent className="p-3 space-y-1.5">
                      <p className="text-xs text-muted-foreground font-mono truncate">{item.variant_id.slice(0, 16)}…</p>
                      {item.design_id && (
                        <a
                          href={bundleUrl(item.design_id)}
                          className="text-xs text-primary hover:underline flex items-center gap-1"
                          download
                        >
                          <Download className="h-3 w-3" /> bundle.zip
                        </a>
                      )}
                    </CardContent>
                  </Card>
                ))}
                {items.length === 0 && (
                  <p className="col-span-3 text-center text-muted-foreground py-12">
                    No prints linked to this product yet.
                  </p>
                )}
              </div>
            )}

            {/* PACKAGE TAB — exactly what the buyer receives */}
            {tab === 'package' && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Package className="h-4 w-4" /> What the buyer gets
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <p className="text-sm text-muted-foreground">
                    {items.length} print{items.length === 1 ? '' : 's'}, each rendered in 5 industry-standard
                    ratios at 300 DPI (JPEG) — sized for every common frame.
                  </p>
                  {items.map((item, idx) => (
                    <div key={item.id} className="border rounded-lg p-3 bg-muted/30">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium text-foreground">print-{idx + 1}/</span>
                        {item.design_id && (
                          <a href={bundleUrl(item.design_id)} download className="text-xs flex items-center gap-1 text-primary hover:underline">
                            <Download className="h-3 w-3" /> Download bundle ({idx + 1})
                          </a>
                        )}
                      </div>
                      <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5 text-xs font-mono">
                        {PRINT_RATIOS.map((r) => (
                          <span key={r} className="px-2 py-1 bg-muted rounded text-foreground">
                            {item.variant_id.slice(0, 8)}…/{r}.jpg
                          </span>
                        ))}
                      </div>
                      <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5 text-xs font-mono mt-1.5">
                        <span className="px-2 py-1 bg-muted rounded text-muted-foreground">manifest.json</span>
                        <span className="px-2 py-1 bg-muted rounded text-muted-foreground">README.txt</span>
                        <span className="px-2 py-1 bg-muted rounded text-muted-foreground">LICENSE.txt</span>
                      </div>
                    </div>
                  ))}
                  <div className="text-xs text-muted-foreground border-t pt-3 space-y-0.5">
                    <p><strong className="text-foreground">2:3</strong> — 12x18, 16x24, 20x30, 24x36 in</p>
                    <p><strong className="text-foreground">3:4</strong> — 12x16, 18x24, 30x40 in</p>
                    <p><strong className="text-foreground">4:5</strong> — 8x10, 16x20, 24x30 in</p>
                    <p><strong className="text-foreground">11x14</strong> — 11x14 in</p>
                    <p><strong className="text-foreground">ISO A</strong> — A2/A3/A4/A5</p>
                  </div>
                </CardContent>
              </Card>
            )}

            {/* PUBLISH PREVIEW TAB — what goes live on each platform */}
            {tab === 'publish' && (
              <div className="space-y-4">
                {!meta && (
                  <p className="text-sm text-muted-foreground text-center py-8">
                    No listing metadata stored for this product yet.
                  </p>
                )}
                {meta && etsy && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2 text-base">
                        <Store className="h-4 w-4" /> Etsy listing (preview)
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div>
                        <p className="text-xs uppercase tracking-wide text-muted-foreground">Title</p>
                        <p className="text-sm font-medium text-foreground">{etsy.title}</p>
                      </div>
                      <div className="flex items-center gap-3">
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground">Price</p>
                          <p className="text-lg font-bold text-foreground">${etsy.price_usd?.toFixed(2)}</p>
                        </div>
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground">Type</p>
                          <p className="text-sm text-foreground">{etsy.type ?? 'digital_download'}</p>
                        </div>
                      </div>
                      {etsy.tags && etsy.tags.length > 0 && (
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1.5">
                            Tags ({etsy.tags.length}/13)
                          </p>
                          <div className="flex flex-wrap gap-1.5">
                            {etsy.tags.map((t) => (
                              <Badge key={t} variant="secondary" className="text-xs">
                                <Tag className="h-3 w-3 mr-1" />
                                {t}
                              </Badge>
                            ))}
                          </div>
                        </div>
                      )}
                      {etsy.description && (
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1">Description</p>
                          <pre className="text-xs text-foreground whitespace-pre-wrap font-sans bg-muted/40 rounded p-3 max-h-64 overflow-y-auto">
                            {etsy.description}
                          </pre>
                        </div>
                      )}
                    </CardContent>
                  </Card>
                )}
                {meta && lsq && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2 text-base">
                        <Layers className="h-4 w-4" /> Lemon Squeezy product (preview)
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div>
                        <p className="text-xs uppercase tracking-wide text-muted-foreground">Name</p>
                        <p className="text-sm font-medium text-foreground">{lsq.name}</p>
                      </div>
                      <div className="flex items-center gap-6">
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground">Price</p>
                          <p className="text-lg font-bold text-foreground">${lsq.price_usd?.toFixed(2)}</p>
                        </div>
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground">License</p>
                          <p className="text-sm text-foreground">{lsq.license ?? 'personal_use'}</p>
                        </div>
                      </div>
                      {lsq.description && (
                        <div>
                          <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1">Description</p>
                          <pre className="text-xs text-foreground whitespace-pre-wrap font-sans bg-muted/40 rounded p-3 max-h-48 overflow-y-auto">
                            {lsq.description}
                          </pre>
                        </div>
                      )}
                    </CardContent>
                  </Card>
                )}
                <p className="text-xs text-muted-foreground text-center">
                  Publishing goes live once platform credentials are added in Settings → Credentials.
                </p>
              </div>
            )}

            {/* OVERVIEW TAB */}
            {tab === 'overview' && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <FileText className="h-4 w-4" /> Product information
                  </CardTitle>
                </CardHeader>
                <CardContent className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-1">
                    <p className="text-sm text-muted-foreground">Product ID</p>
                    <p className="font-mono text-sm">{detail.product.id}</p>
                  </div>
                  <div className="space-y-1">
                    <p className="text-sm text-muted-foreground">SKU</p>
                    <p className="font-medium">{detail.product.sku}</p>
                  </div>
                  <div className="space-y-1">
                    <p className="text-sm text-muted-foreground">License Tier</p>
                    <Badge variant={licenseVariants[detail.product.license_tier] || 'outline'}>
                      {formatLicense(detail.product.license_tier)}
                    </Badge>
                  </div>
                  <div className="space-y-1">
                    <p className="text-sm text-muted-foreground">Price</p>
                    <p className="font-medium">${detail.product.price}</p>
                  </div>
                  {items[0]?.run_id && (
                    <div className="space-y-1">
                      <p className="text-sm text-muted-foreground">Origin Run</p>
                      <p className="font-mono text-xs">{items[0].run_id}</p>
                    </div>
                  )}
                  {items[0]?.niche_id && (
                    <div className="space-y-1">
                      <p className="text-sm text-muted-foreground">Origin Niche</p>
                      <p className="font-mono text-xs">{items[0].niche_id}</p>
                    </div>
                  )}
                  <div className="space-y-1 sm:col-span-2">
                    <p className="text-sm text-muted-foreground">Created</p>
                    <p className="font-medium">{new Date(detail.product.created_at).toLocaleString()}</p>
                  </div>
                </CardContent>
              </Card>
            )}
          </div>
        ) : null}

        {/* Footer actions */}
        <div className="flex justify-end gap-2 border-t pt-4">
          <Button variant="outline" onClick={onClose}>Close</Button>
          {onEdit && (
            <Button variant="outline" onClick={() => onEdit(product)}>
              <Eye className="mr-2 h-4 w-4" /> Edit Product
            </Button>
          )}
        </div>
      </DialogContent>

      {/* Zoom overlay */}
      {zoom && (
        <div
          className="fixed inset-0 z-[60] bg-black/80 flex items-center justify-center p-6 cursor-zoom-out"
          onClick={() => setZoom(null)}
        >
          <img src={assetUrl(zoom)} alt="Zoomed print" className="max-h-[88vh] max-w-full object-contain" />
        </div>
      )}
    </Dialog>
  );
}
