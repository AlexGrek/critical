import React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { cva, type VariantProps } from "class-variance-authority";
import { X } from "lucide-react";
import { cn } from "~/lib/utils";

const DrawerOverlay = React.forwardRef<
  React.ElementRef<typeof Dialog.Overlay>,
  React.ComponentPropsWithoutRef<typeof Dialog.Overlay>
>(({ className, ...props }, ref) => (
  <Dialog.Overlay
    ref={ref}
    className={cn(
      "drawer-overlay fixed inset-0 z-50 bg-black/40 backdrop-blur-sm",
      className
    )}
    {...props}
  />
));
DrawerOverlay.displayName = Dialog.Overlay.displayName;

const drawerContentVariants = cva(
  "fixed z-50 flex flex-col bg-white border-gray-200 dark:border-gray-800 dark:bg-gray-900 shadow-xl focus:outline-none",
  {
    variants: {
      direction: {
        right:
          "drawer-content-right right-0 top-0 h-screen w-full sm:max-w-[640px] border-l",
        left:
          "drawer-content-left left-0 top-0 h-screen w-full sm:max-w-[640px] border-r",
        top:
          "drawer-content-top top-0 left-0 right-0 w-screen h-auto max-h-[80vh] border-b rounded-b-(--radius-component-lg)",
        bottom:
          "drawer-content-bottom bottom-0 left-0 right-0 w-screen h-auto max-h-[80vh] border-t rounded-t-(--radius-component-lg)",
      },
    },
    defaultVariants: {
      direction: "right",
    },
  }
);

interface DrawerContentProps
  extends React.ComponentPropsWithoutRef<typeof Dialog.Content>,
    VariantProps<typeof drawerContentVariants> {}

const DrawerContent = React.forwardRef<
  React.ElementRef<typeof Dialog.Content>,
  DrawerContentProps
>(({ className, direction, children, ...props }, ref) => (
  <Dialog.Portal>
    <DrawerOverlay />
    <Dialog.Content
      ref={ref}
      className={cn(drawerContentVariants({ direction }), className)}
      {...props}
    >
      {children}
    </Dialog.Content>
  </Dialog.Portal>
));
DrawerContent.displayName = Dialog.Content.displayName;

const DrawerHeader = ({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex items-start justify-between gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-800 shrink-0",
      className
    )}
    {...props}
  >
    <div className="flex flex-col gap-0.5 min-w-0">{children}</div>
    <Dialog.Close
      className="shrink-0 mt-0.5 p-1.5 rounded-(--radius-component) opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 ring-offset-white dark:ring-offset-gray-950 disabled:cursor-not-allowed disabled:opacity-50 disabled:pointer-events-none"
      data-testid="drawer-close"
    >
      <X className="h-4 w-4 text-gray-500 dark:text-gray-400" />
      <span className="sr-only">Close drawer</span>
    </Dialog.Close>
  </div>
);
DrawerHeader.displayName = "DrawerHeader";

const DrawerTitle = React.forwardRef<
  React.ElementRef<typeof Dialog.Title>,
  React.ComponentPropsWithoutRef<typeof Dialog.Title>
>(({ className, ...props }, ref) => (
  <Dialog.Title
    ref={ref}
    className={cn(
      "text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100",
      className
    )}
    {...props}
  />
));
DrawerTitle.displayName = Dialog.Title.displayName;

const DrawerDescription = React.forwardRef<
  React.ElementRef<typeof Dialog.Description>,
  React.ComponentPropsWithoutRef<typeof Dialog.Description>
>(({ className, ...props }, ref) => (
  <Dialog.Description
    ref={ref}
    className={cn("text-xs text-gray-600 dark:text-gray-400", className)}
    {...props}
  />
));
DrawerDescription.displayName = Dialog.Description.displayName;

const DrawerBody = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex-1 overflow-y-auto px-4 py-4", className)}
    {...props}
  />
));
DrawerBody.displayName = "DrawerBody";

const DrawerFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex gap-2 p-4 border-t border-gray-200 dark:border-gray-800 shrink-0",
      className
    )}
    {...props}
  />
);
DrawerFooter.displayName = "DrawerFooter";

/**
 * A slide-out drawer component with support for 4 directions (left, right, top, bottom).
 * Built with Radix UI and theme-aware styling.
 * @example
 * <Drawer.Root>
 *   <Drawer.Trigger asChild>
 *     <Button>Open Drawer</Button>
 *   </Drawer.Trigger>
 *   <Drawer.Content direction="right">
 *     <Drawer.Header>
 *       <Drawer.Title>Drawer Title</Drawer.Title>
 *       <Drawer.Description>Optional description</Drawer.Description>
 *     </Drawer.Header>
 *     <Drawer.Body>
 *       Content goes here...
 *     </Drawer.Body>
 *     <Drawer.Footer>
 *       <Button variant="outline">Cancel</Button>
 *       <Button>Save</Button>
 *     </Drawer.Footer>
 *   </Drawer.Content>
 * </Drawer.Root>
 */
export const Drawer = {
  Root: Dialog.Root,
  Trigger: Dialog.Trigger,
  Content: DrawerContent,
  Header: DrawerHeader,
  Title: DrawerTitle,
  Description: DrawerDescription,
  Body: DrawerBody,
  Footer: DrawerFooter,
  Close: Dialog.Close,
};
