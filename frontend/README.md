# Critical Frontend

React 19 + React Router 7.5 single-page application for the Critical project management system.

## Setup

```bash
npm install
npm run dev         # Dev server on port 5173 (proxies API to localhost:3742)
npm run build       # Production build
npm run typecheck   # react-router typegen && tsc
npm start           # Serve production build
```

## Routes

All routes are defined in [`app/routes.ts`](app/routes.ts):

| Route         | Component                                   | Purpose                                                             |
| ------------- | ------------------------------------------- | ------------------------------------------------------------------- |
| `/`           | [home.tsx](app/routes/home.tsx)             | Welcome page with link to UI gallery                                |
| `/sign-in`    | [sign-in.tsx](app/routes/sign-in.tsx)       | Login form (POST to `/api/login`)                                   |
| `/sign-up`    | [sign-up.tsx](app/routes/sign-up.tsx)       | Registration form (POST to `/api/register`, auto-login on success)  |
| `/ui-gallery` | [ui-gallery.tsx](app/routes/ui-gallery.tsx) | Component showcase (buttons, inputs, modals, logos, theme switcher) |

Other routes also exist, but not listed.

## Architecture

- **Root layout**: [root.tsx](app/root.tsx) — HTML shell, theme loading script, error boundary
- **Theme context**: [contexts/ThemeContext.tsx](app/contexts/ThemeContext.tsx) — 5 themes (light/dark/barbie/orange/grayscale) with theme-dependent colors and roundness
- **UI components**: [components/](app/components/) — buttons, inputs, modals, logo, theme selector
- **Styling**: TailwindCSS 4 with theme support via CSS custom properties
- **Build tool**: Vite 6 with React Router SSR plugin

## Auth Flow

1. User submits credentials on `/sign-in` or `/sign-up`
2. Frontend sends POST to backend (`/api/login` or `/api/register`)
3. Backend returns JWT token in `Set-Cookie` header
4. Frontend redirects to `/` on success, displays error on failure
5. Subsequent API requests include JWT via cookie (handled by browser)

## Development

- **HMR enabled**: Changes auto-reload without losing state
- **API proxy**: Requests to `/api/*` are proxied to `http://localhost:3742` (see `vite.config.ts`)
- **TypeScript**: Full type safety with React Router's type generation

## Creating New Routes

### 1. Create a new route file in `app/routes/`

```bash
# Example: create /dashboard route
touch app/routes/dashboard.tsx
```

### 2. Implement the route component

```typescript
// app/routes/dashboard.tsx
import type { Route } from "./+types/dashboard";
import { Link } from "react-router";
import { Button } from "~/components";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "{!} Dashboard - Critical" },
    { name: "description", content: "Project dashboard" },
  ];
}

export default function Dashboard() {
  return (
    <div className="space-y-8">
      <h1 className="text-3xl font-bold">Dashboard</h1>
      <p>Welcome to your dashboard</p>
      <Link to="/sign-in">
        <Button variant="primary">Sign In</Button>
      </Link>
    </div>
  );
}
```

### 3. Register the route in `app/routes.ts`

```typescript
// app/routes.ts
import { type RouteConfig, index, route } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("sign-in", "routes/sign-in.tsx"),
  route("sign-up", "routes/sign-up.tsx"),
  route("ui-gallery", "routes/ui-gallery.tsx"),
  route("dashboard", "routes/dashboard.tsx"), // ← Add new route here
] satisfies RouteConfig;
```

### Route with Form Action

For routes that handle form submissions (like sign-in/sign-up):

```typescript
// app/routes/example.tsx
import type { Route } from "./+types/example";
import { Form, redirect, useActionData, useNavigation } from "react-router";
import { Button, Input } from "~/components";

export async function action({ request }: Route.ActionArgs) {
  const formData = await request.formData();
  const name = String(formData.get("name") ?? "");

  if (!name) {
    return { error: "Name is required" };
  }

  try {
    const res = await fetch("http://localhost:3742/api/example", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    });

    if (!res.ok) {
      const body = await res.json();
      return { error: body.error?.message ?? "Request failed" };
    }

    return redirect("/"); // Redirect on success
  } catch {
    return { error: "Unable to reach the server" };
  }
}

export default function Example() {
  const actionData = useActionData<typeof action>();
  const navigation = useNavigation();
  const isSubmitting = navigation.state === "submitting";

  return (
    <Form method="post" className="space-y-4">
      {actionData?.error && (
        <div className="rounded-md bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
          {actionData.error}
        </div>
      )}
      <Input name="name" placeholder="Enter name" disabled={isSubmitting} />
      <Button type="submit" disabled={isSubmitting}>
        {isSubmitting ? "Submitting..." : "Submit"}
      </Button>
    </Form>
  );
}
```

