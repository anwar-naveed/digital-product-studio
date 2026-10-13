'use client';

import * as React from 'react';

import { cn } from '@/lib/utils';
import { cva } from 'class-variance-authority';

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground hover:bg-primary/80',
        secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive: 'border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80',
        outline: 'text-foreground',
        success: 'border-transparent bg-green-500 text-white hover:bg-green-600',
        warning: 'border-transparent bg-yellow-500 text-white hover:bg-yellow-600',
        info: 'border-transparent bg-blue-500 text-white hover:bg-blue-600',
        // Listing state badges (G3 - 8 states)
        draft: 'border-transparent bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
        publishing: 'border-transparent bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 animate-pulse',
        live: 'border-transparent bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
        updating: 'border-transparent bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 animate-pulse',
        failed: 'border-transparent bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
        suspended: 'border-transparent bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
        delisting: 'border-transparent bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 animate-pulse',
        delisted: 'border-transparent bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400',
      },
      size: {
        default: 'px-2.5 py-0.5 text-xs',
        sm: 'px-2 py-0.5 text-[10px]',
        lg: 'px-3 py-1 text-sm',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement> {
  variant?: string | null;
  size?: 'default' | 'sm' | 'lg' | null;
}

export const Badge = React.forwardRef<HTMLDivElement, BadgeProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(badgeVariants({ variant: (variant ?? 'default') as any, size }), className)}
        {...props}
      />
    );
  }
);
Badge.displayName = 'Badge';