// MorphModal.tsx
import {
    useRef,
    useState,
    useEffect,
    type ReactNode,
    type MouseEvent,
    cloneElement,
    isValidElement,
} from 'react';
import { createPortal } from 'react-dom';
import { motion, AnimatePresence } from 'framer-motion';

interface MorphModalProps {
    trigger: ReactNode;
    children: ReactNode;
    modalWidth?: number;
    modalHeight?: number;
}

export default function MorphModal({
    trigger,
    children,
    modalWidth = 400,
    modalHeight = 300,
}: MorphModalProps) {
    const triggerRef = useRef<HTMLDivElement>(null);
    const [originRect, setOriginRect] = useState<DOMRect | null>(null);
    const [isOpen, setIsOpen] = useState(false);
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(typeof document !== 'undefined');
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
                    animate={{ opacity: isOpen ? 0 : 1, scale: isOpen ? 0.9 : 1 }}
                    transition={{ duration: 0.2 }}
                >
                    {isValidElement(trigger) ? cloneElement(trigger) : trigger}
                </motion.div>
            </div>

            {createPortal(
                <AnimatePresence>
                    {isOpen && originRect && (
                        <>
                            <motion.div
                                className="fixed inset-0 z-40 bg-black/50 backdrop-blur-md"
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                exit={{ opacity: 0 }}
                                onClick={closeModal}
                            />

                            <motion.div
                                className="fixed z-50 rounded-xl bg-white overflow-hidden shadow-xl"
                                initial={{
                                    top: originRect.top,
                                    left: originRect.left,
                                    width: originRect.width,
                                    height: originRect.height,
                                    position: 'absolute',
                                }}
                                animate={{
                                    top: '50%',
                                    left: '50%',
                                    x: '-50%',
                                    y: '-50%',
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
                                }}
                                transition={{ type: 'spring', stiffness: 300, damping: 25 }}
                                onClick={(e) => e.stopPropagation()}
                            >
                                {children}
                            </motion.div>
                        </>
                    )}
                </AnimatePresence>,
                document.body
            )}
        </>
    );
}
