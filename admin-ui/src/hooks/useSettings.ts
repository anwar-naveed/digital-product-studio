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
    // Core wraps the list: { credentials: [...] }
    queryFn: async () => (await api.settings.credentials()).credentials,
    staleTime: 1000 * 60 * 10, // 10 minutes
  });
}

export function useUpdateCredential() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ platform, fields }: { platform: Platform; fields: Record<string, string> }) =>
      api.settings.updateCredential(platform, fields),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.credentials() });
    },
  });
}

export function useFeeTables() {
  return useQuery({
    queryKey: settingsKeys.feeTables(),
    // Core wraps the list: { fee_tables: [...] }
    queryFn: async () => (await api.settings.feeTables()).fee_tables,
    staleTime: 1000 * 60 * 10, // 10 minutes
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