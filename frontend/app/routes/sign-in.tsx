import type { Route } from "./+types/sign-in";
import { Form, Link, redirect, useActionData, useNavigation } from "react-router";
import { Button, Input } from "~/components";
import { LogoCriticalAnimated } from "~/components/LogoCritical";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Sign In - Critical" },
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
    res = await fetch("http://localhost:3742/api/login", {
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
    <div className="min-h-screen flex items-center justify-center bg-gray-950 px-4">
      <div className="w-full max-w-112 space-y-8">
        <div className="flex flex-col items-center gap-2">
          <Link to="/">
            <LogoCriticalAnimated size="lg" />
          </Link>
          <h1 className="text-2xl font-semibold text-white">Sign in</h1>
          <p className="text-sm text-gray-400">
            Enter your credentials to continue
          </p>
        </div>

        <Form method="post" className="space-y-4">
          {actionData?.error && (
            <div className="rounded-md bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
              {actionData.error}
            </div>
          )}

          <div className="space-y-2">
            <label htmlFor="user" className="block text-sm font-medium text-gray-300">
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
            />
          </div>

          <div className="space-y-2">
            <label htmlFor="password" className="block text-sm font-medium text-gray-300">
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
            />
          </div>

          <Button
            type="submit"
            variant="primary"
            size="lg"
            className="w-full"
            disabled={isSubmitting}
          >
            {isSubmitting ? "Signing in..." : "Sign in"}
          </Button>
        </Form>

        <p className="text-center text-sm text-gray-400">
          Don&apos;t have an account?{" "}
          <Link to="/sign-up" className="text-primary-400 hover:text-primary-300 font-medium">
            Sign up
          </Link>
        </p>
      </div>
    </div>
  );
}
