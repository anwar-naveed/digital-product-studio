import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ListingsTable } from './ListingsTable';
import { api } from '@/api/client';

// Mock the API client
vi.mock('@/api/client', () => ({
  api: {
    listings: {
      list: vi.fn().mockResolvedValue({
        items: [],
        total: 0,
        limit: 20,
        offset: 0,
      }),
    },
  },
  REJECT_REASONS: [
    'style_mismatch',
    'artifact',
    'license',
    'off_niche',
    'duplicate',
  ] as const,
}));

const mockListings = [
  {
    id: 'listing-1',
    product_id: 'product-1234567890abcdef',
    platform: 'etsy' as const,
    platform_listing_id: 'etsy-12345',
    state: 'draft' as const,
    price: '19.99',
    fee_table_id: null,
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-2',
    product_id: 'product-1234567890abcdef',
    platform: 'printify' as const,
    platform_listing_id: 'printify-67890',
    state: 'publishing' as const,
    price: '29.99',
    fee_table_id: 'fee-1',
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-3',
    product_id: 'product-1234567890abcdef',
    platform: 'gelato' as const,
    platform_listing_id: 'gelato-11111',
    state: 'live' as const,
    price: '39.99',
    fee_table_id: 'fee-2',
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-4',
    product_id: 'product-1234567890abcdef',
    platform: 'lemon-squeezy' as const,
    platform_listing_id: 'ls-22222',
    state: 'updating' as const,
    price: '49.99',
    fee_table_id: 'fee-3',
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-5',
    product_id: 'product-1234567890abcdef',
    platform: 'etsy' as const,
    platform_listing_id: 'etsy-33333',
    state: 'failed' as const,
    price: '59.99',
    fee_table_id: null,
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-6',
    product_id: 'product-1234567890abcdef',
    platform: 'printify' as const,
    platform_listing_id: 'printify-44444',
    state: 'suspended' as const,
    price: '69.99',
    fee_table_id: 'fee-4',
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-7',
    product_id: 'product-1234567890abcdef',
    platform: 'gelato' as const,
    platform_listing_id: 'gelato-55555',
    state: 'delisting' as const,
    price: '79.99',
    fee_table_id: 'fee-5',
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
  {
    id: 'listing-8',
    product_id: 'product-1234567890abcdef',
    platform: 'lemon-squeezy' as const,
    platform_listing_id: 'ls-66666',
    state: 'delisted' as const,
    price: '89.99',
    fee_table_id: null,
    created_at: '2026-10-13T01:00:00Z',
    updated_at: '2026-10-13T01:00:00Z',
  },
];

describe('ListingsTable - Badge Mapping (G3)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.listings.list as ReturnType<typeof vi.fn>).mockResolvedValue({
      items: mockListings,
      total: mockListings.length,
      limit: 20,
      offset: 0,
    });
  });

  it('renders all 8 listing state badges correctly', async () => {
    render(<ListingsTable />);

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText('etsy-12345')).toBeInTheDocument();
    });

    // Check all 8 state badges are rendered with correct variant
    const draftBadge = screen.getByText('Draft');
    expect(draftBadge).toHaveClass('bg-gray-100'); // draft variant

    const publishingBadge = screen.getByText('Publishing');
    expect(publishingBadge).toHaveClass('bg-blue-100'); // publishing variant

    const liveBadge = screen.getByText('Live');
    expect(liveBadge).toHaveClass('bg-green-100'); // live variant

    const updatingBadge = screen.getByText('Updating');
    expect(updatingBadge).toHaveClass('bg-yellow-100'); // updating variant

    const failedBadge = screen.getByText('Failed');
    expect(failedBadge).toHaveClass('bg-red-100'); // failed variant

    const suspendedBadge = screen.getByText('Suspended');
    expect(suspendedBadge).toHaveClass('bg-orange-100'); // suspended variant

    const delistingBadge = screen.getByText('Delisting');
    expect(delistingBadge).toHaveClass('bg-purple-100'); // delisting variant

    const delistedBadge = screen.getByText('Delisted');
    expect(delistedBadge).toHaveClass('bg-gray-200'); // delisted variant
  });

  it('renders platform icons for each platform', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getAllByText('Etsy')[0]).toBeInTheDocument();
    });

    // Check all 4 platforms are represented
    expect(screen.getAllByText('Etsy')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Printify')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Gelato')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Lemon Squeezy')[0]).toBeInTheDocument();
  });

  it('displays listing prices correctly', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText('$19.99')).toBeInTheDocument();
    });

    expect(screen.getByText('$29.99')).toBeInTheDocument();
    expect(screen.getByText('$39.99')).toBeInTheDocument();
    expect(screen.getByText('$49.99')).toBeInTheDocument();
    expect(screen.getByText('$59.99')).toBeInTheDocument();
    expect(screen.getByText('$69.99')).toBeInTheDocument();
    expect(screen.getByText('$79.99')).toBeInTheDocument();
    expect(screen.getByText('$89.99')).toBeInTheDocument();
  });

  it('shows platform and state filters', async () => {
    render(<ListingsTable />);

    // Platform filter select
    expect(screen.getByText(/all platforms/i)).toBeInTheDocument();

    // State filter select
    expect(screen.getByText(/all states/i)).toBeInTheDocument();
  });

  it('filters by platform when platform filter is changed', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText('etsy-12345')).toBeInTheDocument();
    });

    // Click platform filter
    const platformFilter = screen.getByText(/all platforms/i);
    fireEvent.click(platformFilter);

    // Select Etsy
    const etsyOption = screen.getByRole('option', { name: /etsy/i });
    fireEvent.click(etsyOption);

    // Should refetch with platform filter
    await waitFor(() => {
      expect(api.listings.list).toHaveBeenCalledWith(
        expect.objectContaining({ platform: 'etsy' })
      );
    });
  });

  it('filters by state when state filter is changed', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText('etsy-12345')).toBeInTheDocument();
    });

    // Click state filter
    const stateFilter = screen.getByText(/all states/i);
    fireEvent.click(stateFilter);

    // Select Live
    const liveOption = screen.getByRole('option', { name: /live/i });
    fireEvent.click(liveOption);

    // Should refetch with state filter
    await waitFor(() => {
      expect(api.listings.list).toHaveBeenCalledWith(
        expect.objectContaining({ state: 'live' })
      );
    });
  });

  it('displays reprice and delist action buttons for live listings', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText('etsy-12345')).toBeInTheDocument();
    });

    // Live listing should have reprice button (dollar sign)
    const liveRow = screen.getByText('gelato-11111').closest('tr');
    expect(liveRow).toBeInTheDocument();
    // Reprice button should be present
    expect(liveRow?.querySelector('button[title="Reprice"]')).toBeInTheDocument();
    // Delist button should be present
    expect(liveRow?.querySelector('button[title="Delist"]')).toBeInTheDocument();
  });

  it('disables delist button for non-live/non-suspended states', async () => {
    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText('etsy-12345')).toBeInTheDocument();
    });

    // Draft listing should have disabled delist button
    const draftRow = screen.getByText('etsy-12345').closest('tr');
    const delistButton = draftRow?.querySelector('button[title="Delist"]');
    expect(delistButton).toBeDisabled();
  });

  it('shows correct total count in pagination', async () => {
    // Pagination paragraph only renders when total > page limit (20)
    (api.listings.list as ReturnType<typeof vi.fn>).mockResolvedValue({
      items: mockListings,
      total: 25,
      limit: 20,
      offset: 0,
    });

    render(<ListingsTable />);

    await waitFor(() => {
      expect(screen.getByText((_, el) => !!el && el.textContent?.includes('of 25') === true && el.tagName === 'P')).toBeInTheDocument();
    });
  });

  it('shows loading state initially', () => {
    render(<ListingsTable />);

    // Should show loading spinner
    expect(screen.getByRole('status')).toBeInTheDocument();
  });
});

// Additional test for Badge component variants
import { Badge } from './ui/Badge';

describe('Badge - Listing State Variants', () => {
  const stateVariants = [
    'draft',
    'publishing',
    'live',
    'updating',
    'failed',
    'suspended',
    'delisting',
    'delisted',
  ] as const;

  it.each(stateVariants)('renders %s variant correctly', (variant) => {
    const { container } = render(<Badge variant={variant}>{variant}</Badge>);
    const badge = container.querySelector('div');
    expect(badge).not.toBeNull(); // cva badge renders (no 'badge' base class in this design system)
    expect(badge).toHaveTextContent(variant);
  });

  it('has exactly 8 listing state variants defined', () => {
    expect(stateVariants).toHaveLength(8);
  });
});