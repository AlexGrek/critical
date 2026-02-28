import { motion } from "framer-motion";
import { User } from "lucide-react";
import { cn } from "~/lib/utils";

interface TopBarProps {
  isOpen: boolean;
  onToggle: () => void;
}

/** Animated logo that uses topbar CSS variables — acts as the hamburger toggle */
function TopBarLogo({ onToggle }: { onToggle: () => void }) {
  return (
    <motion.button
      className={cn(
        "font-mono font-bold tracking-tighter flex items-center cursor-pointer select-none",
        "text-xl px-2 py-1.5 rounded-(--radius-component) transition-colors",
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
        style={{ color: "var(--color-topbar-text)" }}
        variants={{ rest: { x: 0 }, hover: { x: -4 } }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        {"{"}
      </motion.span>
      <motion.span
        className="text-red-500"
        variants={{ rest: { scale: 1 }, hover: { scale: 1.1 } }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        !
      </motion.span>
      <motion.span
        style={{ color: "var(--color-topbar-text)" }}
        variants={{ rest: { x: 0 }, hover: { x: 4 } }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        {"}"}
      </motion.span>
    </motion.button>
  );
}

export function TopBar({ isOpen: _, onToggle }: TopBarProps) {
  return (
    <header
      className={cn(
        "fixed top-0 left-0 right-0 h-14 z-50",
        "flex items-center px-3 gap-2",
        "bg-(--color-topbar-bg)",
        "border-b border-(--color-nav-border)"
      )}
      data-testid="topbar"
    >
      <TopBarLogo onToggle={onToggle} />

      <div className="flex-1" />

      {/* User account button — placeholder, filled later */}
      <button
        className={cn(
          "flex items-center justify-center p-2",
          "rounded-(--radius-component) transition-colors",
          "hover:bg-(--color-topbar-item-hover)"
        )}
        style={{ color: "var(--color-topbar-text)" }}
        aria-label="User account"
        data-testid="topbar-user-button"
      >
        <User className="w-5 h-5" />
      </button>
    </header>
  );
}
