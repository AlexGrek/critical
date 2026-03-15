---
name: frontend
description: >
  Expert TypeScript/React/CSS developer for this project's frontend. Use when writing,
  reviewing, or debugging frontend code — routes, components, styling, themes, API
  integration, or new pages. Enforces custom component usage, theme compliance, and
  React Router 7 patterns specific to this codebase.
user-invocable: true
---

You are a **skilled TypeScript developer with strong CSS/Tailwind knowledge** working on
the **Critical** frontend. Apply the following architectural knowledge to every piece of
code you write or review.

---

## Prerequisites — Running the Full Stack

The frontend requires the backend and ArangoDB to be running for API calls.

```bash
make run                    # Terminal 1: Start ArangoDB + backend (port 3742)
cd frontend && npm run dev  # Terminal 2: Vite dev server (port 5173)
```

After `make reset-db` or `make run-fresh`, you **must restart the backend** — it creates
collections on startup. If ArangoDB wasn't running when the backend started, all API
calls will 500.

Building the full app:
```bash
cd frontend
npm run build               # Production build (SSR + client bundles)
npm run typecheck            # react-router typegen && tsc
npm start                   # Serve production build via react-router-serve
```

---

## Toolchain

| Tool              | Version   | Purpose                                        |
| ----------------- | --------- | ---------------------------------------------- |
| React             | 19        | UI framework                                   |
| React Router      | 7.12      | Framework mode with SSR (NOT classic SPA mode, but SPA after hydration)  |
| Vite              | 7         | Build tool + dev server                        |
| TailwindCSS       | 4         | Utility-first CSS (v4 syntax, NOT v3)          |
| TypeScript        | 5.9       | Strict mode enabled                            |
| CVA               | 0.7       | `class-variance-authority` for component variants |
| clsx + tw-merge   | latest    | `cn()` utility for class merging               |
| Framer Motion     | 12        | Animations (MorphModal, logo)                  |
| Radix UI          | latest    | Accessible primitives (Dialog, Select, etc.)   |
| Headless UI       | 2         | ThemeCombobox (Listbox)                        |
| Lucide React      | latest    | Icons                                          |

Path alias: `~/*` maps to `./app/*` (use `import { Button } from "~/components"`)

---

## SSR + SPA Hybrid Architecture (CRITICAL — understand this)

This app runs in **React Router 7 framework mode with SSR enabled**.

### How it works

1. **Server render**: On first page load, the React Router server renders the full HTML
   (loaders run server-side, component tree rendered to HTML string)
2. **Hydration**: The client-side JS bundle hydrates the server-rendered HTML — React
   attaches event listeners to the existing DOM without re-rendering
3. **SPA after hydration**: Once hydrated, all subsequent navigations are client-side SPA
   navigations — loaders run via `fetch()` from the browser, no full page reloads
4. **Both must work**: Every page must render correctly both as server HTML AND as a
   hydrated SPA. No `window`/`document`/`localStorage` access during SSR render —
   guard with `typeof window !== "undefined"` or use `useEffect`

### Production vs Dev

- **Dev**: Vite dev server (`npm run dev`) on port 5173, proxies `/api/*` to `localhost:3742`
- **Production**: `react-router-serve` on port 3000, nginx gateway routes `/api/*` to
  backend and `/*` to the frontend SSR server

---

## React Router 7 Routing (CRITICAL — totally different from classic React Router)

React Router 7 in framework mode is NOT the same as React Router 5/6. Key differences:

### Route Configuration (`app/routes.ts`)

Routes are defined **programmatically** in `app/routes.ts` — NOT via `<Route>` JSX
components, NOT via file-system auto-discovery:

```ts
import { type RouteConfig, index, route, layout } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),                    // /
  route("sign-in", "routes/sign-in.tsx"),      // /sign-in
  route("groups", "routes/groups.tsx"),         // /groups
  // Nested routes use layout():
  // layout("routes/dashboard/layout.tsx", [
  //   index("routes/dashboard/index.tsx"),
  //   route("settings", "routes/dashboard/settings.tsx"),
  // ]),
] satisfies RouteConfig;
```

**Adding a new route**: Add an entry in `app/routes.ts` and create the route file.

### Route File Exports

Each route file is a module with these named exports:

```ts
// Types — auto-generated per-route by `react-router typegen`
import type { Route } from "./+types/route-name";

// meta() — SEO metadata (runs on both server and client)
export function meta({}: Route.MetaArgs) {
  return [
    { title: "Page Title - Critical" },
    { name: "description", content: "..." },
  ];
}

// loader() — Data fetching (runs on SERVER for initial load, via fetch for SPA navigations)
export async function loader({ request, params }: Route.LoaderArgs) {
  // Forward cookies for auth (see API Integration section)
  const response = await fetch("http://localhost:3742/api/v1/global/groups", {
    headers: { Cookie: request.headers.get("Cookie") || "" },
  });
  return { items: await response.json() };
}

// action() — Handles form POST/PUT/DELETE submissions
export async function action({ request }: Route.ActionArgs) {
  const formData = await request.formData();
  const intent = formData.get("intent");
  // ... process and forward to API
  return { success: true };
}

// default export — The page component
export default function PageComponent() {
  const data = useLoaderData<typeof loader>();  // Typed!
  return <div>...</div>;
}

// ErrorBoundary — Per-route error handling (optional)
export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) { ... }
```

