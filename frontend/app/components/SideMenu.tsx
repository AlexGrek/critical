import { motion, AnimatePresence } from "framer-motion";
import { Link, useLocation } from "react-router";
import {
  Home,
  LayoutDashboard,
  Folder,
  Users,
  Ticket,
  GitBranch,
  Settings,
  Key,
  UserCog,
  ClipboardList,
  BookOpen,
  MessageSquare,
  type LucideIcon,
} from "lucide-react";
import { cn } from "~/lib/utils";
import { ThemeCombobox } from "~/components/ThemeCombobox";

interface SideMenuProps {
  isOpen: boolean;
  isDesktop: boolean;
  onClose: () => void;
  topOffset: number;
}

type NavItem =
  | { icon: LucideIcon; label: string; href: string; kind: "link" }
  | { icon: LucideIcon; label: string; kind: "button" };

const navSections: Array<{ title: string; items: NavItem[] }> = [
  {
    title: "Navigate",
    items: [
      { icon: Home, label: "Home", href: "/", kind: "link" },
      { icon: LayoutDashboard, label: "Dashboard", kind: "button" },
    ],
  },
  {
    title: "Resources",
    items: [
      { icon: Folder, label: "Projects", kind: "button" },
      { icon: Users, label: "Groups", href: "/groups", kind: "link" },
      { icon: Ticket, label: "Tickets", kind: "button" },
      { icon: GitBranch, label: "Pipelines", kind: "button" },
    ],
  },
  {
    title: "Administration",
    items: [
      { icon: Settings, label: "Settings", kind: "button" },
      { icon: Key, label: "API Keys", kind: "button" },
      { icon: UserCog, label: "Team", kind: "button" },
      { icon: ClipboardList, label: "Audit Log", kind: "button" },
    ],
  },
  {
    title: "Help",
    items: [
      { icon: BookOpen, label: "Documentation", kind: "button" },
      { icon: MessageSquare, label: "Support", kind: "button" },
    ],
  },
];

const itemBase = cn(
  "w-full flex items-center gap-3 px-3 py-2 text-sm transition-colors",
  "rounded-(--radius-component)"
);

const itemResting = cn(
  "text-(--color-nav-text)",
  "hover:bg-(--color-nav-item-hover)",
  "hover:text-(--color-nav-item-hover-text)"
);

const itemActive = cn(
  "bg-(--color-nav-item-active)",
  "text-(--color-nav-item-active-text)"
);

function NavButton({
  icon: Icon,
  label,
}: {
  icon: LucideIcon;
  label: string;
}) {
  return (
    <button className={cn(itemBase, itemResting)}>
      <Icon className="w-4 h-4 shrink-0" />
      <span>{label}</span>
    </button>
  );
}

function NavLink({
  icon: Icon,
  label,
  href,
  active,
  onNavigate,
}: {
  icon: LucideIcon;
  label: string;
  href: string;
  active: boolean;
  onNavigate: () => void;
}) {
  return (
    <Link
      to={href}
      onClick={onNavigate}
      className={cn(itemBase, active ? itemActive : itemResting)}
      data-testid={`sidemenu-link-${label.toLowerCase().replace(/\s+/g, "-")}`}
    >
      <Icon className="w-4 h-4 shrink-0" />
      <span>{label}</span>
    </Link>
  );
}

export function SideMenu({ isOpen, isDesktop, onClose, topOffset }: SideMenuProps) {
  const location = useLocation();

  return (
    <>
      {/* Mobile backdrop */}
      <AnimatePresence>
        {isOpen && !isDesktop && (
          <motion.div
            key="sidemenu-backdrop"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 z-30 bg-black/40"
            onClick={onClose}
            aria-hidden="true"
            data-testid="sidemenu-backdrop"
          />
        )}
      </AnimatePresence>

      {/* Sidebar panel */}
      <motion.aside
        initial={false}
        animate={{ x: isOpen ? 0 : -280, top: topOffset }}
        transition={{ type: "spring", stiffness: 300, damping: 32 }}
        className={cn(
          "fixed left-0 bottom-0 w-64 z-40",
          "bg-(--color-nav-bg)",
          "border-r border-(--color-nav-border)",
          "flex flex-col overflow-hidden"
        )}
        aria-label="Navigation menu"
        data-testid="sidemenu"
      >
        {/* Scrollable nav area */}
        <nav className="flex-1 overflow-y-auto py-3 px-2 space-y-5">
          {navSections.map((section) => (
            <div key={section.title}>
              <div className="px-3 pb-1.5 text-xs font-semibold uppercase tracking-wider text-(--color-nav-text-muted)">
                {section.title}
              </div>
              <div className="space-y-0.5">
                {section.items.map((item) =>
                  item.kind === "link" ? (
                    <NavLink
                      key={item.label}
                      icon={item.icon}
                      label={item.label}
                      href={item.href}
                      active={location.pathname === item.href}
                      onNavigate={() => {
                        if (!isDesktop) onClose();
                      }}
                    />
                  ) : (
                    <NavButton key={item.label} icon={item.icon} label={item.label} />
                  )
                )}
              </div>
            </div>
          ))}
        </nav>

        {/* Bottom: theme picker */}
        <div className="shrink-0 border-t border-(--color-nav-border) p-3">
          <div className="px-1 pb-1.5 text-xs font-semibold uppercase tracking-wider text-(--color-nav-text-muted)">
            Theme
          </div>
          <ThemeCombobox />
        </div>
      </motion.aside>
    </>
  );
}
