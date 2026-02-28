import type { Route } from "./+types/sign-in";
import { Form, Link, redirect, useActionData, useNavigation } from "react-router";
import {
  Button,
  Input,
  Card,
  CardContent,
  LogoCriticalAnimated,
  ThemeCombobox,
} from "~/components";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "{!} Sign In - Critical" },
    { name: "description", content: "Sign in to your Critical account" },
  ];
}

export async function action({ request }: Route.ActionArgs) {
  const formData = await request.formData();
  const user = String(formData.get("user") ?? "");
  const password = String(formData.get("password") ?? "");

  if (!user || !password) {
    return { error: "Username and password are required" };
  }

  let res: Response;
  try {
    res = await fetch("http://localhost:3742/api/v1/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user, password }),
    });
  } catch {
    return { error: "Unable to reach the server. Please try again later." };
  }

  if (!res.ok) {
    try {
      const body = await res.json();
      return { error: body.error?.message ?? "Invalid username or password" };
    } catch {
      return { error: "Invalid username or password" };
    }
  }

  const setCookie = res.headers.get("set-cookie");
  return redirect("/", {
    headers: setCookie ? { "Set-Cookie": setCookie } : {},
  });
}

export default function SignIn() {
  const actionData = useActionData<typeof action>();
  const navigation = useNavigation();
  const isSubmitting = navigation.state === "submitting";

  return (
    <div className="min-h-screen flex flex-col items-center justify-center px-4 py-12 relative overflow-hidden">
      {/* Subtle ambient glow — uses theme's primary color */}
      <div className="pointer-events-none fixed inset-0" aria-hidden="true">
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-125 h-75 rounded-full bg-primary-500/6 blur-3xl" />
        <div className="absolute bottom-1/4 left-1/2 -translate-x-1/2 translate-y-1/2 w-75 h-50 rounded-full bg-primary-400/4 blur-3xl" />
      </div>

      {/* Theme switcher — positioned below the topbar (max h-14 = 56px) */}
      <div className="fixed top-16 right-4 z-10">
        <ThemeCombobox />
      </div>

      <div className="w-full max-w-112 space-y-6 relative">
        {/* Logo + heading */}
        <div className="flex flex-col items-center gap-3 text-center">
          <Link to="/" aria-label="Go to home">
            <LogoCriticalAnimated size="lg" />
          </Link>
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
              Sign in
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Enter your credentials to continue
            </p>
          </div>
        </div>

        {/* Form card */}
        <Card>
          <CardContent className="pt-6">
            {actionData?.error && (
              <div
                data-testid="sign-in-error"
                className="mb-4 rounded-(--radius-component) bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-600 dark:text-red-400"
              >
                {actionData.error}
              </div>
            )}

            <Form method="post" className="space-y-4">
              <div className="space-y-2">
                <label
                  htmlFor="user"
                  className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Username
                </label>
                <Input
                  id="user"
                  name="user"
                  type="text"
                  autoComplete="username"
                  required
                  placeholder="your_username"
                  disabled={isSubmitting}
                  data-testid="sign-in-username"
                />
              </div>

              <div className="space-y-2">
                <label
                  htmlFor="password"
                  className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Password
                </label>
                <Input
                  id="password"
                  name="password"
                  type="password"
                  autoComplete="current-password"
                  required
                  placeholder="Password"
                  disabled={isSubmitting}
                  data-testid="sign-in-password"
                />
              </div>

              <Button
                type="submit"
                variant="primary"
                size="lg"
                className="w-full"
                disabled={isSubmitting}
                data-testid="sign-in-submit"
              >
                {isSubmitting ? "Signing in…" : "Sign in"}
              </Button>
            </Form>
          </CardContent>
        </Card>

        <p className="text-center text-sm text-gray-500 dark:text-gray-400">
          Don&apos;t have an account?{" "}
          <Link
            to="/sign-up"
            className="font-medium text-primary-600 hover:text-primary-500 dark:text-primary-400 dark:hover:text-primary-300"
          >
            Sign up
          </Link>
        </p>
      </div>
    </div>
  );
}
