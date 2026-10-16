// API Types matching backend DTOs from pipeline-core/src/api/*.rs

// Base types
export interface UUID {
  id: string;
}

// Error envelope from backend
export interface ApiError {
  error: {
    code: string;
    message: string;
    correlation_id: string;
  };
}

// Pagination
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}

// Niche types
export interface NicheResponse {
  id: string;
  name: string;
  vertical: string;
  demand_signals: Record<string, unknown>;
  competition_score: string; // Decimal as string
  created_at: string; // ISO 8601
}

export interface CreateNicheRequest {
  name: string;
  vertical: string;
  demand_signals?: Record<string, unknown>;
  competition_score?: string;
}

export interface UpdateNicheRequest {
  name?: string;
  vertical?: string;
  demand_signals?: Record<string, unknown>;
  competition_score?: string;
}

export interface ListNichesQuery {
  limit: number;
  offset: number;
  vertical?: string;
}

export interface NicheListResponse extends PaginatedResponse<NicheResponse> {}

// Run types
export interface RunResponse {
  id: string;
  niche_id: string;
  status: string;
  target_count: number;
  params: Record<string, unknown>;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
}

export interface JobSummary {
  id: string;
  run_id: string;
  tool: string;
  status: string;
  attempts: number;
  queued_at: string;
  started_at: string | null;
  finished_at: string | null;
  last_error: string | null;
}

export interface RunDetailResponse extends RunResponse {
  jobs: JobSummary[];
}

export interface CreateRunRequest {
  niche_id: string;
  target_count: number;
  params?: Record<string, unknown>;
}

export interface ListRunsQuery {
  limit: number;
  offset: number;
  status?: string;
  niche_id?: string;
}

export interface RunListResponse extends PaginatedResponse<RunResponse> {}

export interface RunVariantItem {
  variant_id: string;
  design_id: string;
  s3_key: string;
  sha256: string;
  kind: string;
  presigned_url: string;
  created_at: string;
}

export interface RunVariantsResponse {
  items: RunVariantItem[];
  total: number;
}

// Curation types
export interface CurationQueueItem {
  variant_id: string;
  design_id: string;
  run_id: string;
  niche_id: string;
  s3_key: string;
  sha256: string;
  kind: string;
  base_prompt: string;
  presigned_url: string;
  created_at: string;
}

export interface CurationQueueQuery {
  limit: number;
  offset: number;
  decided?: 'pending' | 'approved' | 'rejected';
}

export interface CurationQueueResponse extends PaginatedResponse<CurationQueueItem> {}

export interface ApproveCurationRequest {
  reviewer_id?: string;
}

export interface RejectCurationRequest {
  reject_reason: string;
  reviewer_id?: string;
}

export const REJECT_REASONS = [
  'style_mismatch',
  'artifact',
  'license',
  'off_niche',
  'duplicate',
] as const;

export type RejectReason = (typeof REJECT_REASONS)[number];

export interface CurationDecisionResponse {
  variant_id: string;
  decision: string;
  reviewer_id: string;
  decided_at: string;
  job_enqueued: string | null;
}

// Product types
export type LicenseTier = 'personal' | 'small_commercial' | 'extended_commercial';

export interface ProductItemResponse {
  id: string;
  product_id: string;
  variant_id: string;
  position: number;
  created_at: string;
}

export interface ProductResponse {
  id: string;
  sku: string;
  title: string;
  license_tier: LicenseTier;
  price: string; // Decimal as string
  s3_prefix: string;
  created_at: string;
}

export interface CreateProductItemRequest {
  variant_id: string;
  position: number;
}

export interface CreateProductRequest {
  sku: string;
  title: string;
  license_tier: LicenseTier;
  price: string;
  s3_prefix: string;
  items?: CreateProductItemRequest[];
}

export interface UpdateProductRequest {
  title?: string;
  license_tier?: LicenseTier;
  price?: string;
  s3_prefix?: string;
  items?: CreateProductItemRequest[];
}

export interface ListProductsQuery {
  limit: number;
  offset: number;
  license_tier?: LicenseTier;
}

export interface ProductListResponse extends PaginatedResponse<ProductResponse> {}

export interface ProductDetailResponse {
  product: ProductResponse;
  items: ProductItemResponse[];
}

// Listing types
export type Platform = 'etsy' | 'lemon-squeezy' | 'printify' | 'gelato';

export type ListingState = 
  | 'draft'
  | 'publishing'
  | 'live'
  | 'updating'
  | 'failed'
  | 'suspended'
  | 'delisting'
  | 'delisted';

export interface ListingResponse {
  id: string;
  product_id: string;
  platform: Platform;
  platform_listing_id: string;
  state: ListingState;
  price: string;
  fee_table_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListListingsQuery {
  limit: number;
  offset: number;
  platform?: Platform;
  state?: ListingState;
  product_id?: string;
}

export interface ListingListResponse extends PaginatedResponse<ListingResponse> {}

export interface PublishRequest {
  platform: Platform;
  license_tier?: LicenseTier;
}

export interface PublishResponse {
  listing_id: string;
  job_id: string;
  state: string;
  message: string;
}

export interface DelistRequest {
  platform: Platform;
}

export interface DelistResponse {
  listing_id: string;
  job_id: string;
  state: string;
  message: string;
}

export interface RepriceRequest {
  platform: Platform;
  base_price: string;
}

export interface FeePreview {
  platform_fee_percent: string;
  transaction_fee: string;
  personal_net: string;
  small_commercial_net: string;
  extended_commercial_net: string;
}

export interface RepriceResponse {
  platform: Platform;
  base_price: string;
  personal_price: string;
  small_commercial_price: string;
  extended_commercial_price: string;
  fee_preview: FeePreview;
}

// Analytics types
export interface ListingAnalyticsResponse {
  listing_id: string;
  product_id: string;
  product_sku: string;
  product_title: string;
  platform: Platform;
  platform_listing_id: string;
  state: ListingState;
  current_price: string;
  views: number;
  favorites: number;
  sales_count: number;
  total_revenue: string;
  total_fees: string;
  total_net: string;
  margin_percent: string;
  first_sale_at: string | null;
  last_sale_at: string | null;
}

export interface AnalyticsQuery {
  limit: number;
  offset: number;
  platform?: string;
  product_id?: string;
  since?: string;
  until?: string;
}

export interface AnalyticsListResponse extends PaginatedResponse<ListingAnalyticsResponse> {}

// Settings types
export interface CredentialMetadata {
  platform: Platform;
  has_credentials: boolean;
  last_validated: string | null;
}

export interface FeeTableResponse {
  id: string;
  platform: Platform;
  rules: Record<string, unknown>;
  effective_from: string;
}

export interface UpdateFeeTableRequest {
  rules?: Record<string, unknown>;
  platform_fee_percent?: string;
  transaction_fee?: string;
  currency?: string;
  payoneer_fee_percent?: string;
}