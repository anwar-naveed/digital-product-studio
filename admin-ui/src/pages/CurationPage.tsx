// Curation page component
import { useCurationQueue, useApproveCuration, useRejectCuration } from '../hooks/useCuration';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDateTime } from '../lib/utils';
import { Check, X, Loader2, Image } from 'lucide-react';
import { useState } from 'react';
import { assetUrl } from '../api/client';

const REJECT_REASONS = [
  { value: 'style_mismatch', label: 'Style Mismatch' },
  { value: 'artifact', label: 'Artifact/Quality Issue' },
  { value: 'license', label: 'License Concern' },
  { value: 'off_niche', label: 'Off-Niche' },
  { value: 'duplicate', label: 'Duplicate' },
];

export function CurationPage() {
  const [selectedVariant, setSelectedVariant] = useState<{
    variantId: string;
    rejectReason: string;
  } | null>(null);
  
  const { data, isLoading, error, refetch } = useCurationQueue({ limit: 50, offset: 0 });
  const approveCuration = useApproveCuration();
  const rejectCuration = useRejectCuration();

  const handleApprove = async (variantId: string) => {
    await approveCuration.mutateAsync({ variantId, data: {} });
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
    return <div className="text-red-500">Failed to load curation queue</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Curation Queue</h1>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Review and approve/reject generated design variants
        </p>
      </div>

      <Card>
        <CardContent className="p-0">
          {data?.items?.length === 0 ? (
            <div className="p-12 text-center">
              <Image className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">No items in queue</h3>
              <p className="text-gray-500 dark:text-gray-400 mt-1">
                All variants have been reviewed. Spawn a new production run to generate more.
              </p>
            </div>
          ) : (
            <div className="grid gap-4 p-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {data?.items?.map((item) => (
                <div key={item.variant_id} className="border rounded-lg overflow-hidden bg-white dark:bg-gray-800 hover:shadow-md transition-shadow">
                  <div className="aspect-square relative bg-gray-100 dark:bg-gray-700">
                    <img
                      src={assetUrl(item.s3_key)}
                      alt={item.base_prompt}
                      className="w-full h-full object-cover"
                      loading="lazy"
                    />
                    <div className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
                      <span className="text-white font-medium">Preview</span>
                    </div>
                  </div>
                  <div className="p-4 space-y-3">
                    <h4 className="font-medium text-gray-900 dark:text-white truncate">
                      {item.base_prompt.slice(0, 50)}...
                    </h4>
                    <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
                      <span className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">
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

      {/* Reject Dialog */}
      {selectedVariant && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6">
            <h2 className="text-xl font-semibold mb-4">Reject Variant</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Select a reason for rejection:
            </p>
            <div className="space-y-2 mb-6">
              {REJECT_REASONS.map((reason) => (
                <label key={reason.value} className="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50">
                  <input
                    type="radio"
                    name="reject_reason"
                    value={reason.value}
                    checked={selectedVariant.rejectReason === reason.value}
                    onChange={() => setSelectedVariant({ ...selectedVariant, rejectReason: reason.value })}
                    className="text-primary"
                  />
                  <span className="text-gray-900 dark:text-white">{reason.label}</span>
                </label>
              ))}
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setSelectedVariant(null)}
                className="px-4 py-2 border rounded-md text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
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