### Using Components

All components are exported from `~/components` for easy importing:

```typescript
import {
  // Core UI
  Button,
  Input,
  Modal,
  MorphModal,
  Card,

  // Typography
  H1, H2, H3, H4, H5, H6,
  Header,
  Paragraph,

  // Code Display
  CodeBlock,
  InlineCode,
  ScrollableLogWindow,

  // Branding & Theme
  LogoCritical,
  LogoCriticalAnimated,
  ThemeCombobox,
} from "~/components";
```

## Component Library

### Typography Components

#### Headers (H1-H6)
Semantic heading components with automatic theme adaptation and responsive sizing.

```typescript
import { H1, H2, H3, Header } from "~/components";

// Convenience components
<H1>Main Title</H1>
<H2>Section Title</H2>
<H3>Subsection</H3>

// Flexible Header component with variants
<Header level="h2" weight="semibold" align="center">
  Centered Heading
</Header>
```

**Props:**
- `level`: `"h1" | "h2" | "h3" | "h4" | "h5" | "h6"` (default: `"h1"`)
- `weight`: `"normal" | "medium" | "semibold" | "bold" | "extrabold"` (default: `"bold"`)
- `align`: `"left" | "center" | "right"` (default: `"left"`)
- `as`: Override semantic element (render h2 styled as h1, etc.)

#### Paragraph
Flexible paragraph component with size and color variants.

```typescript
import { Paragraph } from "~/components";

<Paragraph size="base" variant="default">
  Standard body text
</Paragraph>

<Paragraph size="sm" variant="muted">
  Smaller, muted text for captions
</Paragraph>

<Paragraph variant="danger" weight="semibold">
  Error message in bold
</Paragraph>
```

**Props:**
- `size`: `"xs" | "sm" | "base" | "lg" | "xl"` (default: `"base"`)
- `variant`: `"default" | "muted" | "subtle" | "primary" | "success" | "warning" | "danger"` (default: `"default"`)
- `align`: `"left" | "center" | "right" | "justify"` (default: `"left"`)
- `weight`: `"normal" | "medium" | "semibold" | "bold"` (default: `"normal"`)
- `as`: Render as `"p" | "span" | "div"` (default: `"p"`)

### Code Display Components

#### CodeBlock
Theme-aware code display with monospace font. Ready for future syntax highlighting.

```typescript
import { CodeBlock } from "~/components";

// Block code (default)
<CodeBlock language="typescript">
{`function greet(name: string) {
  return \`Hello, \${name}!\`;
}`}
</CodeBlock>

// Inline code
<CodeBlock variant="inline">npm install</CodeBlock>
```

**Props:**
- `variant`: `"block" | "inline"` (default: `"block"`)
- `size`: `"xs" | "sm" | "base" | "lg"` (default: `"sm"`)
- `language`: Language identifier (for future syntax highlighting)
- `showLineNumbers`: Enable line numbers (future feature)

#### InlineCode
Convenience component for inline code snippets.

```typescript
import { InlineCode } from "~/components";

<Paragraph>
  Run <InlineCode>npm install</InlineCode> to get started
</Paragraph>
```

#### ScrollableLogWindow
Terminal-style log display with auto-scroll behavior. Always displays white text on black background for consistent terminal aesthetic.

```typescript
import { ScrollableLogWindow } from "~/components";

// With log array
<ScrollableLogWindow
  title="Application Logs"
  logs={[
    "[INFO] Server started",
    "[ERROR] Connection failed",
    "[SUCCESS] Retry succeeded",
  ]}
  maxHeight="300px"
/>

// With custom content
<ScrollableLogWindow maxHeight="400px">
  <div className="text-green-400">✓ Build successful</div>
  <div className="text-red-400">✗ 2 tests failed</div>
