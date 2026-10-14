// Typed API client for admin-ui
// Matches pipeline-core API surface from docs/11-architecture.md §3

import type {
  // Niche types
  NicheResponse,
  CreateNicheRequest,
  UpdateNicheRequest,
  ListNichesQuery,
  NicheListResponse,
  // Run types
  RunResponse,
  RunDetailResponse,
  RunListResponse,
  CreateRunRequest,
  ListRunsQuery,
  // Curation types
  CurationQueueQuery,
  CurationQueueResponse,
  ApproveCurationRequest,
  RejectCurationRequest,
  CurationDecisionResponse,
  // Product types
  ProductResponse,
  ProductDetailResponse,
  ProductListResponse,
  CreateProductRequest,
  UpdateProductRequest,
  ListProductsQuery,
  // Listing types
  ListingListResponse,
  ListListingsQuery,
  PublishRequest,
  PublishResponse,
  DelistRequest,
  DelistResponse,
  RepriceRequest,
  RepriceResponse,
  Platform,
  // Analytics types
  AnalyticsQuery,
  AnalyticsListResponse,
  // Settings types
  CredentialMetadata,
  FeeTableResponse,
  UpdateFeeTableRequest,
} from '../types/api';

// Re-export all API types so consumers can import them from the client
export type * from '../types/api';
export { REJECT_REASONS } from '../types/api';

// API Configuration
const API_BASE = '/api/v1';

// Error handling
export class ApiError extends Error {
  constructor(
    public code: string,
    public message: string,
    public correlationId: string,
    public status: number
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// Token management
let authToken: string | null = null;

export function setAuthToken(token: string | null): void {
  authToken = token;
}

export function getAuthToken(): string | null {
  return authToken;
}

// Generic fetch wrapper with error handling
async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...options.headers,
  };

  if (authToken) {
    (headers as Record<string, string>)['Authorization'] = `Bearer ${authToken}`;
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    let errorData: { error: { code: string; message: string; correlation_id: string } } | null = null;
    
    try {
      errorData = await response.json();
    } catch {
      // Ignore JSON parse errors
    }

    if (response.status === 401) {
      // Clear token and redirect to login
      authToken = null;
      window.location.href = '/login';
      throw new ApiError(
        'UNAUTHORIZED',
        'Authentication required',
        errorData?.error?.correlation_id ?? 'unknown',
        401
      );
    }

    throw new ApiError(
      errorData?.error?.code ?? 'API_ERROR',
      errorData?.error?.message ?? `Request failed with status ${response.status}`,
      errorData?.error?.correlation_id ?? 'unknown',
      response.status
    );
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

// Niche API
export const nichesApi = {
  list: (query?: ListNichesQuery): Promise<NicheListResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.vertical) params.set('vertical', query.vertical);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<NicheListResponse>(`/niches${queryString}`);
  },

  create: (data: CreateNicheRequest): Promise<NicheResponse> =>
    fetchApi<NicheResponse>('/niches', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  get: (id: string): Promise<NicheResponse> =>
    fetchApi<NicheResponse>(`/niches/${id}`),

  update: (id: string, data: UpdateNicheRequest): Promise<NicheResponse> =>
    fetchApi<NicheResponse>(`/niches/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<void> =>
    fetchApi<void>(`/niches/${id}`, {
      method: 'DELETE',
    }),
};

// Runs API
export const runsApi = {
  list: (query?: ListRunsQuery): Promise<RunListResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.status) params.set('status', query.status);
    if (query?.niche_id) params.set('niche_id', query.niche_id);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<RunListResponse>(`/runs${queryString}`);
  },

  create: (data: CreateRunRequest): Promise<RunResponse> =>
    fetchApi<RunResponse>('/runs', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  get: (id: string): Promise<RunDetailResponse> =>
    fetchApi<RunDetailResponse>(`/runs/${id}`),

  cancel: (id: string): Promise<void> =>
    fetchApi<void>(`/runs/${id}/cancel`, {
      method: 'POST',
    }),
};

// Curation API
export const curationApi = {
  queue: (query?: CurationQueueQuery): Promise<CurationQueueResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.decided) params.set('decided', query.decided);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<CurationQueueResponse>(`/curation/queue${queryString}`);
  },

  approve: (variantId: string, data: ApproveCurationRequest): Promise<CurationDecisionResponse> =>
    fetchApi<CurationDecisionResponse>(`/curation/${variantId}/approve`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  reject: (variantId: string, data: RejectCurationRequest): Promise<CurationDecisionResponse> =>
    fetchApi<CurationDecisionResponse>(`/curation/${variantId}/reject`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
};

// Products/Catalog API
export const catalogApi = {
  list: (query?: ListProductsQuery): Promise<ProductListResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.license_tier) params.set('license_tier', query.license_tier);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<ProductListResponse>(`/catalog/products${queryString}`);
  },

  create: (data: CreateProductRequest): Promise<ProductResponse> =>
    fetchApi<ProductResponse>('/catalog/products', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  get: (id: string): Promise<ProductDetailResponse> =>
    fetchApi<ProductDetailResponse>(`/catalog/products/${id}`),

  update: (id: string, data: UpdateProductRequest): Promise<ProductResponse> =>
    fetchApi<ProductResponse>(`/catalog/products/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<void> =>
    fetchApi<void>(`/catalog/products/${id}`, {
      method: 'DELETE',
    }),
};

// Listings API
export const listingsApi = {
  list: (query?: ListListingsQuery): Promise<ListingListResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.platform) params.set('platform', query.platform);
    if (query?.state) params.set('state', query.state);
    if (query?.product_id) params.set('product_id', query.product_id);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<ListingListResponse>(`/listings${queryString}`);
  },

