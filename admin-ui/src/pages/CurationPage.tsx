// Curation page component
import { useCurationQueue, useApproveCuration, useRejectCuration } from '../hooks/useCuration';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDateTime } from '../lib/utils';
import { Check, X, Loader2, Image, Eye } from 'lucide-react';
import { useState } from 'react';
import { assetUrl } from '../api/client';

const REJECT_REASONS = [
  { value: 'style_mismatch', label: 'Style Mismatch' },
  { value: 'artifact', label: 'Artifact/Quality Issue' },
  { value: 'license', label: 'License Concern' },
  { value: 'off_niche', label: 'Off-Niche' },
  { value: 'duplicate', label: 'Duplicate' },
];

interface PreviewItem {
  variantId: string;
  s3Key: string;
  basePrompt: string;
  kind: string;
  runId: string;
}

export function CurationPage() {
  const [selectedVariant, setSelectedVariant] = useState<{
    variantId: string;
    rejectReason: string;
  } | null>(null);
  const [previewItem, setPreviewItem] = useState<PreviewItem | null>(null);
  
  const { data, isLoading, error, refetch } = useCurationQueue({ limit: 50, offset: 0 });
  const approveCuration = useApproveCuration();
  const rejectCuration = useRejectCuration();

  const handleApprove = async (variantId: string) => {
    await approveCuration.mutateAsync({ variantId, data: {} });
    setPreviewItem(null);
    refetch();
  };

  const handleRejectOpen = (variantId: string) => {
    setSelectedVariant({ variantId, rejectReason: '' });
  };

  const handleRejectConfirm = async () => {
    if (!selectedVariant) return;
    await rejectCuration.mutateAsync({
      variantId: selectedVariant.variantId,
      data: { reject_reason: selectedVariant.rejectReason },
    });
    setSelectedVariant(null);
    refetch();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-destructive">Failed to load curation queue</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Curation Queue</h1>
        <p className="text-muted-foreground mt-1">
          Review and approve/reject generated design variants
        </p>
      </div>

      <Card>
        <CardContent className="p-0">
          {data?.items?.length === 0 ? (
            <div className="p-12 text-center">
              <Image className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-foreground">No items in queue</h3>
              <p className="text-muted-foreground mt-1">
                All variants have been reviewed. Spawn a new production run to generate more.
              </p>
            </div>
          ) : (
            <div className="grid gap-4 p-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {data?.items?.map((item) => (
                <div key={item.variant_id} className="border rounded-lg overflow-hidden bg-card hover:shadow-md transition-shadow">
                  <div
                    className="aspect-square relative bg-muted cursor-zoom-in"
                    onClick={() => setPreviewItem({
                      variantId: item.variant_id,
                      s3Key: item.s3_key,
                      basePrompt: item.base_prompt,
                      kind: item.kind,
                      runId: item.run_id,
                    })}
                  >
                    <img
                      src={assetUrl(item.s3_key)}
                      alt={item.base_prompt}
                      className="w-full h-full object-cover"
                      loading="lazy"
                    />
                    <div className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
                      <Eye className="h-8 w-8 text-white" />
                    </div>
                  </div>
                  <div className="p-4 space-y-3">
                    <h4 className="font-medium text-foreground truncate">
                      {item.base_prompt.slice(0, 50)}...
                    </h4>
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <span className="px-2 py-0.5 bg-muted rounded">
                        {item.kind}
                      </span>
                      <span>{formatDateTime(item.created_at)}</span>
                    </div>
                    <div className="flex gap-2 pt-2">
                      <button
                        onClick={() => handleApprove(item.variant_id)}
                        disabled={approveCuration.isPending}
                        className="flex-1 btn-primary justify-center"
                      >
                        {approveCuration.isPending ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <>
                            <Check className="h-4 w-4 mr-2" />
                            Approve
                          </>
                        )}
                      </button>
                      <button
                        onClick={() => handleRejectOpen(item.variant_id)}
                        disabled={rejectCuration.isPending}
                        className="btn-destructive"
                      >
                        <X className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Full-size Preview Dialog */}
      {previewItem && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" onClick={() => setPreviewItem(null)}>
          <div
            className="w-full max-w-5xl max-h-[92vh] bg-card rounded-lg shadow-xl overflow-hidden flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-5 py-3 border-b border-border shrink-0">
              <h2 className="text-lg font-semibold text-foreground">Variant Preview</h2>
              <button onClick={() => setPreviewItem(null)} className="p-2 hover:bg-accent rounded-md text-muted-foreground">
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto grid md:grid-cols-[2fr,1fr] gap-0">
              <div className="bg-black/20 flex items-center justify-center p-4">
                <img
                  src={assetUrl(previewItem.s3Key)}
                  alt={previewItem.basePrompt}
                  className="max-h-[70vh] w-auto object-contain rounded"
                />
              </div>
              <div className="p-5 space-y-4 border-l border-border">
                <div>
                  <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1">Prompt</p>
                  <p className="text-sm text-foreground">{previewItem.basePrompt}</p>
                </div>
                <div>
                  <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1">Variant</p>
                  <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{previewItem.variantId.slice(0, 12)}…</code>
                </div>
                <div>
                  <p className="text-xs uppercase tracking-wide text-muted-foreground mb-1">Run</p>
                  <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{previewItem.runId.slice(0, 12)}…</code>
                </div>
                <div className="pt-4 border-t border-border flex gap-2">
                  <button
                    onClick={() => handleApprove(previewItem.variantId)}
                    disabled={approveCuration.isPending}
                    className="flex-1 px-3 py-2 bg-green-600 text-white rounded-md text-sm hover:bg-green-700 disabled:opacity-50 flex items-center justify-center gap-2"
                  >
                    {approveCuration.isPending ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <>
                        <Check className="h-4 w-4" /> Approve
                      </>
                    )}
                  </button>
                  <button
                    onClick={() => {
                      handleRejectOpen(previewItem.variantId);
                      setPreviewItem(null);
                    }}
                    className="flex-1 px-3 py-2 bg-red-600 text-white rounded-md text-sm hover:bg-red-700 flex items-center justify-center gap-2"
                  >
                    <X className="h-4 w-4" /> Reject
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Reject Dialog */}
      {selectedVariant && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-md bg-card rounded-lg shadow-xl p-6">
            <h2 className="text-xl font-semibold mb-4">Reject Variant</h2>
            <p className="text-muted-foreground mb-4">
              Select a reason for rejection:
            </p>
            <div className="space-y-2 mb-6">
              {REJECT_REASONS.map((reason) => (
                <label key={reason.value} className="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-accent">
                  <input
                    type="radio"
                    name="reject_reason"
                    value={reason.value}
                    checked={selectedVariant.rejectReason === reason.value}
                    onChange={() => setSelectedVariant({ ...selectedVariant, rejectReason: reason.value })}
                    className="text-primary"
                  />
                  <span className="text-foreground">{reason.label}</span>
                </label>
              ))}
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setSelectedVariant(null)}
                className="px-4 py-2 border rounded-md text-sm hover:bg-accent"
              >
                Cancel
              </button>
              <button
                onClick={handleRejectConfirm}
                disabled={!selectedVariant.rejectReason || rejectCuration.isPending}
                className="px-4 py-2 bg-red-600 text-white rounded-md text-sm hover:bg-red-700 disabled:opacity-50"
              >
                {rejectCuration.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  'Reject'
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}