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
      "fixed inset-0 z-50 bg-black/70 backdrop-blur-sm",
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
        "fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2",
        "w-[90vw] max-w-md",
        "rounded-lg border border-gray-200 bg-white shadow-xl",
        "dark:border-gray-800 dark:bg-gray-900",
        "p-6",
        "focus:outline-none",
        "data-[state=open]:animate-scale-in",
        className
      )}
      {...props}
    >
      {children}
      <Dialog.Close className="absolute top-4 right-4 rounded-sm opacity-70 ring-offset-white transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 disabled:pointer-events-none dark:ring-offset-gray-950">
        <X className="h-4 w-4 text-gray-500 dark:text-gray-400" />
        <span className="sr-only">Close</span>
      </Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
));
ModalContent.displayName = Dialog.Content.displayName;

const ModalHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div className={cn("mb-4", className)} {...props} />
);
ModalHeader.displayName = "ModalHeader";

const ModalTitle = React.forwardRef<
  React.ElementRef<typeof Dialog.Title>,
  React.ComponentPropsWithoutRef<typeof Dialog.Title>
>(({ className, ...props }, ref) => (
  <Dialog.Title
    ref={ref}
    className={cn(
      "text-xl font-bold font-sans text-gray-900 dark:text-gray-50",
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
    className={cn("text-sm text-gray-500 dark:text-gray-400", className)}
    {...props}
  />
));
ModalDescription.displayName = Dialog.Description.displayName;

const ModalFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("mt-6 flex justify-end gap-2", className)}
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