### Route Type Generation

React Router 7 auto-generates types for each route in `.react-router/types/`.
Run `npx react-router typegen` (or `npm run typecheck`) to regenerate.
Import as: `import type { Route } from "./+types/route-name"`

This gives you typed `Route.LoaderArgs`, `Route.ActionArgs`, `Route.MetaArgs`, and
typed `useLoaderData<typeof loader>()`.

### Data Flow Patterns

```ts
// Reading loader data in component
const { groups } = useLoaderData<typeof loader>();

// Form submission (triggers action, then re-runs loader automatically)
<Form method="post">
  <input type="hidden" name="intent" value="create" />
  ...
</Form>

// Non-navigating mutation (no page transition, stays on current page)
const fetcher = useFetcher();
fetcher.submit(formData, { method: "POST" });

// Manual revalidation (re-run loader to refresh data)
const revalidator = useRevalidator();
revalidator.revalidate();

// Client-side navigation
import { Link, useNavigate } from "react-router";
<Link to="/groups">Groups</Link>
const navigate = useNavigate();
navigate("/groups");
```

---

## API Integration — Cookie-Based JWT Auth

The backend sets an **HttpOnly cookie** containing the JWT. The browser automatically
sends it on all requests. But in SSR loaders/actions, you must **forward the cookie
explicitly** from the incoming request:

```ts
export async function loader({ request }: Route.LoaderArgs) {
  const response = await fetch("http://localhost:3742/api/v1/global/groups", {
    headers: {
      Cookie: request.headers.get("Cookie") || "",
    },
  });
  // handle response...
}
```

For mutations (POST/PUT/DELETE):
```ts
const response = await fetch("http://localhost:3742/api/v1/global/groups", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    Cookie: request.headers.get("Cookie") || "",
  },
  body: JSON.stringify({ name, id }),
});
```

### Auth Flow

- **Sign-in**: POST to `/api/login` → backend returns `Set-Cookie` → action captures
  it and returns `redirect("/", { headers: { "Set-Cookie": setCookieValue } })`
- **Sign-up**: POST to `/api/register`, then auto-login via `/api/login`
- **Subsequent requests**: Cookie sent automatically by browser; SSR forwards it

### API Routes (Backend)

All API calls go through `/api/` prefix:

| Endpoint | Method | Auth | Purpose |
| -------- | ------ | ---- | ------- |
| `/api/login` | POST | No | Login (returns JWT cookie) |
| `/api/register` | POST | No | Register new user |
| `/api/health` | GET | No | Health check |
| `/api/v1/global/{kind}` | GET | JWT | List resources of a kind |
| `/api/v1/global/{kind}` | POST | JWT | Create resource |
| `/api/v1/global/{kind}/{id}` | GET | JWT | Get single resource |
| `/api/v1/global/{kind}/{id}` | PUT | JWT | Update resource |
| `/api/v1/global/{kind}/{id}` | DELETE | JWT | Soft-delete resource |
| `/api/v1/ws` | WS | JWT | WebSocket |

**Resource kinds**: `users`, `groups`, `projects`, `memberships`, `service_accounts`,
`pipeline_accounts`, `permissions`, `resource_history`, `resource_events`

Response format for list: `{ items: [...] }`
Response format for single: the resource object directly

---

## Custom Components (CRITICAL — always use these, NEVER use bare HTML elements)

All custom components live in `app/components/` and are exported from the barrel
`~/components`. **Always import from `~/components`**, never from individual files.

**NEVER use plain `<button>`, `<input>`, or build ad-hoc card/modal elements.**
Always use the project's custom components instead.

### Available Components

