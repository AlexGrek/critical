import React from "react";
import * as RadixTabs from "@radix-ui/react-tabs";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// Tabs.List
// ---------------------------------------------------------------------------

const TabsList = React.forwardRef<
  React.ElementRef<typeof RadixTabs.List>,
  React.ComponentPropsWithoutRef<typeof RadixTabs.List>
>(({ className, ...props }, ref) => (
  <RadixTabs.List
    ref={ref}
    className={cn(
      "flex gap-0 border-b border-gray-200 dark:border-gray-800",
      className
    )}
    {...props}
  />
));
TabsList.displayName = "Tabs.List";

// ---------------------------------------------------------------------------
// Tabs.Trigger
// ---------------------------------------------------------------------------

const TabsTrigger = React.forwardRef<
  React.ElementRef<typeof RadixTabs.Trigger>,
  React.ComponentPropsWithoutRef<typeof RadixTabs.Trigger>
>(({ className, ...props }, ref) => (
  <RadixTabs.Trigger
    ref={ref}
    className={cn(
      "relative px-4 py-2.5 text-sm font-medium",
      "text-gray-500 dark:text-gray-400",
      "hover:text-gray-900 dark:hover:text-gray-100 transition-colors",
      // Active indicator — a 2px bottom border that overlaps the List border
      "border-b-2 border-transparent -mb-px",
      "data-[state=active]:border-primary-500",
      "data-[state=active]:text-primary-700 dark:data-[state=active]:text-primary-300",
      "focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500",
      "disabled:pointer-events-none disabled:opacity-50",
      className
    )}
    {...props}
  />
));
TabsTrigger.displayName = "Tabs.Trigger";

// ---------------------------------------------------------------------------
// Tabs.Content
// ---------------------------------------------------------------------------

const TabsContent = React.forwardRef<
  React.ElementRef<typeof RadixTabs.Content>,
  React.ComponentPropsWithoutRef<typeof RadixTabs.Content>
>(({ className, ...props }, ref) => (
  <RadixTabs.Content
    ref={ref}
    className={cn("focus:outline-none", className)}
    {...props}
  />
));
TabsContent.displayName = "Tabs.Content";

// ---------------------------------------------------------------------------
// Namespace export
// ---------------------------------------------------------------------------

export const Tabs = {
  Root: RadixTabs.Root,
  List: TabsList,
  Trigger: TabsTrigger,
  Content: TabsContent,
};
