# Critical Frontend

React 19 + React Router 7.5 single-page application for the Critical project management system.

## Setup

```bash
npm install
npm run dev         # Dev server on port 5173 (proxies API to localhost:8080)
npm run build       # Production build
npm run typecheck   # react-router typegen && tsc
npm start           # Serve production build
```

## Routes

All routes are defined in [`app/routes.ts`](app/routes.ts):

| Route | Component | Purpose |
|-------|-----------|---------|
| `/` | [home.tsx](app/routes/home.tsx) | Welcome page with link to UI gallery |
| `/sign-in` | [sign-in.tsx](app/routes/sign-in.tsx) | Login form (POST to `/api/login`) |
| `/sign-up` | [sign-up.tsx](app/routes/sign-up.tsx) | Registration form (POST to `/api/register`, auto-login on success) |
| `/ui-gallery` | [ui-gallery.tsx](app/routes/ui-gallery.tsx) | Component showcase (buttons, inputs, modals, logos, theme switcher) |

## Architecture

- **Root layout**: [root.tsx](app/root.tsx) — HTML shell, theme loading script, error boundary
- **Theme context**: [contexts/ThemeContext.tsx](app/contexts/ThemeContext.tsx) — light/dark/grayscale theme switching
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
- **API proxy**: Requests to `/api/*` are proxied to `http://localhost:8080` (see `vite.config.ts`)
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
    { title: "Dashboard - Critical" },
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
    const res = await fetch("http://localhost:8080/api/example", {
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

Import from `~/components`:

```typescript
import { Button, Input, Modal, LogoCritical, ThemeCombobox } from "~/components";
```

### API Calls

Requests to `/api/*` are automatically proxied to the backend:

```typescript
// In dev: http://localhost:5173/api/users → http://localhost:8080/api/users
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

Supported theme modes: `light`, `dark`, `grayscale` (stored in `localStorage` as `critical-theme`).
