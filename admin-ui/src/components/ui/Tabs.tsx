'use client';

import * as React from 'react';

import { cn } from '@/lib/utils';

interface TabsProps extends React.ComponentPropsWithoutRef<'div'> {
  value?: string;
  defaultValue?: string;
  onValueChange?: ((value: string) => void) | React.Dispatch<React.SetStateAction<any>>;
}

const TabsContext = React.createContext<{ value?: string; defaultValue?: string; onValueChange?: (v: string) => void }>({});

const Tabs = React.forwardRef<HTMLDivElement, TabsProps>(
  ({ className, value, defaultValue, onValueChange, ...props }, ref) => (
    <TabsContext.Provider value={{ value, defaultValue, onValueChange }}>
      <div ref={ref} className={cn('flex flex-col', className)} {...props} />
    </TabsContext.Provider>
  )
);
Tabs.displayName = 'Tabs';

interface TabsListProps extends React.ComponentPropsWithoutRef<'div'> {}

const TabsList = React.forwardRef<HTMLDivElement, TabsListProps>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        'inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground',
        className
      )}
      {...props}
    />
  )
);
TabsList.displayName = 'TabsList';

interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value?: string;
}

const TabsTrigger = React.forwardRef<HTMLButtonElement, TabsTriggerProps>(
  ({ className, value, onClick, ...props }, ref) => {
    const ctx = React.useContext(TabsContext);
    const active = ctx.value !== undefined ? ctx.value === value : undefined;
    const handle = (e: React.MouseEvent<HTMLButtonElement>) => {
      onClick?.(e);
      if (value !== undefined) ctx.onValueChange?.(value);
    };
    return (
      <button
        ref={ref}
        value={value}
        data-state={active === undefined ? undefined : active ? 'active' : 'inactive'}
        onClick={handle}
        className={cn(
          'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm',
          className
        )}
        {...props}
      />
    );
  }
);
TabsTrigger.displayName = 'TabsTrigger';

interface TabsContentProps extends React.ComponentPropsWithoutRef<'div'> {
  value?: string;
}

const TabsContent = React.forwardRef<HTMLDivElement, TabsContentProps>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        className
      )}
      {...props}
    />
  )
);
TabsContent.displayName = 'TabsContent';

export { Tabs, TabsList, TabsTrigger, TabsContent };