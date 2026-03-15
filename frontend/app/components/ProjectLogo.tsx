import { useEffect, useState } from "react";
import { useLocation } from "react-router";
import { Squircle } from "lucide-react";
import { cn } from "~/lib/utils";

interface ProjectLogoProps {
  size?: "sm" | "md" | "lg";
  className?: string;
  "data-testid"?: string;
}

/**
 * Displays the current project context with a squircle icon + name.
 * Auto-detects project from URL. Returns null if not in a project route.
 */
export function ProjectLogo({
  size = "md",
  className,
  "data-testid": testId,
}: ProjectLogoProps) {
  const location = useLocation();
  const [projectName, setProjectName] = useState<string | null>(null);

  // Parse route to get project ID
  const pathParts = location.pathname.split("/").filter(Boolean);
  const pIndex = pathParts.indexOf("p");
  const projectId = pIndex >= 0 ? pathParts[pIndex + 1] : undefined;

  // Fetch project name
  useEffect(() => {
    if (!projectId) {
      setProjectName(null);
      return;
    }
    let cancelled = false;
    fetch(`/api/v1/global/projects/${projectId}`, { credentials: "include" })
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (!cancelled && data) setProjectName(data.name ?? projectId);
      })
      .catch(() => {
        if (!cancelled) setProjectName(projectId);
      });
    return () => {
      cancelled = true;
    };
  }, [projectId]);

  if (!projectId || !projectName) return null;

  const sizeClasses = {
    sm: "gap-2",
    md: "gap-2",
    lg: "gap-3",
  };

  const iconSizes = {
    sm: "w-3.5 h-3.5",
    md: "w-4 h-4",
    lg: "w-5 h-5",
  };

  const textSizes = {
    sm: "text-xs",
    md: "text-sm",
    lg: "text-base",
  };

  return (
    <div
      className={cn(
        "flex items-center",
        sizeClasses[size],
        className
      )}
      data-testid={testId}
    >
      <Squircle className={cn("shrink-0 opacity-60", iconSizes[size])} />
      <span className={cn("font-medium truncate", textSizes[size])}>
        {projectName}
      </span>
    </div>
  );
}
