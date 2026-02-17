import {
  useRef,
  useState,
  useEffect,
  type ReactNode,
  type MouseEvent,
  cloneElement,
  isValidElement,
} from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";
import { cn } from "~/lib/utils";

interface MorphModalProps {
  trigger: ReactNode;
  children: ReactNode | ((close: () => void) => ReactNode);
  modalWidth?: number;
  modalHeight?: number;
  className?: string;
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

/**
 * A morphing modal that animates from the trigger element to a centered modal.
 * Uses Framer Motion for smooth transitions.
 *
 * Can be used in controlled or uncontrolled mode:
 * - Uncontrolled: Just pass trigger and children, modal manages its own state
 * - Controlled: Pass isOpen and onOpenChange to control from parent
 *
 * Children can be ReactNode or a render function that receives a close callback.
 */
export default function MorphModal({
  trigger,
  children,
  modalWidth = 500,
  modalHeight = 400,
  className,
  isOpen: controlledIsOpen,
  onOpenChange,
}: MorphModalProps) {
  const triggerRef = useRef<HTMLDivElement>(null);
  const [originRect, setOriginRect] = useState<DOMRect | null>(null);
  const [internalIsOpen, setInternalIsOpen] = useState(false);
  const [mounted, setMounted] = useState(false);

  // Use controlled state if provided, otherwise use internal state
  const isOpen = controlledIsOpen ?? internalIsOpen;
  const setIsOpen = onOpenChange ?? setInternalIsOpen;

  useEffect(() => {
    setMounted(typeof document !== "undefined");
  }, []);

  const openModal = (e: MouseEvent) => {
    e.stopPropagation();
    if (triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      setOriginRect(rect);
      setIsOpen(true);
    }
  };

  const closeModal = () => setIsOpen(false);

  if (!mounted) return null;

  return (
    <>
      <div
        ref={triggerRef}
        onClick={openModal}
        className="inline-block cursor-pointer"
      >
        <motion.div
          animate={{ opacity: isOpen ? 0 : 1, scale: isOpen ? 0.95 : 1 }}
          transition={{ duration: 0.2 }}
        >
          {isValidElement(trigger) ? cloneElement(trigger) : trigger}
        </motion.div>
      </div>

      {createPortal(
        <AnimatePresence>
          {isOpen && originRect && (
            <>
              {/* Backdrop */}
              <motion.div
                className="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                onClick={closeModal}
              />

              {/* Morphing Modal */}
              <motion.div
                className={cn(
                  "fixed z-50 rounded-(--radius-component-xl) overflow-hidden shadow-2xl",
                  "bg-white dark:bg-gray-900",
                  "border border-gray-200 dark:border-gray-800",
                  className
                )}
                initial={{
                  top: originRect.top,
                  left: originRect.left,
                  width: originRect.width,
                  height: originRect.height,
                  position: "fixed" as const,
                }}
                animate={{
                  top: "50%",
                  left: "50%",
                  x: "-50%",
                  y: "-50%",
                  width: modalWidth,
                  height: modalHeight,
                }}
                exit={{
                  top: originRect.top,
                  left: originRect.left,
                  x: 0,
                  y: 0,
                  width: originRect.width,
                  height: originRect.height,
                  opacity: 0,
                }}
                transition={{
                  type: "tween",
                  duration: 0.25,
                  ease: [0.4, 0, 0.2, 1],
                }}
                onClick={(e) => e.stopPropagation()}
              >
                <motion.div
                  className="h-full w-full p-6"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ delay: 0.1 }}
                >
                  {typeof children === "function"
                    ? children(closeModal)
                    : children}
                </motion.div>
              </motion.div>
            </>
          )}
        </AnimatePresence>,
        document.body
      )}
    </>
  );
}

export { MorphModal };