| Component | Import | Use for |
| --------- | ------ | ------- |
| `Button` | `~/components` | All buttons. Variants: `primary`, `secondary`, `destructive`, `outline`, `ghost`, `link`. Sizes: `sm`, `default`, `lg`, `icon` |
| `Input` | `~/components` | All text inputs. Props: `monospace`, `copyable` (clipboard icon) |
| `Modal` | `~/components` | Dialogs. Namespace pattern: `Modal.Root`, `.Trigger`, `.Content`, `.Header`, `.Title`, `.Description`, `.Footer`, `.Close` |
| `MorphModal` | `~/components` | Animated modal that morphs from trigger element. Children can be `(close) => ReactNode` |
| `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter` | `~/components` | Content containers |
| `Header`, `H1`-`H6` | `~/components` | Headings with CVA variants (`level`, `weight`, `align`) |
| `Paragraph` | `~/components` | Text blocks. Variants: `default`, `muted`, `subtle`, `primary`, `success`, `warning`, `danger` |
| `CodeBlock`, `InlineCode` | `~/components` | Code display (block and inline) |
| `ScrollableLogWindow` | `~/components` | Terminal-style log viewer with auto-scroll |
| `YamlEditor` | `~/components` | Textarea-based YAML editor for resource documents. Props: `value` (object), `onChange(parsed)`, `readOnlyFields?` (strips server-managed keys like `state`, `hash_code`, `deletion` from display). Use `useMemo` for `value` to avoid spurious re-serialization. |
| `LogoCritical`, `LogoCriticalAnimated` | `~/components` | `{!}` branding logo |
| `ThemeCombobox` | `~/components` | Theme picker dropdown |
| `TopBar` | `~/components` | Fixed app header (h-14, z-50) with animated logo toggle and user button. Props: `isOpen`, `onToggle` |
| `SideMenu` | `~/components` | Collapsible nav sidebar (w-64) with sections, active-link detection, theme picker at bottom. Props: `isOpen`, `isDesktop`, `onClose` |
| `PrincipalChip` | `~/components` | Inline avatar + display name for any principal (user/group/SA/PA). Props: `id` (raw principal ID), `info` (resolved `PrincipalInfo`), `size` (`xs`\|`sm`\|`md`). Falls back to monospace ID when info is not yet resolved. **Always use this — never display raw principal IDs.** |

### Component Patterns

All components follow these conventions:
- **CVA** for variant definitions (`cva("base-classes", { variants: { ... } })`)
- **`cn()`** for merging classes: `cn(variantClasses, className)`
- **`React.forwardRef`** for ref forwarding on interactive elements
- **`data-testid`** attributes on all interactive elements (for Playwright E2E tests)

```tsx
// Good
import { Button, Card, CardTitle, Input } from "~/components";
<Button variant="primary" size="lg" data-testid="save-btn">Save</Button>
<Input monospace placeholder="Enter ID" data-testid="id-input" />

// Bad — NEVER do this
<button className="bg-blue-500 ...">Save</button>
<input className="border ..." />
```

### Installed but not yet wrapped as custom components

These Radix UI primitives are installed and available for building new components:
`@radix-ui/react-accordion`, `@radix-ui/react-checkbox`, `@radix-ui/react-dropdown-menu`,
`@radix-ui/react-popover`, `@radix-ui/react-select`, `@radix-ui/react-switch`,
`@radix-ui/react-tabs`, `@radix-ui/react-tooltip`, `@floating-ui/react`

When you need these, wrap them as project components in `app/components/` following the
existing CVA + `cn()` + `forwardRef` pattern, add to the barrel export, then use them.

### Radix UI Animation Gotcha (CRITICAL)

Radix `Dialog.Portal` (used by Drawer, Modal, etc.) **unmounts the DOM on close**. It
listens for `animationend` before unmounting, but only when the browser actually applies
a CSS `animation` property to the element.

**Tailwind `data-[state=closed]:` variants DO NOT work with custom utility classes.**
`data-[state=closed]:slide-out-to-right` will NOT generate any CSS output because
Tailwind's variant system only scopes standard Tailwind utilities — custom classes
defined in `@layer utilities` are ignored by variant modifiers.

**Solution — use plain CSS selectors in `app.css`:**

```css
/* Target Radix's data-state attribute directly */
.drawer-overlay[data-state="open"] {
  animation: fadeIn var(--duration-fast) ease-out;
}
.drawer-overlay[data-state="closed"] {
  animation: fadeOut var(--duration-fast) ease-in;
}
.drawer-content-right[data-state="open"] {
  animation: slideInFromRight var(--duration-normal) ease-out;
}
.drawer-content-right[data-state="closed"] {
  animation: slideOutToRight var(--duration-normal) ease-in;
}
```

Then apply the marker class (e.g., `drawer-content-right`) in the component, and let
the CSS rule handle the state-based animation. **Never use `transition-*` for Radix
open/close** — transitions require the element to stay in the DOM, which Radix doesn't
guarantee.

The keyframe animations and duration variables (`--duration-fast`, `--duration-normal`)
are defined in `app.css`.

---

## Theme System (CRITICAL — all elements must support all themes)

6 themes are currently defined. **More themes will be added in the future**, so all code
must be theme-aware and never assume a fixed set of themes.

### Creating a New Theme

**Read the guide first**: `docs/HOW_TO_MAKE_FRONTEND_THEMES.md` — it contains the full
copy-paste CSS template and step-by-step instructions for all 3 files you must touch.

The single most common mistake: **forgetting to override `--color-primary-*`**. The
global default is green. If your theme block doesn't include all 11 `--color-primary-*`
variables (50–950), your theme will have green buttons, badges, and links everywhere.

### Current Themes

| Theme | Base | Colors | Roundness |
| ----- | ---- | ------ | --------- |
| `light` | light | White bg, dark text, green primary | Standard (md/lg/xl) |
| `dark` | dark | Near-black green-tinted bg, green primary | Standard (md/lg/xl) |
| `barbie` | light | Pink bg, hot pink primary | **Very round** (2xl/full/2xl) — pill-shaped |
| `fusion` | light | Light blue bg, sky blue primary | Slightly rounder (lg/xl/xl) |
| `orange` | dark | Dark warm bg, orange primary | **Very minimal** (sm/sm/sm) — sharp |
| `grayscale` | dark | True black bg + `grayscale(100%)` filter | **Zero** (0.125rem) — brutalist |

