'use client';

import { useState, useEffect } from 'react';



import { Loader2, X, Package, Layers, Eye, Edit } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';



import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { api, type ProductResponse, type ProductDetailResponse, type LicenseTier } from '@/api/client';

const licenseVariants: Record<LicenseTier | string, string> = {
  personal: 'info',
  small_commercial: 'warning',
  extended_commercial: 'success',
};

const formatLicense = (license: string) => {
  return license.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase());
};

interface ProductDetailDialogProps {
  product: ProductResponse;
  onClose: () => void;
}

export function ProductDetailDialog({ product, onClose }: ProductDetailDialogProps) {
  const [detail, setDetail] = useState<ProductDetailResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchDetail = async () => {
    setIsLoading(true);
    try {
      const data = await api.catalog.get(product.id);
      setDetail(data);
    } catch (err) {
      setError('Failed to load product details');
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDetail();
  }, [product.id]);

  // We use a Dialog as a drawer-like overlay
  return (
    <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-start justify-between">
            <DialogTitle>{product.title}</DialogTitle>
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
          <div className="space-y-6 py-4">
            {/* Product Info */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Package className="h-5 w-5" />
                  Product Information
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
                <div className="space-y-1 sm:col-span-2">
                  <p className="text-sm text-muted-foreground">S3 Prefix</p>
                  <p className="font-mono text-sm text-muted-foreground">{detail.product.s3_prefix}</p>
                </div>
                <div className="space-y-1 sm:col-span-2">
                  <p className="text-sm text-muted-foreground">Created</p>
                  <p className="font-medium">{new Date(detail.product.created_at).toLocaleString()}</p>
                </div>
              </CardContent>
            </Card>

            {/* Items */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle className="flex items-center gap-2">
                  <Layers className="h-5 w-5" />
                  Product Items ({detail.items?.length ?? 0})
                </CardTitle>
              </CardHeader>
              <CardContent>
                {detail.items && detail.items.length > 0 ? (
                  <div className="space-y-2">
                    {detail.items.map((item, index) => (
                      <div key={item.id} className="flex items-center justify-between p-3 border rounded-lg bg-muted/50">
                        <div className="flex items-center gap-3">
                          <span className="text-sm font-medium text-muted-foreground">#{index + 1}</span>
                          <code className="text-xs bg-muted px-2 py-1 rounded font-mono">
                            {item.variant_id.slice(0, 12)}...
                          </code>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-sm text-muted-foreground">Position: {item.position}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-muted-foreground text-center py-8">No items in this product</p>
                )}
              </CardContent>
            </Card>

            {/* Actions */}
            <DialogFooter className="gap-2 border-t pt-4">
              <Button variant="outline" onClick={onClose}>
                Close
              </Button>
              <Button variant="outline" onClick={() => window.location.href = `/catalog/products/${product.id}/publish`}>
                <Eye className="mr-2 h-4 w-4" />
                Publish
              </Button>
              <Button variant="default" onClick={() => { /* Edit handler */ }}>
                <Edit className="mr-2 h-4 w-4" />
                Edit Product
              </Button>
            </DialogFooter>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}