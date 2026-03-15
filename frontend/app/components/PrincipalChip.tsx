import * as Popover from "@radix-ui/react-popover";
import { useRef, useState } from "react";
import { User2, Users, Cpu, GitBranch } from "lucide-react";
import { principalAvatarUrl, principalInitials } from "~/lib/principals";
import type { PrincipalInfo } from "~/lib/principals";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

const TYPE_BG: Record<string, string> = {
  user: "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
  group:
    "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
  service_account:
    "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  pipeline_account:
    "bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-300",
};

type Size = "xs" | "sm" | "md";

const AVATAR_DIM: Record<Size, string> = {
  xs: "w-4 h-4",
  sm: "w-5 h-5",
  md: "w-7 h-7",
};

const TEXT_SIZE: Record<Size, string> = {
  xs: "text-xs",
  sm: "text-xs",
  md: "text-sm",
};

const INITIALS_TEXT: Record<Size, string> = {
  xs: "text-[8px]",
  sm: "text-[9px]",
  md: "text-xs",
};

function TypeIcon({ type, className }: { type: string; className?: string }) {
  switch (type) {
    case "group":
      return <Users className={className} />;
    case "service_account":
      return <Cpu className={className} />;
    case "pipeline_account":
      return <GitBranch className={className} />;
    default:
      return <User2 className={className} />;
  }
}

// ---------------------------------------------------------------------------
// Avatar — used in both the chip and the popup (at different sizes)
// ---------------------------------------------------------------------------

function Avatar({
  info,
  id,
  dimClass,
  initialsClass,
}: {
  info: PrincipalInfo | undefined;
  id: string;
  dimClass: string;
  initialsClass: string;
}) {
  if (!info || info.error) {
    return (
      <span
        className={cn(
          "rounded-(--radius-component) bg-gray-200 dark:bg-gray-700 flex items-center justify-center shrink-0",
          dimClass
        )}
      >
        <User2 className="w-2.5 h-2.5 text-gray-400 dark:text-gray-500" />
      </span>
    );
  }

  const avatarUrl = principalAvatarUrl(info.type, info.avatar_ulid);
  const initials = info.name?.trim() ? principalInitials(info.name) : null;
  const colorClass =
    TYPE_BG[info.type] ??
    "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300";

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={info.name || id}
        className={cn(
          "rounded-(--radius-component) object-cover shrink-0",
          dimClass
        )}
      />
    );
  }

  return (
    <span
      className={cn(
        "rounded-(--radius-component) flex items-center justify-center font-medium shrink-0",
        dimClass,
        colorClass,
        initialsClass
      )}
    >
      {initials ?? <TypeIcon type={info.type} className="w-2.5 h-2.5" />}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Popup card content
// ---------------------------------------------------------------------------

function PopupCard({
  id,
  info,
  onEnter,
  onLeave,
}: {
  id: string;
  info: PrincipalInfo | undefined;
  onEnter: () => void;
  onLeave: () => void;
}) {
  const displayName = info && !info.error && info.name?.trim() ? info.name : null;

  return (
    <div
      className="flex items-center gap-3 p-3 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-(--radius-component-lg) shadow-lg min-w-0 max-w-64"
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
    >
      {/* Larger avatar — still a thumbnail, just bigger */}
      <Avatar
        info={info}
        id={id}
        dimClass="w-10 h-10 shrink-0"
        initialsClass="text-sm"
      />
      <div className="min-w-0 flex flex-col gap-0.5">
        <span className="font-medium text-sm text-gray-900 dark:text-gray-100 truncate">
          {displayName ?? id}
        </span>
        {/* Always show the ID in mono so power users can copy it */}
        <span className="font-mono text-xs text-gray-400 dark:text-gray-500 truncate">
          {id}
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// PrincipalChip
// ---------------------------------------------------------------------------

export interface PrincipalChipProps {
  /** The principal ID, e.g. "u_alice". */
  id: string;
  /** Resolved info from the principals/resolve endpoint. */
  info?: PrincipalInfo;
  /** Size variant — default "sm". */
  size?: Size;
  className?: string;
  "data-testid"?: string;
}

/**
 * Displays a principal (user/group/service-account/pipeline-account) as an
 * inline chip with avatar and display name.
 *
 * - Avatars use `rounded-(--radius-component)` — inherits theme roundness.
 * - Hover (desktop) or tap (mobile) opens a small popover with a larger
 *   avatar, display name, and raw ID.
 * - Falls back to the raw ID when info is not yet resolved or empty.
 */
export function PrincipalChip({
  id,
  info,
  size = "sm",
  className,
  "data-testid": testId,
}: PrincipalChipProps) {
  const [open, setOpen] = useState(false);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const scheduleClose = () => {
    closeTimer.current = setTimeout(() => setOpen(false), 120);
  };
  const cancelClose = () => {
    if (closeTimer.current) clearTimeout(closeTimer.current);
  };
  const handleOpenTrue = () => {
    cancelClose();
    setOpen(true);
  };
  const handleToggle = () => {
    cancelClose();
    setOpen((v) => !v);
  };

  const avatarDim = AVATAR_DIM[size];
  const textSize = TEXT_SIZE[size];
  const initialsText = INITIALS_TEXT[size];

  const displayName =
    info && !info.error && info.name?.trim() ? info.name : null;

  const chipInner = (
    <>
      <Avatar info={info} id={id} dimClass={avatarDim} initialsClass={initialsText} />
      <span
        className={cn(
          info && !info.error
            ? "text-gray-800 dark:text-gray-200"
            : "text-gray-500 dark:text-gray-400 font-mono",
          textSize
        )}
      >
        {displayName ?? id}
      </span>
    </>
  );

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger asChild>
        <span
          role="button"
          tabIndex={0}
          className={cn(
            "inline-flex items-center gap-1.5 cursor-pointer select-none",
            className
          )}
          data-testid={testId}
          onMouseEnter={handleOpenTrue}
          onMouseLeave={scheduleClose}
          onClick={handleToggle}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") handleToggle();
          }}
        >
          {chipInner}
        </span>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content
          side="top"
          align="start"
          sideOffset={6}
          // Prevent Radix from closing on outside pointer events while hover is active
          onOpenAutoFocus={(e) => e.preventDefault()}
          style={{ zIndex: 9999 }}
        >
          <PopupCard
            id={id}
            info={info}
            onEnter={handleOpenTrue}
            onLeave={scheduleClose}
          />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
