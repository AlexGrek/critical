import { motion } from "framer-motion";
import { LogIn, LogOut, Settings, User } from "lucide-react";
import { Link, useNavigate } from "react-router";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { cn } from "~/lib/utils";
import type { AuthUser } from "~/layouts/app-layout";

interface TopBarProps {
  isOpen: boolean;
  onToggle: () => void;
  scrolled: boolean;
  isAuthenticated: boolean;
  user: AuthUser | null;
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

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length >= 2) return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  return name.slice(0, 2).toUpperCase();
}

function UserAvatar({ user, size }: { user: AuthUser | null; size: number }) {
  if (user?.avatarUrl) {
    return (
      <img
        src={user.avatarUrl}
        alt={user.name}
        className="rounded-(--radius-component) object-cover"
        style={{ width: size, height: size }}
        data-testid="topbar-avatar-img"
      />
    );
  }

  if (user?.name) {
    return (
      <div
        className={cn(
          "rounded-(--radius-component) flex items-center justify-center",
          "bg-primary-600 text-white font-medium select-none"
        )}
        style={{ width: size, height: size, fontSize: size * 0.4 }}
        data-testid="topbar-avatar-initials"
      >
        {getInitials(user.name)}
      </div>
    );
  }

  return <User style={{ width: size, height: size }} />;
}

const menuItemClasses = cn(
  "flex items-center gap-2 px-3 py-2 text-sm cursor-pointer select-none",
  "rounded-(--radius-component) outline-none",
  "text-gray-700 dark:text-gray-200",
  "data-highlighted:bg-gray-100 dark:data-highlighted:bg-gray-700"
);

function UserMenu({ user, scrolled }: { user: AuthUser | null; scrolled: boolean }) {
  const navigate = useNavigate();
  const avatarSize = scrolled ? 24 : 32;

  async function handleLogout() {
    await fetch("/api/v1/logout", { method: "POST", credentials: "include" });
    navigate("/sign-in");
  }

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          className={cn(
            "flex items-center justify-center",
            "rounded-(--radius-component) transition-colors cursor-pointer",
            "hover:ring-2 hover:ring-primary-500/40"
          )}
          aria-label="User menu"
          data-testid="topbar-user-button"
        >
          <UserAvatar user={user} size={avatarSize} />
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="end"
          sideOffset={8}
          className={cn(
            "z-100 min-w-48 p-1",
            "rounded-(--radius-component-lg)",
            "bg-white dark:bg-gray-800",
            "border border-gray-200 dark:border-gray-700",
            "shadow-lg",
            "animate-in fade-in-0 zoom-in-95"
          )}
          data-testid="topbar-user-menu"
        >
          {user && (
            <>
              <div className="px-3 py-2 border-b border-gray-100 dark:border-gray-700">
                <div className="text-sm font-medium text-gray-900 dark:text-gray-100" data-testid="topbar-user-menu-name">
                  {user.name}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  {user.id}
                </div>
              </div>
            </>
          )}

          <DropdownMenu.Item
            className={menuItemClasses}
            onSelect={() => navigate("/settings")}
            data-testid="topbar-menu-settings"
          >
            <Settings className="w-4 h-4" />
            Settings
          </DropdownMenu.Item>

          <DropdownMenu.Separator className="my-1 h-px bg-gray-100 dark:bg-gray-700" />

          <DropdownMenu.Item
            className={cn(menuItemClasses, "text-red-600 dark:text-red-400 data-highlighted:bg-red-50 dark:data-highlighted:bg-red-900/20")}
            onSelect={handleLogout}
            data-testid="topbar-menu-logout"
          >
            <LogOut className="w-4 h-4" />
            Log out
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

export function TopBar({ isOpen: _, onToggle, scrolled, isAuthenticated, user }: TopBarProps) {
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

      {isAuthenticated ? (
        <UserMenu user={user} scrolled={scrolled} />
      ) : (
        <Link
          to="/sign-in"
          className={cn(
            "flex items-center gap-1.5",
            "px-3 py-1.5 text-sm font-medium",
            "rounded-(--radius-component) transition-colors",
            "hover:bg-(--color-topbar-item-hover)"
          )}
          style={{ color: "var(--color-topbar-text)" }}
          data-testid="topbar-sign-in-button"
        >
          <LogIn className="w-4 h-4" />
          <span className="hidden sm:inline">Sign in</span>
        </Link>
      )}
    </motion.header>
  );
}