</ScrollableLogWindow>
```

**Props:**
- `logs`: `string[] | string` — Log lines to display (or use `children` for custom content)
- `title`: Optional header title
- `maxHeight`: Maximum height (default: `"400px"`)
- `scrollThreshold`: Distance in px to consider "at bottom" (default: `10`)

**Features:**
- Auto-scrolls to bottom when new logs appear (only if already at bottom)
- Stops auto-scroll when user scrolls up (preserves reading position)
- Monospace font for consistent formatting
- Theme-aware border radius
- Shows "New logs ↓" indicator when not at bottom

### Core UI Components

See the [UI Gallery](/ui-gallery) route for interactive examples of all components.

### API Calls

Requests to `/api/*` are automatically proxied to the backend:

```typescript
// In dev: http://localhost:5173/api/users → http://localhost:3742/api/users
const res = await fetch("/api/v1/global/users", {
  method: "GET",
  headers: {
    "Authorization": "Bearer " + token, // If using header auth
  },
});
```

### Styling with TailwindCSS

Use Tailwind utility classes. Theme colors are available:

```typescript
<div className="bg-gray-950 dark:bg-gray-900 text-primary-400">
  Themed content
</div>
```

## Theme System

Critical supports **5 distinct visual themes** with automatic color and roundness adaptation:

### Available Themes

| Theme         | Description        | Color Scheme                                 | Roundness                                                 |
| ------------- | ------------------ | -------------------------------------------- | --------------------------------------------------------- |
| **light**     | Bright interface   | White background, gray text                  | Standard (md/lg/xl)                                       |
| **dark**      | Green-tinted dark  | Dark green background, light text            | Standard (md/lg/xl)                                       |
| **barbie**    | Pink-focused light | Pink background, hot pink accents            | **VERY round** (2xl/full) — playful, bubbly               |
| **orange**    | Orange-tinted dark | Dark orange/brown background                 | **Very minimal** (sm) — sharp, utilitarian                |
| **grayscale** | Neutral monochrome | True grayscale with 100% desaturation filter | **NO roundness** (0.125rem) — completely sharp, brutalist |

### Theme Switching

Themes are controlled by the `ThemeContext` ([contexts/ThemeContext.tsx](app/contexts/ThemeContext.tsx)):

```typescript
import { useTheme } from "~/contexts/ThemeContext";

function MyComponent() {
  const { theme, setTheme } = useTheme();

  return (
    <button onClick={() => setTheme("barbie")}>
      Switch to Barbie theme
    </button>
  );
}
```

Or use the built-in `<ThemeCombobox />` component for a dropdown selector.

### Theme Storage

- Current theme is stored in `localStorage` with key `critical-theme`
- Falls back to system preference (`prefers-color-scheme`) if not set
- Theme is applied as a CSS class on the `<html>` element (e.g., `<html class="barbie">`)

### Theme-Dependent Styling

The theme system automatically adjusts:

**1. Colors** — Use `dark:` variant for dark mode support:
```typescript
<div className="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-50">
  Adapts to light/dark themes
</div>
```

**2. Border Radius** — Components automatically adjust roundness per theme:

| CSS Variable            | Usage                    | Barbie            | Orange                | Grayscale           | Light/Dark          |
| ----------------------- | ------------------------ | ----------------- | --------------------- | ------------------- | ------------------- |
| `--radius-component`    | Buttons, inputs, badges  | 1rem (very round) | 0.25rem (very minimal) | 0.125rem (sharp)    | 0.375rem (standard) |
| `--radius-component-lg` | Cards, dropdowns         | 9999px (pill)     | 0.25rem (very minimal) | 0.125rem (sharp)    | 0.5rem (standard)   |
| `--radius-component-xl` | Modals, large containers | 1rem (very round) | 0.25rem (very minimal) | 0.125rem (sharp)    | 0.75rem (standard)  |

Use in components with Tailwind 4 syntax:
```typescript
<div className="rounded-(--radius-component) bg-white p-4">
  Roundness adapts to current theme
</div>

<div className="rounded-(--radius-component-lg) border border-gray-200">
  Large card with theme-dependent roundness
</div>
```

### Theme Configuration

Theme definitions are in [app/app.css](app/app.css):

- Color palettes defined per theme in the `@theme` block
- Theme-specific CSS variables override defaults in theme-specific selectors
- Border radius tokens: `--radius-component`, `--radius-component-lg`, `--radius-component-xl`

### Visual Examples

**Barbie Theme:**
- Buttons become pill-shaped (`rounded-full`)
- Cards have very round corners (`rounded-2xl`)
- Pink color palette throughout

**Orange Theme:**
- Very minimal, sharp corners (0.25rem)
- Utilitarian, compact aesthetic
- Warm orange/brown color tints

**Grayscale Theme:**
- NO roundness — completely sharp edges (0.125rem)
- 100% grayscale filter applied to entire UI
- Brutalist, industrial aesthetic

**Standard Themes (Light/Dark):**
- Comfortable, moderate roundness
- Theme-specific color tints (green for dark theme)
