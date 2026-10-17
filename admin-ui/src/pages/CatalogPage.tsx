// Catalog page component
import { useProducts } from '../hooks/useProducts';
import { useCreateProduct, useUpdateProduct, useDeleteProduct } from '../hooks/useProducts';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDate, formatCurrency } from '../lib/utils';
import { Plus, Package, Loader2, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { assetUrl } from '../api/client';
import { ProductDetailDialog } from '../components/ProductDetailDialog';

const LICENSE_TIERS = [
  { value: 'personal', label: 'Personal ($5-10)' },
  { value: 'small_commercial', label: 'Small Commercial ($25-50)' },
  { value: 'extended_commercial', label: 'Extended Commercial ($75-150)' },
] as const;

const productSchema = z.object({
  sku: z.string().min(1, 'SKU is required').max(100),
  title: z.string().min(1, 'Title is required').max(200),
  license_tier: z.enum(['personal', 'small_commercial', 'extended_commercial']),
  price: z.string().min(1, 'Price is required'),
  s3_prefix: z.string().min(1, 'S3 prefix is required'),
});

type ProductFormData = z.infer<typeof productSchema>;

interface ProductDialogProps {
  product?: {
    id: string;
    sku: string;
    title: string;
    license_tier: string;
    price: string;
    s3_prefix: string;
  } | null;
  onClose: () => void;
}

function ProductDialog({ product, onClose }: ProductDialogProps) {
  const isEditing = !!product;
  const createProduct = useCreateProduct();
  const updateProduct = useUpdateProduct();
  
  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<ProductFormData>({
    resolver: zodResolver(productSchema),
    defaultValues: product ? {
      sku: product.sku,
      title: product.title,
      license_tier: product.license_tier as 'personal' | 'small_commercial' | 'extended_commercial',
      price: product.price,
      s3_prefix: product.s3_prefix,
    } : {
      sku: '',
      title: '',
      license_tier: 'personal',
      price: '',
      s3_prefix: '',
    },
  });

  const onSubmit = async (data: ProductFormData) => {
    try {
      if (isEditing && product) {
        await updateProduct.mutateAsync({ id: product.id, data });
      } else {
        await createProduct.mutateAsync(data);
      }
      reset();
      onClose();
    } catch (err) {
      console.error('Failed to save product:', err);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 overflow-y-auto">
      <div className="w-full max-w-md bg-card rounded-lg shadow-xl p-6">
        <h2 className="text-xl font-semibold mb-4">{isEditing ? 'Edit Product' : 'Create Product'}</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">SKU</label>
            <input
              {...register('sku')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="PROD-001"
            />
            {errors.sku && <p className="text-destructive text-sm mt-1">{errors.sku.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Title</label>
            <input
              {...register('title')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="Product title"
            />
            {errors.title && <p className="text-destructive text-sm mt-1">{errors.title.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">License Tier</label>
            <select
              {...register('license_tier')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
            >
              {LICENSE_TIERS.map((tier) => (
                <option key={tier.value} value={tier.value}>{tier.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Price</label>
            <input
              {...register('price')}
              type="number"
              step="0.01"
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="29.99"
            />
            {errors.price && <p className="text-destructive text-sm mt-1">{errors.price.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">S3 Prefix</label>
            <input
              {...register('s3_prefix')}
              className="w-full px-3 py-2 border border-border rounded-md bg-white dark:bg-gray-700"
              placeholder="products/prod-001/"
            />
            {errors.s3_prefix && <p className="text-destructive text-sm mt-1">{errors.s3_prefix.message}</p>}
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
              disabled={isSubmitting || createProduct.isPending || updateProduct.isPending}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90 disabled:opacity-50"
            >
              {isSubmitting || createProduct.isPending || updateProduct.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  Saving...
                </>
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

const tierColors: Record<string, string> = {
  personal: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
  small_commercial: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
  extended_commercial: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
};

export function CatalogPage() {
  const [showDialog, setShowDialog] = useState(false);
  const [editingProduct, setEditingProduct] = useState<ProductDialogProps['product'] | null>(null);
  const [detailProduct, setDetailProduct] = useState<ProductDialogProps['product'] & { listing_metadata?: unknown; cover_s3_key?: string | null } | null>(null);
  const deleteProduct = useDeleteProduct();

  const { data, isLoading, error } = useProducts();

  const handleCreate = () => {
    setEditingProduct(null);
    setShowDialog(true);
  };

  const handleEdit = (product: ProductDialogProps['product']) => {
    setEditingProduct(product);
    setShowDialog(true);
  };

  const handleDelete = async (id: string) => {
    if (window.confirm('Are you sure you want to delete this product?')) {
      await deleteProduct.mutateAsync(id);
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
    return <div className="text-destructive">Failed to load products</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Final Products</h1>
          <p className="text-muted-foreground mt-1">
            Assembled, publish-ready product sets — click any product to preview prints, package contents and listing metadata
          </p>
        </div>
        <button onClick={handleCreate} className="btn-primary">
          <Plus className="h-4 w-4 mr-2" /> Add Product
        </button>
      </div>

      {data?.items?.length === 0 ? (
        <Card>
          <CardContent className="p-12 text-center">
            <Package className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium text-foreground">No products yet</h3>
            <p className="text-muted-foreground mt-1">
              Approve variants in Curation to assemble bundles, then register the set here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {data?.items?.map((product) => (
            <div
              key={product.id}
              className="border rounded-lg overflow-hidden bg-card hover:shadow-lg transition-all cursor-pointer group relative"
              onClick={() => setDetailProduct(product)}
            >
              <div className="aspect-[4/3] bg-muted relative overflow-hidden">
                {product.cover_s3_key ? (
                  <img
                    src={assetUrl(product.cover_s3_key)}
                    alt={product.title}
                    className="w-full h-full object-cover transition-transform group-hover:scale-105"
                    loading="lazy"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center">
                    <Package className="h-10 w-10 text-muted-foreground" />
                  </div>
                )}
                <span className={`absolute top-2 right-2 px-2 py-0.5 rounded-full text-xs font-medium ${tierColors[product.license_tier] ?? 'bg-muted text-foreground'}`}>
                  {product.license_tier.replace('_', ' ')}
                </span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(product.id);
                  }}
                  className="absolute top-2 left-2 p-1.5 bg-black/50 text-white rounded-md opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-600"
                  title="Delete product"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
              <div className="p-4 space-y-2">
                <h3 className="font-medium text-foreground text-sm line-clamp-2 min-h-[2.5rem]">{product.title}</h3>
                <div className="flex items-center justify-between">
                  <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{product.sku}</code>
                  <span className="text-lg font-bold text-foreground">{formatCurrency(product.price)}</span>
                </div>
                <p className="text-xs text-muted-foreground">{formatDate(product.created_at)}</p>
              </div>
            </div>
          ))}
        </div>
      )}

      {detailProduct && (
        <ProductDetailDialog
          product={detailProduct as never}
          onClose={() => setDetailProduct(null)}
          onEdit={(p: ProductDialogProps['product']) => {
            setDetailProduct(null);
            handleEdit(p);
          }}
        />
      )}

      {showDialog && (
        <ProductDialog
          product={editingProduct}
          onClose={() => {
            setShowDialog(false);
            setEditingProduct(null);
          }}
        />
      )}
    </div>
  );
}