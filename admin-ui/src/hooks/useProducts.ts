// TanStack Query hooks for Products/Catalog domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, CreateProductRequest, UpdateProductRequest, ListProductsQuery} from '../api/client';

// Query keys
export const productKeys = {
  all: ['products'] as const,
  lists: () => [...productKeys.all, 'list'] as const,
  list: (query?: ListProductsQuery) => [...productKeys.lists(), query] as const,
  details: () => [...productKeys.all, 'detail'] as const,
  detail: (id: string) => [...productKeys.details(), id] as const,
};

// Hooks
export function useProducts(query?: ListProductsQuery) {
  return useQuery({
    queryKey: productKeys.list(query),
    queryFn: () => api.catalog.list(query),
    staleTime: 1000 * 60 * 5,
  });
}

export function useProduct(id: string) {
  return useQuery({
    queryKey: productKeys.detail(id),
    queryFn: () => api.catalog.get(id),
    enabled: !!id,
    staleTime: 1000 * 60 * 5,
  });
}

export function useCreateProduct() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateProductRequest) => api.catalog.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: productKeys.lists() });
    },
  });
}

export function useUpdateProduct() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateProductRequest }) => api.catalog.update(id, data),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: productKeys.lists() });
      queryClient.invalidateQueries({ queryKey: productKeys.detail(id) });
    },
  });
}

export function useDeleteProduct() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => api.catalog.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: productKeys.lists() });
    },
  });
}