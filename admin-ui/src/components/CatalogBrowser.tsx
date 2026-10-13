import * as React from 'react';
'use client';

import { useState, useCallback, useEffect } from 'react';
import { Loader2, RefreshCw, Search, Plus, Eye, Edit, Package } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';

import { api, type ProductResponse, type ProductListResponse, type ListProductsQuery, type LicenseTier } from '@/api/client';
import { ProductDetailDialog } from './ProductDetailDialog';
import { CreateProductDialog } from './CreateProductDialog';

const licenseVariants: Record<string, string> = {
  personal: 'info',
  small_commercial: 'warning',
  extended_commercial: 'success',
};

interface CatalogBrowserProps {
  initialQuery?: ListProductsQuery;
}

export function CatalogBrowser({ initialQuery }: CatalogBrowserProps) {
  const [query, setQuery] = useState<ListProductsQuery>({
    limit: 20,
    offset: 0,
    ...initialQuery,
  });
  const [products, setProducts] = useState<ProductResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedProduct, setSelectedProduct] = useState<ProductResponse | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const fetchProducts = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: ProductListResponse = await api.catalog.list(query);
      setProducts(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch products:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  const handleFilterChange = (newQuery: Partial<ListProductsQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  const handleCreateSuccess = () => {
    setShowCreateDialog(false);
    fetchProducts();
  };

  const formatLicense = (license: string) => {
    return license.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <Package className="h-5 w-5" />
          <CardTitle>Catalog Browser</CardTitle>
        </div>
        <div className="flex items-center gap-4">
          <Select
            value={query.license_tier ?? ''}
            onValueChange={(value) => handleFilterChange({ license_tier: value as LicenseTier || undefined })}
          >
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="All license tiers" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Tiers</SelectItem>
              <SelectItem value="personal">Personal</SelectItem>
              <SelectItem value="small_commercial">Small Commercial</SelectItem>
              <SelectItem value="extended_commercial">Extended Commercial</SelectItem>
            </SelectContent>
          </Select>
          <Input
            placeholder="Search SKU or title..."
            value={query.limit ? '' : ''}
            onChange={(_e) => handleFilterChange({ limit: 20, offset: 0 })}
            className="w-[250px]"
            leftIcon={<Search className="h-4 w-4" />}
          />
          <Button
            variant="outline"
            size="icon"
            onClick={fetchProducts}
            disabled={isLoading}
            aria-label="Refresh products"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
          <Button onClick={() => setShowCreateDialog(true)}>
            <Plus className="mr-2 h-4 w-4" />
            New Product
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && products.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : products.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <Package className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground">No products found</p>
            <p className="text-sm text-muted-foreground">
              Get started by creating your first product
            </p>
          </div>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {products.map((product) => (
                <Card key={product.id} className="hover:shadow-md transition-shadow cursor-pointer">
                  <CardContent className="p-4">
                    <div className="space-y-3">
                      <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                          <p className="font-medium truncate">{product.title}</p>
                          <p className="text-sm text-muted-foreground font-mono">
                            SKU: {product.sku}
                          </p>
                        </div>
                        <Badge variant={licenseVariants[product.license_tier] || 'outline'}>
                          {formatLicense(product.license_tier)}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between text-sm text-muted-foreground">
                        <span>Price: <span className="font-medium text-foreground">${product.price}</span></span>
                        <span>Created: {new Date(product.created_at).toLocaleDateString()}</span>
                      </div>
                      <div className="flex gap-2 pt-2 border-t">
                        <Button
                          variant="ghost"
                          size="sm"
                          className="flex-1"
                          onClick={() => setSelectedProduct(product)}
                        >
                          <Eye className="mr-1 h-3 w-3" />
                          View
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          className="flex-1"
                          onClick={() => setSelectedProduct(product)}
                        >
                          <Edit className="mr-1 h-3 w-3" />
                          Edit
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Pagination */}
            {total > query.limit && (
              <div className="flex items-center justify-between mt-6 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  Showing {query.offset + 1} to {Math.min(query.offset + products.length, total)} of {total}
                </p>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(Math.max(0, query.offset - query.limit))}
                    disabled={query.offset === 0}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(query.offset + query.limit)}
                    disabled={query.offset + products.length >= total}
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}

            {/* Product Detail Dialog */}
            {selectedProduct && (
              <ProductDetailDialog
                product={selectedProduct}
                onClose={() => setSelectedProduct(null)}
              />
            )}

            {/* Create Product Dialog */}
            <CreateProductDialog
              open={showCreateDialog}
              onClose={() => setShowCreateDialog(false)}
              onSuccess={handleCreateSuccess}
            />
          </>
        )}
      </CardContent>
    </Card>
  );
}

// Helper component for Input with left icon
interface InputWithIconProps extends React.InputHTMLAttributes<HTMLInputElement> {
  leftIcon?: React.ReactNode;
}

const InputWithIcon = React.forwardRef<HTMLInputElement, InputWithIconProps>(
  ({ className, leftIcon, ...props }, ref) => {
    return (
      <div className="relative">
        {leftIcon && (
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
            {leftIcon}
          </div>
        )}
        <input
          ref={ref}
          className={cn(
            'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
            leftIcon && 'pl-10',
            className
          )}
          {...props}
        />
      </div>
    );
  }
);
InputWithIcon.displayName = 'InputWithIcon';