### Roundness — CSS Variable Tokens (MANDATORY)

**NEVER hardcode `rounded-md`, `rounded-lg`, `rounded-xl`, or any fixed radius.**
Always use the theme-aware CSS variable tokens:

```tsx
// CORRECT — theme-aware roundness
className="rounded-(--radius-component)"       // buttons, inputs, badges, chips
className="rounded-(--radius-component-lg)"    // cards, panels, modals
className="rounded-(--radius-component-xl)"    // large containers, hero sections

// WRONG — hardcoded roundness (breaks barbie/orange/grayscale themes)
className="rounded-md"
className="rounded-lg"
className="rounded-xl"
```

These variables resolve differently per theme:
- Light/dark: `md` / `lg` / `xl` (standard)
- Barbie: `2xl` / `full` / `2xl` (pill-shaped, bubbly)
- Fusion: `lg` / `xl` / `xl` (slightly rounder)
- Orange: `sm` / `sm` / `sm` (sharp, utilitarian)
- Grayscale: `0.125rem` / `0.125rem` / `0.125rem` (no roundness)

### Color Classes — Always Support Dark Mode

Use Tailwind's `dark:` variant for colors. Themes that are dark-based (`dark`, `orange`,
`grayscale`) automatically get the `.dark` class on `<html>`:

```tsx
// Good — works in all themes
className="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"
className="border-gray-200 dark:border-gray-700"

// Bad — only works in light theme
className="bg-white text-gray-900"
```

### Theme Context

```tsx
import { useTheme } from "~/contexts/ThemeContext";
import type { Theme } from "~/contexts/ThemeContext";

// Theme type: "light" | "dark" | "grayscale" | "barbie" | "orange" | "fusion"
const { theme, setTheme } = useTheme();
```

Theme is stored in `localStorage("critical-theme")` and applied as a class on `<html>`.
A blocking `<script>` in `root.tsx` prevents FOUC by reading localStorage before React
hydrates.

### Custom dark mode selector (Tailwind 4)

```css
@custom-variant dark (&:where(.dark, .dark *));
```

The `dark:` variant is driven by the `.dark` class on `<html>`, NOT by
`prefers-color-scheme`. The `orange` and `grayscale` themes add `.dark` alongside
their own class.

---

## Utility Functions (`app/lib/utils.ts`)

```ts
cn(...inputs: ClassValue[]): string          // clsx + tailwind-merge
formatDate(date: Date | string): string      // "Feb 27, 2026"
formatRelativeTime(date: Date | string): string  // "2 hours ago"
truncate(str: string, length: number): string    // "Hello wor..."
sleep(ms: number): Promise<void>
```

---

## File Structure

```
frontend/
├── app/
│   ├── app.css                    # Theme definitions, Tailwind imports
│   ├── root.tsx                   # HTML shell, ThemeProvider, ErrorBoundary
│   ├── routes.ts                  # Route configuration (programmatic)
│   ├── routes/                    # Route modules
│   │   ├── home.tsx
│   │   ├── sign-in.tsx
│   │   ├── sign-up.tsx
│   │   ├── groups.tsx
│   │   └── ui-gallery.tsx
│   ├── layouts/
│   │   └── app-layout.tsx         # Shell: TopBar + SideMenu + <Outlet />; wraps all routes
│   ├── components/                # Custom components (always use these!)
│   │   ├── index.ts               # Barrel export
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   ├── Modal.tsx
│   │   ├── MorphModal.tsx
│   │   ├── Card.tsx
│   │   ├── Header.tsx
│   │   ├── Paragraph.tsx
│   │   ├── CodeBlock.tsx
│   │   ├── ScrollableLogWindow.tsx
│   │   ├── LogoCritical.tsx
│   │   ├── ThemeCombobox.tsx
│   │   ├── TopBar.tsx             # Fixed top bar; uses --color-topbar-* CSS vars
│   │   ├── SideMenu.tsx           # Collapsible sidebar; uses --color-nav-* CSS vars
│   │   ├── AclEditor.tsx          # ACL modal editor (AccessControlStore)
│   │   ├── ResourcePicker.tsx     # Async search-as-you-type dropdown for any kind
│   │   ├── YamlEditor.tsx         # Textarea YAML editor for resource documents
│   │   └── PrincipalChip.tsx      # Avatar + name chip for any principal kind
│   ├── contexts/
│   │   └── ThemeContext.tsx        # Theme state management
│   └── lib/
│       └── utils.ts               # cn(), formatDate(), etc.
├── react-router.config.ts         # SSR: true
├── vite.config.ts                 # Tailwind plugin, /api proxy
├── tsconfig.json                  # Strict, ~ alias, bundler resolution
└── package.json
```

