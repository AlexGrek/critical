import React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "~/lib/utils";

const ModalOverlay = React.forwardRef<
  React.ElementRef<typeof Dialog.Overlay>,
  React.ComponentPropsWithoutRef<typeof Dialog.Overlay>
>(({ className, ...props }, ref) => (
  <Dialog.Overlay
    ref={ref}
    className={cn(
      "fixed inset-0 z-50 bg-black/40 backdrop-blur-sm",
      "data-[state=open]:animate-fade-in",
      className
    )}
    {...props}
  />
));
ModalOverlay.displayName = Dialog.Overlay.displayName;

const ModalContent = React.forwardRef<
  React.ElementRef<typeof Dialog.Content>,
  React.ComponentPropsWithoutRef<typeof Dialog.Content>
>(({ className, children, ...props }, ref) => (
  <Dialog.Portal>
    <ModalOverlay />
    <Dialog.Content
      ref={ref}
      className={cn(
        "fixed z-50 flex flex-col",
        // Always centered via transform
        "top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2",
        // Mobile: fullscreen (top:50% - translateY:50% = 0; same for left)
        "w-screen h-dvh rounded-none border-0",
        // sm+: floating dialog — numeric max-w avoids Tailwind v4 named-container-var issues
        "sm:h-auto sm:w-[90vw] sm:max-w-168 sm:max-h-[85vh]",
        "sm:rounded-(--radius-component-xl) sm:border",
        // Shared styles
        "bg-white border-gray-200 shadow-xl",
        "dark:border-gray-800 dark:bg-gray-900",
        "focus:outline-none",
        "data-[state=open]:animate-scale-in",
        className
      )}
      {...props}
    >
      {children}
    </Dialog.Content>
  </Dialog.Portal>
));
ModalContent.displayName = Dialog.Content.displayName;

const ModalHeader = ({
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
    <Dialog.Close className="shrink-0 mt-0.5 p-1.5 rounded-(--radius-component) opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 ring-offset-white dark:ring-offset-gray-950 disabled:cursor-not-allowed disabled:opacity-50 disabled:pointer-events-none">
      <X className="h-4 w-4 text-gray-500 dark:text-gray-400" />
      <span className="sr-only">Close</span>
    </Dialog.Close>
  </div>
);
ModalHeader.displayName = "ModalHeader";

const ModalTitle = React.forwardRef<
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
ModalTitle.displayName = Dialog.Title.displayName;

const ModalDescription = React.forwardRef<
  React.ElementRef<typeof Dialog.Description>,
  React.ComponentPropsWithoutRef<typeof Dialog.Description>
>(({ className, ...props }, ref) => (
  <Dialog.Description
    ref={ref}
    className={cn("text-xs text-gray-600 dark:text-gray-400", className)}
    {...props}
  />
));
ModalDescription.displayName = Dialog.Description.displayName;

const ModalFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex gap-2 p-4 border-t border-gray-200 dark:border-gray-800 shrink-0", className)}
    {...props}
  />
);
ModalFooter.displayName = "ModalFooter";

/**
 * A modal dialog component built with Radix UI.
 * @example
 * <Modal.Root>
 *   <Modal.Trigger asChild>
 *     <Button>Open Modal</Button>
 *   </Modal.Trigger>
 *   <Modal.Content>
 *     <Modal.Header>
 *       <Modal.Title>Modal Title</Modal.Title>
 *       <Modal.Description>Modal description</Modal.Description>
 *     </Modal.Header>
 *     <p>Modal body content...</p>
 *     <Modal.Footer>
 *       <Button variant="outline">Cancel</Button>
 *       <Button>Confirm</Button>
 *     </Modal.Footer>
 *   </Modal.Content>
 * </Modal.Root>
 */
export const Modal = {
  Root: Dialog.Root,
  Trigger: Dialog.Trigger,
  Content: ModalContent,
  Header: ModalHeader,
  Title: ModalTitle,
  Description: ModalDescription,
  Footer: ModalFooter,
  Close: Dialog.Close,
};
