// TanStack Query hooks for Settings domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, UpdateFeeTableRequest, Platform} from '../api/client';

// Query keys
export const settingsKeys = {
  all: ['settings'] as const,
  credentials: () => [...settingsKeys.all, 'credentials'] as const,
  feeTables: () => [...settingsKeys.all, 'feeTables'] as const,
};

// Hooks
export function useCredentials() {
  return useQuery({
    queryKey: settingsKeys.credentials(),
    queryFn: () => api.settings.credentials(),
    staleTime: 1000 * 60 * 10, // 10 minutes
  });
}

export function useUpdateCredential() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ platform, ref }: { platform: Platform; ref: string }) => 
      api.settings.updateCredential(platform, ref),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.credentials() });
    },
  });
}

export function useFeeTables() {
  return useQuery({
    queryKey: settingsKeys.feeTables(),
    queryFn: () => api.settings.feeTables(),
    staleTime: 1000 * 60 * 10,
  });
}

export function useUpdateFeeTable() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ platform, data }: { platform: Platform; data: UpdateFeeTableRequest }) => 
      api.settings.updateFeeTable(platform, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.feeTables() });
    },
  });
}