---

## Resource Editor Convention — YAML Tab (MANDATORY)

Every resource editor panel/modal that has tabs **must include a "YAML" tab as the last tab**. This gives power users a `kubectl edit`-style raw document view.

### YamlEditor Component Contract

```tsx
export interface YamlEditorProps {
  /** The resource object to display as YAML. Must be useMemo-stabilized. */
  value: Record<string, unknown>;
  /**
   * Called only when user explicitly clicks "Save". Must be async.
   * Throw an Error to show an inline error message and keep the editor dirty.
   * After resolving, the parent MUST refresh `value` from the server so the
   * editor resets to actual server state.
   * Omit (or don't pass) when disabled=true (read-only mode).
   */
  onSave?: (parsed: Record<string, unknown>) => Promise<void>;
  /**
   * Top-level field names to strip from the displayed YAML (server-managed).
   * These fields are preserved on save by merging over the original document.
   * Typical: ["state", "hash_code", "deletion"]
   */
  readOnlyFields?: string[];
  className?: string;
  "data-testid"?: string;
  /** Read-only mode — hides Save button, disables editing. */
  disabled?: boolean;
}
```

**Key behaviours:**
- **No `onChange` prop.** Changes stay local inside the editor until the user clicks Save.
- **Save button** is shown only when not disabled. It becomes active only when there are unsaved changes AND no parse/structure errors.
- **Error blocking**: syntax errors and non-object YAML prevent saving. Errors shown inline below the editor.
- **After onSave resolves**: parent refreshes `value` from the server. YamlEditor detects the new `value` identity and resets to server state (dirty → clean).
- **onSave throws**: error message shown inline, editor stays dirty for the user to fix.

### Pattern for callers — editable resource

```tsx
// 1. Import
import { YamlEditor } from "~/components";

// 2. Memoize the server document (NOT form state — YAML is independent)
const yamlValue = useMemo<Record<string, unknown>>(
  () => resource as unknown as Record<string, unknown>,
  [resource]
);

// 3. Save handler — PUT to API then re-fetch from server
const handleYamlSave = useCallback(async (parsed: Record<string, unknown>) => {
  const res = await fetch(`/api/v1/global/{kind}/${resource.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    // Merge: parsed fields override, but server-managed fields (hidden from YAML) are preserved
    body: JSON.stringify({ ...resource, ...parsed }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.message || body.error || `HTTP ${res.status}`);
  }
  // Refresh from server — updates value prop which resets the editor to server state
  await refreshData();           // e.g. loadGroup(id) for client-fetched data
  revalidator.revalidate();      // if React Router loader also needs refresh
}, [resource, refreshData, revalidator]);

// 4. Render
<Tabs.Content value="yaml" className="p-4 flex flex-col flex-1 min-h-0">
  <YamlEditor
    value={yamlValue}
    onSave={handleYamlSave}
    readOnlyFields={["state", "hash_code", "deletion"]}
    data-testid="yaml-editor"
  />
</Tabs.Content>
```

### Pattern for callers — read-only

```tsx
<YamlEditor
  value={yamlValue}
  disabled
  data-testid="yaml-editor"
/>
```

### Rules
- `YamlEditor` always goes on the **last tab**, after all structured form tabs
- `value` must be `useMemo`-stabilized — never an inline object literal (causes re-serialization on every render)
- `value` must come from **server state** (the loaded resource), NOT from form state. The YAML tab is independent from form tabs — it shows and saves the raw document directly.
- `readOnlyFields={["state", "hash_code", "deletion"]}` strips server-managed fields from display. Always pass this for editable editors.
- On save: merge `{ ...originalDoc, ...parsed }` so hidden server-managed fields are preserved in the PUT body.
- After `onSave` resolves: call both `loadGroup(id)` (or equivalent client re-fetch) AND `revalidator.revalidate()` to refresh all UI from server.
- **NEVER** have `onChange` on YamlEditor — the prop does not exist. YAML changes stay local.
- `yaml` package (`import { stringify, parse } from "yaml"`) is already installed

---

## Displaying Principals — ALWAYS Use PrincipalChip (MANDATORY)

**NEVER display a raw principal ID** (e.g. `u_alice`, `g_eng`) as plain text or in a
`<code>` tag. Always resolve it and render it with `PrincipalChip`.

### API contract — `POST /api/v1/principals/resolve`

```ts
// Request
{ "ids": ["u_alice", "g_eng", "sa_ci"] }   // up to 500 IDs per call

// Response — every requested ID is a key; partial failures are inline
{
  "u_alice": { "type": "user", "name": "Alice", "avatar_ulid": "01jz..." },
  "g_eng":   { "type": "group", "name": "Engineering" },
  "sa_ci":   { "type": "service_account", "name": "CI Runner" },
  "gone":    { "error": "not_found" }
}
```

### Helper utilities (`~/lib/principals`)

```ts
import { resolvePrincipals, resolvePrincipalsClient } from "~/lib/principals";
import type { PrincipalMap } from "~/lib/principals";

