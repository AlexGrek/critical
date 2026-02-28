import { motion } from "framer-motion";
import { User } from "lucide-react";
import { cn } from "~/lib/utils";

interface TopBarProps {
  isOpen: boolean;
  onToggle: () => void;
  scrolled: boolean;
}

function TopBarLogo({ onToggle, compact }: { onToggle: () => void; compact: boolean }) {
  return (
    <motion.button
      className={cn(
        "font-mono font-bold tracking-tighter flex items-center cursor-pointer select-none",
        "px-2 py-1.5 rounded-(--radius-component) transition-colors",
        "hover:bg-(--color-topbar-item-hover)"
      )}
      style={{ color: "var(--color-topbar-text)" }}
      onClick={onToggle}
      aria-label="Toggle navigation menu"
      initial="rest"
      whileHover="hover"
      variants={{ rest: {}, hover: {} }}
      data-testid="topbar-logo-toggle"
    >
      <motion.span
        animate={{ fontSize: compact ? "0.875rem" : "1.25rem" }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        style={{ color: "var(--color-topbar-text)" }}
        variants={{ rest: { x: 0 }, hover: { x: -4 } }}
      >
        {"{"}
      </motion.span>
      <motion.span
        animate={{ fontSize: compact ? "0.875rem" : "1.25rem" }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        className="text-red-500"
        variants={{ rest: { scale: 1 }, hover: { scale: 1.1 } }}
      >
        !
      </motion.span>
      <motion.span
        animate={{ fontSize: compact ? "0.875rem" : "1.25rem" }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        style={{ color: "var(--color-topbar-text)" }}
        variants={{ rest: { x: 0 }, hover: { x: 4 } }}
      >
        {"}"}
      </motion.span>
    </motion.button>
  );
}

export function TopBar({ isOpen: _, onToggle, scrolled }: TopBarProps) {
  return (
    <motion.header
      animate={{ height: scrolled ? 40 : 56 }}
      transition={{ duration: 0.2, ease: "easeInOut" }}
      className={cn(
        "fixed top-0 left-0 right-0 z-50",
        "flex items-center px-3 gap-2",
        "bg-(--color-topbar-bg)",
        scrolled
          ? "border-b border-(--color-nav-border) shadow-sm"
          : "border-b border-transparent"
      )}
      data-testid="topbar"
    >
      <TopBarLogo onToggle={onToggle} compact={scrolled} />

      <div className="flex-1" />

      <motion.button
        animate={{ padding: scrolled ? "0.375rem" : "0.5rem" }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        className={cn(
          "flex items-center justify-center",
          "rounded-(--radius-component) transition-colors",
          "hover:bg-(--color-topbar-item-hover)"
        )}
        style={{ color: "var(--color-topbar-text)" }}
        aria-label="User account"
        data-testid="topbar-user-button"
      >
        <motion.div
          animate={{ width: scrolled ? "1rem" : "1.25rem", height: scrolled ? "1rem" : "1.25rem" }}
          transition={{ duration: 0.2, ease: "easeInOut" }}
        >
          <User className="w-full h-full" />
        </motion.div>
      </motion.button>
    </motion.header>
  );
}
