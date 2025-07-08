import * as Dialog from '@radix-ui/react-dialog';
import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

export function MorphingModal() {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [originRect, setOriginRect] = useState<DOMRect | null>(null);
  const [open, setOpen] = useState(false);

  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen && triggerRef.current) {
      setOriginRect(triggerRef.current.getBoundingClientRect());
    }
    setOpen(isOpen);
  };

  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) return null;

  return (
    <Dialog.Root open={open} onOpenChange={handleOpenChange}>
      <Dialog.Trigger asChild>
        <button ref={triggerRef} className="trigger-button">
          Open Modal
        </button>
      </Dialog.Trigger>

      {createPortal(
        <AnimatePresence>
          {open && originRect && (
            <Dialog.Portal forceMount>
              <Dialog.Overlay asChild>
                <motion.div
                  className="modal-backdrop"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                />
              </Dialog.Overlay>

              <Dialog.Content asChild>
                <motion.div
                  className="modal-content bg-black/50 backdrop-blur-md"
                  initial={{
                    position: 'absolute',
                    top: originRect.top,
                    left: originRect.left,
                    width: originRect.width,
                    height: originRect.height,
                  }}
                  animate={{
                    top: '50%',
                    left: '50%',
                    x: '-50%',
                    y: '-50%',
                    width: 400,
                    height: 300,
                  }}
                  exit={{
                    top: originRect.top,
                    left: originRect.left,
                    width: originRect.width,
                    height: originRect.height,
                    x: 0,
                    y: 0,
                  }}
                  transition={{ duration: 1 }}
                >
                  <Dialog.Title className="modal-title">Modal Title</Dialog.Title>
                  <Dialog.Description className="modal-description">
                    Smooth morph from button to modal.
                  </Dialog.Description>
                  <button
                    onClick={() => setOpen(false)}
                    className="close-button"
                  >
                    Close
                  </button>
                </motion.div>
              </Dialog.Content>
            </Dialog.Portal>
          )}
        </AnimatePresence>,
        document.body
      )}
    </Dialog.Root>
  );
}

export default MorphingModal;