// In an SSR loader — forward the request cookie
const principals = await resolvePrincipals(ids, request.headers.get("Cookie") || "");

// Client-side (effect / hook) — credentials sent automatically
const principals = await resolvePrincipalsClient(ids);
```

### `usePrincipals` hook — for client-fetched lists (`~/lib/usePrincipals`)

Use this inside components that load data incrementally (lazy lists, "load more", etc.).
It accumulates resolved data across renders and never re-fetches already-resolved IDs.

```ts
import { usePrincipals } from "~/lib/usePrincipals";

// Derive a stable list of IDs from local state, then resolve
const principalIds = useMemo(
  () => [...new Set(entries.map((e) => e.changed_by).filter(Boolean))],
  [entries]
);
const principals = usePrincipals(principalIds);

// In JSX
<PrincipalChip id={entry.changed_by} info={principals[entry.changed_by]} />
```

### `PrincipalChip` — the only correct way to display a principal

```tsx
import { PrincipalChip } from "~/components";

// Table cell / inline
<PrincipalChip id={entry.changed_by} info={principals[entry.changed_by]} />

// Larger display
<PrincipalChip id={project.state.created_by} info={principals[project.state.created_by]} size="md" />
```

Props:
- `id` — the raw principal ID (required, shown as fallback while unresolved)
- `info` — the resolved `PrincipalInfo` object from the map (pass `undefined` while loading)
- `size` — `"xs"` | `"sm"` (default) | `"md"`
- `data-testid` — always set for Playwright targeting

### Pattern — SSR loader (projects, single-resource pages)

```ts
export async function loader({ request, params }: Route.LoaderArgs) {
  const resource = await fetchResource(...);

  const principalIds = [
    resource.state?.created_by,
    resource.state?.updated_by,
    ...(resource.acl?.list?.flatMap((e) => e.principals) ?? []),
  ].filter((id): id is string => !!id);

  const principals = await resolvePrincipals(principalIds, request.headers.get("Cookie") || "");
  return { resource, principals };
}

export default function Page() {
  const { resource, principals } = useLoaderData<typeof loader>();
  return (
    <PrincipalChip
      id={resource.state.created_by}
      info={principals[resource.state.created_by]}
      data-testid="created-by"
    />
  );
}
```

### Where to use

- Member lists (group members, project members) — replace raw `member.principal`
- ACL display (entry.principals array) — replace raw monospace IDs
- Audit/history tables — replace raw `changed_by`, `principal` columns
- "Created by" / "Updated by" cards — replace `<code className="font-mono">` ID display

---

## Layout Width — Preventing "Super Narrow" Elements (CRITICAL)

A recurring issue: route-level containers or cards appear narrow because `max-w-*` or a
missing `w-full` collapses them in certain parent contexts (flex/grid ancestors, Radix
Tabs.Content, etc.).

### Rules

1. **Route page wrapper**: always use `w-full` (or rely on block-level default). Pair a
   `max-w-*` with `mx-auto` for centering:
   ```tsx
   <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
     <div className="max-w-7xl mx-auto">   {/* ← correct */}
   ```

2. **Top-level route Card/container**: do **not** add a restrictive `max-w-*` unless you
   truly want the element narrow. If the card should fill the column, use `w-full` with
   no max-width, or with a generous max (`max-w-3xl` / `max-w-4xl`) paired with `mx-auto`:
   ```tsx
   {/* BAD — ProfileTab form looks narrow in a wide layout */}
   <Card className="max-w-xl">

   {/* GOOD — fills available column width */}
   <Card className="w-full">

   {/* GOOD — centered with reasonable max, not cramped */}
   <Card className="w-full max-w-3xl">
   ```

3. **Empty-state panels**: use `flex flex-col items-center` for centering — do NOT rely
   on `text-center` alone (heading components apply `text-left` by default, overriding it).
   Only the description paragraph needs a max-width for readability; use `max-w-prose`:
   ```tsx
   {/* BAD — text-center doesn't override H2's default text-left; max-w-sm is tiny */}
   <Card className="text-center py-16">
     <div className="max-w-sm mx-auto">
       <H2>No items</H2>  {/* renders left-aligned! */}

   {/* GOOD */}
   <Card className="w-full py-20">
     <div className="flex flex-col items-center gap-4 px-8">
       <H2>No items</H2>
       <Paragraph className="text-center max-w-prose">Description...</Paragraph>
       <Button variant="primary" size="lg">CTA</Button>
     </div>
   </Card>
   ```

4. **Tailwind v4 `max-w-{name}` GOTCHA**: In Tailwind v4, named size utilities like
   `max-w-sm`, `max-w-md`, `max-w-lg`, `max-w-xl` resolve to `var(--spacing-{name})`
   from the spacing scale — NOT the old v3 container widths. This produces values like
   `max-w-lg = 24px` instead of the expected 512px.
   - ❌ `max-w-lg` → 24px (spacing variable, not a container width)
   - ✅ `max-w-prose` → 65ch (correct for paragraph text)
   - ✅ `max-w-[32rem]` → 512px (arbitrary value, always reliable)
   - ✅ `max-w-96` → 24rem/384px (numeric spacing — predictable)
   - ✅ `max-w-2xl`, `max-w-6xl`, `max-w-7xl` — these DO map to container vars correctly
   **Rule**: never use `max-w-sm/md/lg/xl` for wide container constraints. Use numeric
   (`max-w-96`, `max-w-2xl`+) or `max-w-prose` for text.

5. **Inputs are `w-full` by default** (built into the `Input` component). If they appear
   narrow, the problem is always the *parent* container, not the input itself. Fix the
   parent width.

6. **Radix Tabs.Content** wraps children in a div. In rare layout contexts this can
   collapse width. Defensive fix: add `w-full` to the direct child of `Tabs.Content`.

---

## Mobile Responsiveness (CRITICAL — always think about small screens)

Every UI element must work on small mobile screens (320px–375px wide). This is not
optional — always design mobile-first and use responsive Tailwind breakpoints to
enhance for larger screens.

### Action Buttons in Tables and Lists

Tables and list rows often have action buttons (Edit, Delete, View, etc.). On small
screens these buttons with labels overflow or crowd the row.

**Rule**: In tables and tight list rows, use `size="icon"` (`Button` variant) on small
screens and show the label only on larger screens:

```tsx
{/* BAD — overflows on mobile */}
<Button variant="ghost" size="sm" onClick={onEdit}>
  <Pencil className="w-4 h-4" /> Edit
