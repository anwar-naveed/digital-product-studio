'use client';

import { useState } from 'react';
import { Check, X, Loader2, Eye } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { type CurationQueueItem, type RejectReason, REJECT_REASONS } from '@/api/client';

interface CurationCardProps {
  item: CurationQueueItem;
  onApprove: (variantId: string) => void;
  onReject: (variantId: string, reason: RejectReason) => void;
}

export function CurationCard({ item, onApprove, onReject }: CurationCardProps) {
  const [showPreview, setShowPreview] = useState(false);
  const [rejectReason, setRejectReason] = useState<RejectReason>('style_mismatch');
  const [isApproving, setIsApproving] = useState(false);
  const [isRejecting, setIsRejecting] = useState(false);
  const [showRejectDialog, setShowRejectDialog] = useState(false);

  const handleApprove = async () => {
    setIsApproving(true);
    try {
      await onApprove(item.variant_id);
    } finally {
      setIsApproving(false);
    }
  };

  const handleReject = async () => {
    setIsRejecting(true);
    try {
      await onReject(item.variant_id, rejectReason);
      setShowRejectDialog(false);
    } finally {
      setIsRejecting(false);
    }
  };

  const formatKind = (kind: string) => {
    return kind.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  };

  return (
    <Card className="w-full">
      <CardContent className="p-4">
        <div className="grid gap-4">
          {/* Image Preview */}
          <div className="relative aspect-square overflow-hidden rounded-lg bg-muted">
            {item.presigned_url && (
              <img
                src={item.presigned_url}
                alt={`Curation item ${item.variant_id}`}
                className="w-full h-full object-cover transition-transform hover:scale-105"
                loading="lazy"
              />
            )}
            <div className="absolute inset-0 flex items-center justify-center gap-2 p-4 bg-black/50 opacity-0 hover:opacity-100 transition-opacity">
              <Button
                variant="ghost"
                size="icon"
                className="bg-white/90 text-foreground"
                onClick={() => setShowPreview(true)}
              >
                <Eye className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {/* Info Section */}
          <div className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">Variant:</span>
              <code className="text-xs bg-muted px-1.5 py-0.5 rounded">
                {item.variant_id.slice(0, 12)}...
              </code>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">Design:</span>
              <code className="text-xs bg-muted px-1.5 py-0.5 rounded">
                {item.design_id.slice(0, 12)}...
              </code>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">Run:</span>
              <code className="text-xs bg-muted px-1.5 py-0.5 rounded">
                {item.run_id.slice(0, 12)}...
              </code>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">Niche:</span>
              <Badge variant="secondary">{item.niche_id}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">Kind:</span>
              <Badge variant="outline">{formatKind(item.kind)}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-muted-foreground">SHA256:</span>
              <code className="text-xs bg-muted px-1.5 py-0.5 rounded font-mono">
                {item.sha256.slice(0, 16)}...
              </code>
            </div>
          </div>

          {/* Prompt */}
          <div className="space-y-1">
            <span className="font-medium text-muted-foreground">Prompt:</span>
            <p className="text-sm text-foreground line-clamp-3">{item.base_prompt}</p>
          </div>

          {/* Actions */}
          <div className="flex gap-2 pt-2 border-t">
            <Button
              variant="outline"
              size="sm"
              className="flex-1"
              onClick={handleApprove}
              disabled={isApproving}
            >
              {isApproving ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Approving...
                </>
              ) : (
                <>
                  <Check className="mr-2 h-4 w-4" />
                  Approve
                </>
              )}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              className="flex-1"
              onClick={() => setShowRejectDialog(true)}
              disabled={isRejecting}
            >
              {isRejecting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Rejecting...
                </>
              ) : (
                <>
                  <X className="mr-2 h-4 w-4" />
                  Reject
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Reject Dialog */}
        <Dialog open={showRejectDialog} onOpenChange={setShowRejectDialog}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Reject Curation Item</DialogTitle>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <p className="text-sm text-muted-foreground">
                Select a reason for rejecting this item. This will be recorded for audit purposes.
              </p>
              <Select
                value={rejectReason}
                onValueChange={(value) => setRejectReason(value as typeof rejectReason)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select reject reason" />
                </SelectTrigger>
                <SelectContent>
                  {REJECT_REASONS.map((reason) => (
                    <SelectItem key={reason} value={reason}>
                      {reason.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setShowRejectDialog(false)}>
                Cancel
              </Button>
              <Button variant="destructive" onClick={handleReject} disabled={isRejecting}>
                {isRejecting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Rejecting...
                  </>
                ) : (
                  'Confirm Reject'
                )}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        {/* Full Screen Preview Dialog */}
        <Dialog open={showPreview} onOpenChange={setShowPreview}>
          <DialogContent className="max-w-4xl max-h-[90vh] p-0">
            {item.presigned_url && (
              <img
                src={item.presigned_url}
                alt={`Full preview ${item.variant_id}`}
                className="w-full h-auto max-h-[80vh] object-contain"
              />
            )}
          </DialogContent>
        </Dialog>
      </CardContent>
    </Card>
  );
}