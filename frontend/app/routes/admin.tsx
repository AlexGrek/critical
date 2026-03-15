import { useState, useEffect, useCallback, useMemo } from "react";
import type { Route } from "./+types/admin";
import {
  Header,
  Paragraph,
  Button,
  Table,
  AppMenu,
  PrincipalChip,
} from "~/components";
import type { AppItem } from "~/components";
import { formatRelativeTime } from "~/lib/utils";
import { usePrincipals } from "~/lib/usePrincipals";
import { Activity, History, Users, Settings } from "lucide-react";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Admin Area - Critical" },
    { name: "description", content: "Administrator dashboard and controls" },
  ];
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type EventKind =
  | "Error"
  | "Background"
  | "EntityManagement"
  | "Server"
  | "Stats"
  | "Database"
  | "ObjectStorage";

type EventPriority = "Expected" | "Important" | "Note" | "Minor" | "Lifecycle";

interface SystemEvent {
  _key: string;
  node: string;
  moment: string;
  priority: EventPriority;
  kind: EventKind;
  principal?: string;
  affects: string[];
  details?: unknown;
}

interface HistoryEntry {
  _key: string;
  resource_kind: string;
  resource_key: string;
  revision: number;
  changed_by: string;
  changed_at: string;
  snapshot?: unknown;
}

// ---------------------------------------------------------------------------
// Badge helpers
// ---------------------------------------------------------------------------

const PRIORITY_COLORS: Record<EventPriority, string> = {
  Expected: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
  Important: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
  Note: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
  Minor: "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-500",
  Lifecycle: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
};

const KIND_COLORS: Record<EventKind, string> = {
  Error: "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
  Background: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
  EntityManagement: "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400",
  Server: "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400",
  Stats: "bg-cyan-100 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-400",
  Database: "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400",
  ObjectStorage: "bg-teal-100 text-teal-700 dark:bg-teal-900/30 dark:text-teal-400",
};

