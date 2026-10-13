// Catalog page component
import { useProducts } from '../hooks/useProducts';
import { useCreateProduct, useUpdateProduct, useDeleteProduct } from '../hooks/useProducts';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDate, formatCurrency } from '../lib/utils';
import { Plus, Edit, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

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
      <div className="w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6">
        <h2 className="text-xl font-semibold mb-4">{isEditing ? 'Edit Product' : 'Create Product'}</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">SKU</label>
            <input
              {...register('sku')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
              placeholder="PROD-001"
            />
            {errors.sku && <p className="text-red-500 text-sm mt-1">{errors.sku.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Title</label>
            <input
              {...register('title')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
              placeholder="Product title"
            />
            {errors.title && <p className="text-red-500 text-sm mt-1">{errors.title.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">License Tier</label>
            <select
              {...register('license_tier')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
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
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
              placeholder="29.99"
            />
            {errors.price && <p className="text-red-500 text-sm mt-1">{errors.price.message}</p>}
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">S3 Prefix</label>
            <input
              {...register('s3_prefix')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
              placeholder="products/prod-001/"
            />
            {errors.s3_prefix && <p className="text-red-500 text-sm mt-1">{errors.s3_prefix.message}</p>}
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

// Need to import Loader2
import { Loader2 } from 'lucide-react';

const tierColors: Record<string, string> = {
  personal: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
  small_commercial: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
  extended_commercial: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
};

export function CatalogPage() {
  const [showDialog, setShowDialog] = useState(false);
  const [editingProduct, setEditingProduct] = useState<ProductDialogProps['product'] | null>(null);
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
    return <div className="text-red-500">Failed to load products</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Catalog</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">Manage product bundles with 3-tier pricing</p>
        </div>
        <button onClick={handleCreate} className="btn-primary">
          <Plus className="h-4 w-4 mr-2" /> Add Product
        </button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-gray-50 dark:bg-gray-800/50">
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">SKU</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Title</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">License Tier</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Price</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">S3 Prefix</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Created</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {data?.items?.map((product) => (
                  <tr key={product.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td className="px-6 py-4 font-mono text-sm text-gray-900 dark:text-white">{product.sku}</td>
                    <td className="px-6 py-4 font-medium text-gray-900 dark:text-white">{product.title}</td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${tierColors[product.license_tier] ?? 'bg-gray-100 text-gray-800'}`}>
                        {product.license_tier.replace('_', ' ')}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-gray-900 dark:text-white">{formatCurrency(product.price)}</td>
                    <td className="px-6 py-4 font-mono text-sm text-gray-600 dark:text-gray-300">{product.s3_prefix}</td>
                    <td className="px-6 py-4 text-gray-600 dark:text-gray-300">{formatDate(product.created_at)}</td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleEdit(product)}
                          className="p-2 text-gray-500 hover:text-primary hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                        >
                          <Edit className="h-4 w-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(product.id)}
                          className="p-2 text-gray-500 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
                {(!data?.items.length) && (
                  <tr>
                    <td colSpan={7} className="px-6 py-12 text-center text-gray-500 dark:text-gray-400">
                      No products found. Create your first product bundle to get started.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

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