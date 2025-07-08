
import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { cva, type VariantProps } from 'class-variance-authority';
import { X } from 'lucide-react';

const ModalContent = React.forwardRef<
    React.ElementRef<typeof Dialog.Content>,
    React.ComponentPropsWithoutRef<typeof Dialog.Content>
>(({ className, children, ...props }, ref) => (
    <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm" />
        <Dialog.Content
            ref={ref}
            className={`
        fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2
        w-[90vw] max-w-md
        border border-gray-800 bg-black
        p-6 shadow-lg
        focus:outline-none
        ${className}
      `}
            {...props}
        >
            {children}
            <Dialog.Close className="absolute top-4 right-4 text-gray-500 hover:text-white transition-colors">
                <X className="h-4 w-4" />
                <span className="sr-only">Close</span>
            </Dialog.Close>
        </Dialog.Content>
    </Dialog.Portal>
));
ModalContent.displayName = Dialog.Content.displayName;

/**
 * A modal dialog component built with Radix UI.
 * @example
 * <Modal>
 * <Modal.Trigger asChild>
 * <Button>Open Modal</Button>
 * </Modal.Trigger>
 * <Modal.Content>
 * <Modal.Header>Modal Title</Modal.Header>
 * <p>Modal body content...</p>
 * <Modal.Footer>
 * <Button appearance="subtle">Cancel</Button>
 * <Button>Confirm</Button>
 * </Modal.Footer>
 * </Modal.Content>
 * </Modal>
 */
export const Modal = {
    Root: Dialog.Root,
    Trigger: Dialog.Trigger,
    Content: ModalContent,
    Header: ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
        <div className={`mb-4 ${className}`} {...props}>
            <Dialog.Title className="text-xl font-bold font-mono text-white" {...props} />
        </div>
    ),
    Footer: ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
        <div className={`mt-6 flex justify-end space-x-2 ${className}`} {...props} />
    ),
};