</Button>

{/* GOOD — icon-only on mobile, labeled on sm+ */}
<Button variant="ghost" size="icon" onClick={onEdit} title="Edit">
  <Pencil className="w-4 h-4" />
  <span className="sr-only sm:not-sr-only sm:ml-1">Edit</span>
</Button>
```

Always add a `title` prop to icon-only buttons so the tooltip identifies the action.

### Table Action Column Layout

For rows with multiple actions (e.g. Edit + Delete), stack them or use a tight flex
row. On very small screens, collapse actions into a single dropdown menu if there are
more than 2 actions:

```tsx
{/* 1–2 actions: icon buttons side by side */}
<div className="flex items-center gap-1">
  <Button variant="ghost" size="icon" title="Edit"><Pencil className="w-4 h-4" /></Button>
  <Button variant="ghost" size="icon" title="Delete"><Trash2 className="w-4 h-4" /></Button>
</div>

{/* 3+ actions: use a dropdown on mobile, inline on md+ */}
<div className="flex items-center gap-1">
  {/* Mobile: dropdown */}
  <div className="md:hidden">
    <DropdownMenu>...</DropdownMenu>
  </div>
  {/* Desktop: inline */}
  <div className="hidden md:flex items-center gap-1">
    <Button ...>Edit</Button>
    <Button ...>Duplicate</Button>
    <Button ...>Delete</Button>
  </div>
</div>
```

### Responsive Table Columns

Hide non-essential columns on small screens using `hidden sm:table-cell` /
`hidden md:table-cell` on `<th>` and `<td>` pairs. Always keep the primary
identifier column and the actions column visible:

```tsx
<th className="hidden md:table-cell">Created</th>
<th className="hidden sm:table-cell">Owner</th>
<th>Name</th>          {/* always visible */}
<th>Actions</th>       {/* always visible */}
```

### General Mobile Rules

- **Padding**: Use `px-3 sm:px-6` on containers — full padding on desktop, tighter on mobile
- **Font sizes**: Prefer `text-sm` in tables; `text-xs` is acceptable for secondary info
- **Wrap flex rows**: Use `flex-wrap` on button groups that might overflow
- **Touch targets**: Minimum 44×44px tap area — `size="icon"` Button is 36px; add
  `p-2` wrapper if the touch area is too small
- **Never truncate primary identifiers** — IDs and names must be readable; use
  `truncate max-w-[120px] sm:max-w-none` if space is tight rather than hiding them

---

## Component Styling Encapsulation (CRITICAL PRINCIPLE)

**NEVER leak styling into component usage sites. Always encapsulate styling within components themselves.**

### The Anti-Pattern (NEVER Do This)

```tsx
// Bad — styling scattered across every usage
<Card className="bg-white dark:bg-gray-900">
  <CardContent className="pt-6">
    <CardTitle className="mb-2">Users</CardTitle>
    <Paragraph className="text-sm text-gray-600 dark:text-gray-400">
      Description here
    </Paragraph>
  </CardContent>
</Card>
```

Problems:
- `bg-white dark:bg-gray-900` is repeated on every `<Card>` usage
- `pt-6` override breaks the component's intended padding logic
- `mb-2` on `CardTitle` should be built-in, not repeated everywhere
- `text-sm text-gray-600 dark:text-gray-400` is verbose and easy to get wrong

### The Correct Pattern

```tsx
// Good — styling fully encapsulated in components
<Card>
  <CardContent>
    <CardTitle>Users</CardTitle>
    <Paragraph size="sm" variant="muted">
      Description here
    </Paragraph>
  </CardContent>