  publish: (productId: string, data: PublishRequest): Promise<PublishResponse> =>
    fetchApi<PublishResponse>(`/listings/${productId}/publish`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  delist: (productId: string, data: DelistRequest): Promise<DelistResponse> =>
    fetchApi<DelistResponse>(`/listings/${productId}/delist`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  reprice: (productId: string, data: RepriceRequest): Promise<RepriceResponse> =>
    fetchApi<RepriceResponse>(`/listings/${productId}/reprice`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
};

// Analytics API
export const analyticsApi = {
  listings: (query?: AnalyticsQuery): Promise<AnalyticsListResponse> => {
    const params = new URLSearchParams();
    if (query?.limit) params.set('limit', query.limit.toString());
    if (query?.offset) params.set('offset', query.offset.toString());
    if (query?.platform) params.set('platform', query.platform);
    if (query?.product_id) params.set('product_id', query.product_id);
    if (query?.since) params.set('since', query.since);
    if (query?.until) params.set('until', query.until);
    const queryString = params.toString() ? `?${params.toString()}` : '';
    return fetchApi<AnalyticsListResponse>(`/analytics/listings${queryString}`);
  },
};

// Settings API
export const settingsApi = {
  credentials: (): Promise<{ credentials: CredentialMetadata[]; total: number; limit: number; offset: number }> =>
    fetchApi<{ credentials: CredentialMetadata[]; total: number; limit: number; offset: number }>('/settings/credentials'),

  updateCredential: (platform: Platform, ref: string): Promise<void> =>
    fetchApi<void>(`/settings/credentials/${platform}`, {
      method: 'PUT',
      body: JSON.stringify({ ref }),
    }),

  feeTables: (): Promise<{ fee_tables: FeeTableResponse[]; total: number; limit: number; offset: number }> =>
    fetchApi<{ fee_tables: FeeTableResponse[]; total: number; limit: number; offset: number }>('/settings/fee-tables'),

  updateFeeTable: (platform: Platform, data: UpdateFeeTableRequest): Promise<void> =>
    fetchApi<void>(`/settings/fee-tables/${platform}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
};

// Health check
export const healthApi = {
  check: (): Promise<{ status: string }> =>
    fetchApi<{ status: string }>('/health'),
};

// Auth API (login/logout)
export const authApi = {
  login: async (username: string, password: string): Promise<{ token: string }> => {
    const response = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Login failed' }));
      throw new Error(error.message ?? 'Login failed');
    }

    const data = await response.json();
    setAuthToken(data.token);
    return data;
  },

  logout: (): void => {
    setAuthToken(null);
  },

  isAuthenticated: (): boolean => {
    return !!authToken;
  },
};

// Export all APIs
export const api = {
  niches: nichesApi,
  runs: runsApi,
  curation: curationApi,
  catalog: catalogApi,
  listings: listingsApi,
  analytics: analyticsApi,
  settings: settingsApi,
  health: healthApi,
  auth: authApi,
};

export default api;