function Badge({ label, colorClass }: { label: string; colorClass: string }) {
  return (
    <span
      className={`inline-block px-1.5 py-0.5 text-xs font-medium rounded-(--radius-component) ${colorClass}`}
    >
      {label}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Filter chip
// ---------------------------------------------------------------------------

function FilterChip({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`px-2.5 py-1 text-xs rounded-(--radius-component) border transition-colors cursor-pointer ${
        active
          ? "bg-primary-500 text-white border-primary-500"
          : "bg-white dark:bg-gray-900 text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-700 hover:border-primary-400"
      }`}
    >
      {label}
    </button>
  );
}

// ---------------------------------------------------------------------------
// System Events Tab
// ---------------------------------------------------------------------------

const EVENT_KINDS: EventKind[] = [
  "Error",
  "Background",
  "EntityManagement",
  "Server",
  "Stats",
  "Database",
  "ObjectStorage",
];

const EVENT_PRIORITIES: EventPriority[] = [
  "Expected",
  "Important",
  "Note",
  "Minor",
  "Lifecycle",
];

function SystemEventsTab() {
  const [events, setEvents] = useState<SystemEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [kindFilter, setKindFilter] = useState<EventKind | null>(null);
  const [priorityFilter, setPriorityFilter] = useState<EventPriority | null>(null);
  const [cursor, setCursor] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [expandedKey, setExpandedKey] = useState<string | null>(null);

  const principalIds = useMemo(
    () => [...new Set(events.map((e) => e.principal).filter((p): p is string => !!p))],
    [events]
  );
  const principals = usePrincipals(principalIds);

  const fetchEvents = useCallback(
    async (append: boolean, cur: string | null) => {
      append ? setLoadingMore(true) : setLoading(true);
      setError(null);
      try {
        const params = new URLSearchParams({ limit: "50" });
        if (kindFilter) params.set("kind", kindFilter);
        if (priorityFilter) params.set("priority", priorityFilter);
        if (cur) params.set("cursor", cur);

        const res = await fetch(`/api/v1/debug/events?${params}`, {
          credentials: "include",
        });
        if (!res.ok) {
          if (res.status === 403) {
            setError("Access denied. Godmode permission required.");
            return;
          }
          setError(`Failed to load events (HTTP ${res.status})`);
          return;
        }
        const data = await res.json();
        const newEvents: SystemEvent[] = data.events ?? [];
        setEvents((prev) => (append ? [...prev, ...newEvents] : newEvents));
        setHasMore(data.has_more ?? false);
        setCursor(data.next_cursor ?? null);
      } catch {
        setError("Network error loading events");
      } finally {
        append ? setLoadingMore(false) : setLoading(false);
      }
    },
    [kindFilter, priorityFilter]
  );

  useEffect(() => {
    setCursor(null);
    setEvents([]);
    fetchEvents(false, null);
  }, [kindFilter, priorityFilter, fetchEvents]);

  return (
    <div className="flex flex-col gap-4">
      {/* Filters */}
      <div className="flex flex-col gap-2">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-xs text-gray-500 dark:text-gray-400 mr-1">Kind:</span>
          <FilterChip
            label="All"
            active={kindFilter === null}
            onClick={() => setKindFilter(null)}
          />
          {EVENT_KINDS.map((k) => (
            <FilterChip
              key={k}
              label={k}
              active={kindFilter === k}
              onClick={() => setKindFilter(kindFilter === k ? null : k)}
            />
          ))}
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-xs text-gray-500 dark:text-gray-400 mr-1">Priority:</span>
          <FilterChip
            label="All"
            active={priorityFilter === null}
            onClick={() => setPriorityFilter(null)}
          />
          {EVENT_PRIORITIES.map((p) => (
            <FilterChip
              key={p}
              label={p}
              active={priorityFilter === p}
              onClick={() => setPriorityFilter(priorityFilter === p ? null : p)}
            />
          ))}
        </div>
      </div>

      {/* Table */}
      {loading ? (
        <Paragraph size="sm" variant="muted">Loading events…</Paragraph>
      ) : error ? (
        <Paragraph size="sm" variant="danger">{error}</Paragraph>
      ) : (
        <>
          <Table.Root>
            <Table.Header>
              <tr>
                <Table.Head>Time</Table.Head>
                <Table.Head>Priority</Table.Head>
                <Table.Head>Kind</Table.Head>
                <Table.Head className="hidden sm:table-cell">Principal</Table.Head>
                <Table.Head className="hidden md:table-cell">Affects</Table.Head>
                <Table.Head>Details</Table.Head>
              </tr>
            </Table.Header>
            {events.length === 0 ? (
              <Table.Empty colSpan={6}>No events found</Table.Empty>
            ) : (
              <>
                <Table.Body>
                  {events.map((ev) => (
                    <>
                      <Table.Row
                        key={ev._key}
                        selected={expandedKey === ev._key}
                        onClick={() =>
                          setExpandedKey(expandedKey === ev._key ? null : ev._key)
                        }
                        className="cursor-pointer"
                        data-testid={`event-row-${ev._key}`}
                      >
                        <Table.Cell className="whitespace-nowrap text-xs text-gray-500 dark:text-gray-400">
                          <span title={ev.moment}>{formatRelativeTime(ev.moment)}</span>
                        </Table.Cell>
                        <Table.Cell>
                          <Badge
                            label={ev.priority}
                            colorClass={PRIORITY_COLORS[ev.priority] ?? "bg-gray-100 text-gray-600"}
                          />
                        </Table.Cell>
                        <Table.Cell>
                          <Badge
                            label={ev.kind}
                            colorClass={KIND_COLORS[ev.kind] ?? "bg-gray-100 text-gray-600"}
                          />
                        </Table.Cell>
                        <Table.Cell className="hidden sm:table-cell">
                          {ev.principal ? (
                            <PrincipalChip
                              id={ev.principal}
                              info={principals[ev.principal]}
                              data-testid={`event-principal-${ev._key}`}
                            />
                          ) : (
                            <span className="text-gray-400 text-xs">—</span>
                          )}
                        </Table.Cell>
                        <Table.Cell className="hidden md:table-cell text-xs">
                          {ev.affects.length > 0 ? ev.affects.join(", ") : <span className="text-gray-400">—</span>}
                        </Table.Cell>
                        <Table.Cell className="text-xs text-primary-600 dark:text-primary-400">
                          {ev.details ? "▾ expand" : <span className="text-gray-400">—</span>}
                        </Table.Cell>
                      </Table.Row>
                      {expandedKey === ev._key && ev.details && (
                        <tr key={`${ev._key}-details`}>
                          <td
                            colSpan={6}
                            className="px-4 pb-3 bg-gray-50 dark:bg-gray-900/40"
                          >
                            <pre className="text-xs text-gray-700 dark:text-gray-300 whitespace-pre-wrap break-all">
                              {JSON.stringify(ev.details, null, 2)}
                            </pre>
                          </td>
                        </tr>
                      )}
                    </>
                  ))}
                </Table.Body>
              </>
            )}
          </Table.Root>

          {hasMore && (
            <div className="flex justify-center pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => fetchEvents(true, cursor)}
                disabled={loadingMore}
                data-testid="load-more-events"
              >
                {loadingMore ? "Loading…" : "Load more"}
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Change History Tab
// ---------------------------------------------------------------------------

const RESOURCE_KINDS = [
  "users",
  "groups",
  "projects",
  "service_accounts",
  "pipeline_accounts",
];

function ChangeHistoryTab() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [kindFilter, setKindFilter] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [expandedKey, setExpandedKey] = useState<string | null>(null);

  const principalIds = useMemo(
    () => [...new Set(entries.map((e) => e.changed_by).filter(Boolean))],
    [entries]
  );
  const principals = usePrincipals(principalIds);

  const fetchHistory = useCallback(
    async (append: boolean, off: number) => {
      append ? setLoadingMore(true) : setLoading(true);
      setError(null);
      try {
        const params = new URLSearchParams({ limit: "50", offset: String(off) });
        if (kindFilter) params.set("kind", kindFilter);

        const res = await fetch(`/api/v1/debug/history?${params}`, {
          credentials: "include",
        });
        if (!res.ok) {
          if (res.status === 403) {
            setError("Access denied. Godmode permission required.");
            return;
          }
          setError(`Failed to load history (HTTP ${res.status})`);
          return;
        }
        const data = await res.json();
        const newEntries: HistoryEntry[] = data.entries ?? [];
        setEntries((prev) => (append ? [...prev, ...newEntries] : newEntries));
        setHasMore(data.has_more ?? false);
        setOffset(data.next_offset ?? off + newEntries.length);
      } catch {
        setError("Network error loading history");
      } finally {
        append ? setLoadingMore(false) : setLoading(false);
      }
    },
    [kindFilter]
  );

  useEffect(() => {
    setOffset(0);
    setEntries([]);
    fetchHistory(false, 0);
  }, [kindFilter, fetchHistory]);

  return (
    <div className="flex flex-col gap-4">
      {/* Kind filter */}
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="text-xs text-gray-500 dark:text-gray-400 mr-1">Resource:</span>
        <FilterChip
          label="All"
          active={kindFilter === null}
          onClick={() => setKindFilter(null)}
        />
        {RESOURCE_KINDS.map((k) => (
          <FilterChip
            key={k}
            label={k}
            active={kindFilter === k}
            onClick={() => setKindFilter(kindFilter === k ? null : k)}
          />
        ))}
      </div>

      {/* Table */}
      {loading ? (
        <Paragraph size="sm" variant="muted">Loading history…</Paragraph>
      ) : error ? (
        <Paragraph size="sm" variant="danger">{error}</Paragraph>
      ) : (
        <>
          <Table.Root>
            <Table.Header>
              <tr>
                <Table.Head>Time</Table.Head>
                <Table.Head>Resource</Table.Head>
                <Table.Head className="hidden sm:table-cell">ID</Table.Head>
                <Table.Head>Rev</Table.Head>
                <Table.Head className="hidden sm:table-cell">Changed by</Table.Head>
                <Table.Head>Snapshot</Table.Head>
              </tr>
            </Table.Header>
            {entries.length === 0 ? (
              <Table.Empty colSpan={6}>No history found</Table.Empty>
            ) : (
              <Table.Body>
                {entries.map((entry) => (
                  <>
                    <Table.Row
                      key={entry._key}
                      selected={expandedKey === entry._key}
                      onClick={() =>
                        setExpandedKey(expandedKey === entry._key ? null : entry._key)
                      }
                      className="cursor-pointer"
                      data-testid={`history-row-${entry._key}`}
                    >
                      <Table.Cell className="whitespace-nowrap text-xs text-gray-500 dark:text-gray-400">
                        <span title={entry.changed_at}>
                          {formatRelativeTime(entry.changed_at)}
                        </span>
                      </Table.Cell>
                      <Table.Cell>
                        <span className="inline-block px-1.5 py-0.5 text-xs font-medium rounded-(--radius-component) bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300">
                          {entry.resource_kind}
                        </span>
                      </Table.Cell>
                      <Table.Cell className="hidden sm:table-cell font-mono text-xs truncate max-w-40">
                        {entry.resource_key}
                      </Table.Cell>
                      <Table.Cell className="text-xs text-gray-500 dark:text-gray-400">
                        #{entry.revision}
                      </Table.Cell>
                      <Table.Cell className="hidden sm:table-cell">
                        <PrincipalChip
                          id={entry.changed_by}
                          info={principals[entry.changed_by]}
                          data-testid={`history-changed-by-${entry._key}`}
                        />
                      </Table.Cell>
                      <Table.Cell className="text-xs text-primary-600 dark:text-primary-400">
                        {entry.snapshot ? "▾ expand" : <span className="text-gray-400">—</span>}
                      </Table.Cell>
                    </Table.Row>
                    {expandedKey === entry._key && entry.snapshot && (
                      <tr key={`${entry._key}-snapshot`}>
                        <td
                          colSpan={6}
                          className="px-4 pb-3 bg-gray-50 dark:bg-gray-900/40"
                        >
                          <pre className="text-xs text-gray-700 dark:text-gray-300 whitespace-pre-wrap break-all">
                            {JSON.stringify(entry.snapshot, null, 2)}
                          </pre>
                        </td>
                      </tr>
                    )}
                  </>
                ))}
              </Table.Body>
            )}
          </Table.Root>

          {hasMore && (
            <div className="flex justify-center pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => fetchHistory(true, offset)}
                disabled={loadingMore}
                data-testid="load-more-history"
              >
                {loadingMore ? "Loading…" : "Load more"}
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

const ADMIN_APPS: AppItem[] = [
  {
    id: "events",
    icon: <Activity className="w-8 h-8" />,
    label: "System Events",
    color: "bg-indigo-500 text-white",
    content: (
      <div className="p-4">
        <SystemEventsTab />
      </div>
    ),
  },
  {
    id: "history",
    icon: <History className="w-8 h-8" />,
    label: "Change History",
    color: "bg-purple-500 text-white",
    content: (
      <div className="p-4">
        <ChangeHistoryTab />
      </div>
    ),
  },
  {
    id: "users",
    icon: <Users className="w-8 h-8" />,
    label: "Users",
    color: "bg-blue-500 text-white",
    content: (
      <div className="p-6">
        <Paragraph variant="muted">User management coming soon.</Paragraph>
      </div>
    ),
  },
  {
    id: "settings",
    icon: <Settings className="w-8 h-8" />,
    label: "System Settings",
    color: "bg-gray-600 text-white",
    content: (
      <div className="p-6">
        <Paragraph variant="muted">System settings coming soon.</Paragraph>
      </div>
    ),
  },
];

export default function AdminPage() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto flex flex-col gap-8">
        <div>
          <Header level="h1">Admin Area</Header>
          <Paragraph variant="muted">
            Administrator dashboard for system management and configuration.
          </Paragraph>
        </div>

        <AppMenu
          apps={ADMIN_APPS}
          columns={4}
          data-testid="admin-app-menu"
        />
      </div>
    </div>
  );
}