</Card>
```

Benefits:
- No repetition — styling is in the component definition once
- Props-based customization via CVA variants (size, variant, etc.)
- Consistent theming across all usages
- Easy to change styling globally (update component, not every usage)

### Refactoring Guidelines

**Card components** (all in `app/components/Card.tsx`):
- `Card` — already has `bg-white dark:bg-gray-900`, `border`, `shadow-sm`, `rounded-(--radius-component-lg)` built-in. **Do NOT add these to className.**
- `CardTitle` — includes `mb-2` for spacing. **Do NOT override in className.**
- `CardContent` — includes `p-6 pt-6 first:pt-0`. **Do NOT add padding overrides to className.**
- `CardHeader`, `CardFooter` — include full padding. **Use as-is, avoid className overrides.**

**Paragraph component** (use props, not className):
- **DO**: `<Paragraph size="sm" variant="muted">` — uses built-in colors (gray-600/400)
- **DON'T**: `<Paragraph className="text-sm text-gray-600 dark:text-gray-400">` — verbose, wrong
- Variants: `default` (gray-700/300), `muted` (gray-600/400), `subtle` (gray-500/500), `primary`, `success`, `warning`, `danger`
- Sizes: `xs`, `sm`, `base`, `lg`, `xl`

**Header/Paragraph/Button/Input components**:
- All use **CVA** for variant-based styling
- **Props win over className** — use `size="sm" variant="primary"` instead of manual classes
- Only use `className` override for truly unique, one-off styling (document it)

### Implementation Pattern for New Components

When building a new component, always ask: **"What styling is always the same?"**

1. **Identify default styling** — colors, padding, borders, roundness, gaps
2. **Put it in the component** — in the `cn()` call inside the component file
3. **Use CVA for variants** — capture size, color, state variations
4. **Set sensible defaults** — the most common case should require no props
5. **Expose className** only for overrides, not for standard styling

**Example**: If a badge always has 4px padding and gray background, don't make users pass `className="px-1.5 py-1 bg-gray-100"` every time. Build it into the Badge component:

```tsx
// Inside Badge.tsx
const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant, ...props }, ref) => (
    <span
      ref={ref}
      className={cn(
        "inline-block px-1.5 py-1 rounded-(--radius-component)",  // ← default styling
        badgeVariants({ variant, className })  // ← CVA variants override these only
      )}
      {...props}
    />
  )
);

// Usage — no padding/bg in className needed
<Badge variant="success">Active</Badge>  // ✅ Clean
<Badge className="px-4">Wide Badge</Badge>  // ✅ Override only if needed
```

### Audit & Migrate Existing Pages

Proactively refactor pages to remove redundant className props:
- `<Card className="bg-white dark:bg-gray-900">` → remove (already in Card)
- `<CardTitle className="mb-2">` → remove (now built-in)
- `<CardContent className="pt-6">` → remove (overrides default logic)
- `<Paragraph className="text-sm text-gray-600 dark:text-gray-400">` → use `<Paragraph size="sm" variant="muted">`
- Repeated color classes → convert to component variant prop
- Fixed padding/margin → move to component default

---

## Self-Review Before Finishing

- [ ] **Custom components used**: No bare `<button>`, `<input>`, or ad-hoc cards/modals
- [ ] **Principal IDs always resolved**: No raw principal ID (`u_*`, `g_*`, `sa_*`, `pa_*`) displayed as plain text or `<code>` — use `PrincipalChip` with data resolved via `resolvePrincipals` (loader) or `usePrincipals` (client) — see "Displaying Principals" section
- [ ] **Component styling encapsulated**: No redundant className on Card/CardContent/CardTitle; using `variant` prop on Paragraph instead of manual color classes — see "Component Styling Encapsulation" section
- [ ] **No narrow containers**: Route cards/forms use `w-full` (not restrictive `max-w-sm`/`max-w-xl`) — see "Layout Width" section above
- [ ] **Theme roundness**: All `rounded-*` use `rounded-(--radius-component)` variants, never hardcoded
- [ ] **Dark mode colors**: All color classes have `dark:` counterparts
- [ ] **SSR safe**: No `window`/`document`/`localStorage` access during render (use `useEffect` or guards)
- [ ] **Route registered**: New routes added to `app/routes.ts`
- [ ] **Cookie forwarding**: Loaders/actions forward `Cookie` header for JWT auth
- [ ] **data-testid**: All interactive elements have `data-testid` attributes
- [ ] **Types**: Using auto-generated `Route` types from `./+types/`
- [ ] **Imports**: Using `~/` path alias, components from `~/components` barrel
- [ ] **`npm run typecheck`** passes
- [ ] **Mobile responsive**: Table action buttons use `size="icon"` on small screens; non-essential columns hidden on mobile; touch targets ≥44px
- [ ] **New theme** (if adding one): `--color-primary-*` (all 11 values) overridden; `ThemeContext.tsx` updated (type + array + removal + dark check); `ThemeCombobox.tsx` entry added; verified in `/ui-gallery`
