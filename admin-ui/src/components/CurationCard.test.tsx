import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CurationCard } from './CurationCard';






// Mock the API client
vi.mock('@/api/client', () => ({
  api: {
    curation: {
      approve: vi.fn().mockResolvedValue({}),
      reject: vi.fn().mockResolvedValue({}),
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

const mockItem = {
  variant_id: 'variant-1234567890abcdef',
  design_id: 'design-1234567890abcdef',
  run_id: 'run-1234567890abcdef',
  niche_id: 'niche-wall-art-gothic',
  s3_key: 'curated/designs/variant-1234567890abcdef.png',
  sha256: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
  kind: 'image_generation',
  base_prompt: 'A gothic wall art design with dark aesthetic',
  presigned_url: 'https://example.com/presigned-image.png',
  created_at: '2026-10-13T01:00:00Z',
};

describe('CurationCard', () => {
  let onApprove: ReturnType<typeof vi.fn>;
  let onReject: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    onApprove = vi.fn().mockResolvedValue({});
    onReject = vi.fn().mockResolvedValue({});
    vi.clearAllMocks();
  });

  it('renders curation card with all item details', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    // Check variant ID is displayed (truncated)
    expect(screen.getByText(/variant-1234/)).toBeInTheDocument();
    // Check design ID
    expect(screen.getByText(/design-12345/)).toBeInTheDocument();
    // Check run ID
    expect(screen.getByText(/run-12345678/)).toBeInTheDocument();
    // Check niche badge
    expect(screen.getByText('niche-wall-art-gothic')).toBeInTheDocument();
    // Check kind badge
    expect(screen.getByText('Image Generation')).toBeInTheDocument();
    // Check SHA256 (truncated)
    expect(screen.getByText('abcdef1234567890...')).toBeInTheDocument();
    // Check prompt
    expect(screen.getByText('A gothic wall art design with dark aesthetic')).toBeInTheDocument();
  });

  it('displays approve and reject buttons', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    expect(screen.getByRole('button', { name: /approve/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reject/i })).toBeInTheDocument();
  });

  it('calls onApprove when approve button is clicked', async () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    const approveButton = screen.getByRole('button', { name: /approve/i });
    fireEvent.click(approveButton);

    await waitFor(() => {
      expect(onApprove).toHaveBeenCalledWith('variant-1234567890abcdef');
    });
  });

  it('opens reject dialog when reject button is clicked', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    // Check reject dialog is open
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText(/reject curation item/i)).toBeInTheDocument();
  });

  it('shows all 5 reject reason options in select', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    // Open the select
    const selectTrigger = screen.getByRole('combobox');
    fireEvent.click(selectTrigger);

    // Check all 5 reject reasons are present
    expect(screen.getByRole('option', { name: /style mismatch/i })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /artifact/i })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /license/i })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /off niche/i })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /duplicate/i })).toBeInTheDocument();
  });

  it('calls onReject with selected reason when confirm reject is clicked', async () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    // Open reject dialog
    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    // Select a reason
    const selectTrigger = screen.getByRole('combobox');
    fireEvent.click(selectTrigger);
    const licenseOption = screen.getByRole('option', { name: /license/i });
    fireEvent.click(licenseOption);

    // Click confirm reject
    const confirmButton = screen.getByRole('button', { name: /confirm reject/i });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(onReject).toHaveBeenCalledWith('variant-1234567890abcdef', 'license');
    });
  });

  it('shows loading state during approve', async () => {
    let resolveApprove: (value: unknown) => void;
    const approvePromise = new Promise(resolve => { resolveApprove = resolve; });
    onApprove.mockReturnValue(approvePromise);

    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    const approveButton = screen.getByRole('button', { name: /approve/i });
    fireEvent.click(approveButton);

    // Should show loading state
    expect(screen.getByRole('button', { name: /approving.../i })).toBeInTheDocument();
    expect(approveButton).toBeDisabled();

    resolveApprove!(undefined);
    await approvePromise;

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /approve/i })).toBeInTheDocument();
    });
  });

  it('shows loading state during reject', async () => {
    let resolveReject: (value: unknown) => void;
    const rejectPromise = new Promise(resolve => { resolveReject = resolve; });
    onReject.mockReturnValue(rejectPromise);

    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    // Open reject dialog
    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    // Select a reason
    const selectTrigger = screen.getByRole('combobox');
    fireEvent.click(selectTrigger);
    const artifactOption = screen.getByRole('option', { name: /artifact/i });
    fireEvent.click(artifactOption);

    // Click confirm reject
    const confirmButton = screen.getByRole('button', { name: /confirm reject/i });
    fireEvent.click(confirmButton);

    // Should show loading state
    expect(screen.getByRole('button', { name: /rejecting.../i })).toBeInTheDocument();

    resolveReject!(undefined);
    await rejectPromise;
  });

  it('closes reject dialog when cancel is clicked', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    expect(screen.getByRole('dialog')).toBeInTheDocument();

    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    fireEvent.click(cancelButton);

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('opens full preview dialog when image is clicked', () => {
    render(
      <CurationCard
        item={mockItem}
        onApprove={onApprove}
        onReject={onReject}
      />
    );

    // Click the image area (eye button overlay)
    const imageContainer = screen.getByAltText('Curation item variant-1234567890abcdef');
    fireEvent.mouseEnter(imageContainer);

    // The eye button should be visible on hover
    const eyeButton = screen.getByTestId('icon-eye').closest('button')!
    if (eyeButton) fireEvent.click(eyeButton);

    // Full preview dialog should open
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByAltText('Full preview variant-1234567890abcdef')).toBeInTheDocument();
  });
});