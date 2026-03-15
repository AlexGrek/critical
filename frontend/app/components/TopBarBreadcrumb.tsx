import { useState, useEffect, useRef, useCallback } from "react";
import { useLocation, useNavigate, Link } from "react-router";
import * as Popover from "@radix-ui/react-popover";
import { ChevronDown, Loader2, Squircle } from "lucide-react";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface ProjectBrief {
  id: string;
  name: string;
}

// Route segment labels for sub-resources inside a project
const SEGMENT_LABELS: Record<string, string> = {
  tickets: "tickets",
  settings: "settings",
  members: "members",
  pipelines: "pipelines",
};

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

/** Fetch a single project's name by ID */
function useProjectName(projectId: string | undefined) {
  const [name, setName] = useState<string | null>(null);

  useEffect(() => {
    if (!projectId) {
      setName(null);
      return;
    }
    let cancelled = false;
    fetch(`/api/v1/global/projects/${projectId}`, { credentials: "include" })
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (!cancelled && data) setName(data.name ?? projectId);
      })
      .catch(() => {
        if (!cancelled) setName(projectId);
      });
    return () => {
      cancelled = true;
    };
  }, [projectId]);

  return name;
}

/** Fetch the list of projects (for the popover) */
function useProjectList(enabled: boolean) {
  const [projects, setProjects] = useState<ProjectBrief[]>([]);
  const [loading, setLoading] = useState(false);
  const fetched = useRef(false);

  useEffect(() => {
    if (!enabled || fetched.current) return;
    fetched.current = true;
    setLoading(true);
    fetch("/api/v1/global/projects", { credentials: "include" })
      .then((r) => (r.ok ? r.json() : { items: [] }))
      .then((data) => setProjects(data.items ?? []))
      .catch(() => setProjects([]))
      .finally(() => setLoading(false));
  }, [enabled]);

  // Reset when disabled so re-opening refetches
  useEffect(() => {
    if (!enabled) fetched.current = false;
  }, [enabled]);

  return { projects, loading };
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function BreadcrumbSlash() {
  return (
    <span
      className="mx-0.5 select-none text-[0.65rem] opacity-30"
      style={{ fontFamily: "'Space Grotesk', sans-serif" }}
      aria-hidden="true"
    >
      /
    </span>
  );
}

function ProjectListPopover({
  projects,
  loading,
  currentProjectId,
  onSelect,
}: {
  projects: ProjectBrief[];
  loading: boolean;
  currentProjectId: string;
  onSelect: (id: string) => void;
}) {
  return (
    <div
      className={cn(
        "max-h-64 overflow-y-auto py-1",
        "min-w-48"
      )}
    >
      {loading ? (
        <div className="flex items-center justify-center py-4">
          <Loader2 className="w-4 h-4 animate-spin opacity-50" />
        </div>
      ) : projects.length === 0 ? (
        <div className="px-3 py-2 text-sm text-gray-500 dark:text-gray-400">
          No projects
        </div>
      ) : (
        projects.map((p) => (
          <button
            key={p.id}
            onClick={() => onSelect(p.id)}
            className={cn(
              "w-full px-3 py-1.5 text-sm text-left",
              "cursor-pointer select-none outline-none",
              "rounded-(--radius-component)",
              "transition-colors",
              p.id === currentProjectId
                ? "bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"
                : "text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700"
            )}
            data-testid={`breadcrumb-project-${p.id}`}
          >
            <span className="truncate">{p.name}</span>
          </button>
        ))
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

interface TopBarBreadcrumbProps {
  compact: boolean;
}

export function TopBarBreadcrumb({ compact }: TopBarBreadcrumbProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [popoverOpen, setPopoverOpen] = useState(false);

  // Parse route: are we inside /p/:project_id?
  const pathParts = location.pathname.split("/").filter(Boolean);
  const pIndex = pathParts.indexOf("p");
  const projectId = pIndex >= 0 ? pathParts[pIndex + 1] : undefined;

  // Segments after /p/:project_id (e.g. "tickets")
  const subSegments = projectId && pIndex + 2 < pathParts.length
    ? pathParts.slice(pIndex + 2)
    : [];

  const projectName = useProjectName(projectId);
  const { projects, loading } = useProjectList(popoverOpen);

  const handleProjectSelect = useCallback(
    (id: string) => {
      setPopoverOpen(false);
      // Navigate to same sub-path in the new project
      const sub = subSegments.length > 0 ? `/${subSegments.join("/")}` : "";
      navigate(`/p/${id}${sub}`);
    },
    [navigate, subSegments]
  );

  // Don't render if we're not in a project context
  if (!projectId) return null;

  const displayName = projectName ?? projectId;

  return (
    <nav
      className="flex items-center min-w-0 -ml-1"
      style={{ fontFamily: "'Space Grotesk', sans-serif" }}
      aria-label="Project breadcrumb"
      data-testid="topbar-breadcrumb"
    >
      {/* Project name with popover */}
      <Popover.Root open={popoverOpen} onOpenChange={setPopoverOpen}>
        <Popover.Trigger asChild>
          <button
            className={cn(
              "flex items-center gap-1 px-1.5 py-0.5",
              "rounded-(--radius-component) transition-colors cursor-pointer",
              "hover:bg-(--color-topbar-item-hover)",
              "font-medium tracking-wide",
              compact ? "text-xs" : "text-sm"
            )}
            style={{
              color: "var(--color-topbar-text)",
              fontFamily: "'Space Grotesk', sans-serif",
              fontVariant: "all-small-caps",
              letterSpacing: "0.04em",
            }}
            data-testid="breadcrumb-project-trigger"
          >
            <Squircle
              className={cn(
                "shrink-0 opacity-60",
                compact ? "w-3 h-3" : "w-4 h-4"
              )}
            />
            <span className="truncate max-w-40">{displayName}</span>
            <ChevronDown
              className={cn(
                "shrink-0 opacity-50 transition-transform",
                compact ? "w-3 h-3" : "w-3.5 h-3.5",
                popoverOpen && "rotate-180"
              )}
            />
          </button>
        </Popover.Trigger>

        <Popover.Portal>
          <Popover.Content
            align="start"
            sideOffset={8}
            className={cn(
              "z-100 p-1",
              "rounded-(--radius-component-lg)",
              "bg-white dark:bg-gray-800",
              "border border-gray-200 dark:border-gray-700",
              "shadow-lg",
              "animate-in fade-in-0 zoom-in-95"
            )}
            data-testid="breadcrumb-project-list"
          >
            <ProjectListPopover
              projects={projects}
              loading={loading}
              currentProjectId={projectId}
              onSelect={handleProjectSelect}
            />
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>

      {/* Sub-resource segments */}
      {subSegments.map((seg, i) => {
        const label = SEGMENT_LABELS[seg] ?? seg;
        // Build the link path up to this segment
        const linkPath = `/p/${projectId}/${subSegments.slice(0, i + 1).join("/")}`;

        return (
          <span key={seg} className="flex items-center min-w-0">
            <BreadcrumbSlash />
            <Link
              to={linkPath}
              className={cn(
                "px-1 py-0.5 rounded-(--radius-component)",
                "transition-colors",
                "hover:bg-(--color-topbar-item-hover)",
                "tracking-wide",
                compact ? "text-xs" : "text-sm"
              )}
              style={{
                color: "var(--color-topbar-text)",
                fontFamily: "'Space Grotesk', sans-serif",
                fontVariant: "all-small-caps",
                letterSpacing: "0.04em",
              }}
              data-testid={`breadcrumb-segment-${seg}`}
            >
              {label}
            </Link>
          </span>
        );
      })}
    </nav>
  );
}
