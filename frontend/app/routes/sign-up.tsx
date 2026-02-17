import type { Route } from "./+types/sign-up";
import { Form, Link, redirect, useActionData, useNavigation } from "react-router";
import { Button, Input } from "~/components";
import { LogoCriticalAnimated } from "~/components/LogoCritical";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "{!} Sign Up - Critical" },
    { name: "description", content: "Create a new Critical account" },
  ];
}

export async function action({ request }: Route.ActionArgs) {
  const formData = await request.formData();
  const user = String(formData.get("user") ?? "");
  const password = String(formData.get("password") ?? "");
  const confirmPassword = String(formData.get("confirmPassword") ?? "");

  if (!user || !password) {
    return { error: "Username and password are required" };
  }

  if (password !== confirmPassword) {
    return { error: "Passwords do not match" };
  }

  // Register
  let registerRes: Response;
  try {
    registerRes = await fetch("http://localhost:3742/api/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user, password }),
    });
  } catch {
    return { error: "Unable to reach the server. Please try again later." };
  }

  if (!registerRes.ok) {
    try {
      const body = await registerRes.json();
      return { error: body.error?.message ?? "Registration failed" };
    } catch {
      return { error: "Registration failed" };
    }
  }

  // Auto-login after successful registration
  let loginRes: Response;
  try {
    loginRes = await fetch("http://localhost:3742/api/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user, password }),
    });
  } catch {
    // Registration succeeded but login failed — redirect to sign-in
    return redirect("/sign-in");
  }

  if (!loginRes.ok) {
    return redirect("/sign-in");
  }

  const setCookie = loginRes.headers.get("set-cookie");
  return redirect("/", {
    headers: setCookie ? { "Set-Cookie": setCookie } : {},
  });
}

export default function SignUp() {
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
          <h1 className="text-2xl font-semibold text-white">{"{!} "}Create account</h1>
          <p className="text-sm text-gray-400">
            Sign up to get started with Critical
          </p>
        </div>

        <Form method="post" className="space-y-4">
          {actionData?.error && (
            <div className="rounded-(--radius-component) bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
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
            <p className="text-xs text-gray-500">
              2-25 characters, letters, numbers, and underscores only
            </p>
          </div>

          <div className="space-y-2">
            <label htmlFor="password" className="block text-sm font-medium text-gray-300">
              Password
            </label>
            <Input
              id="password"
              name="password"
              type="password"
              autoComplete="new-password"
              required
              placeholder="Password"
              disabled={isSubmitting}
            />
          </div>

          <div className="space-y-2">
            <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-300">
              Confirm password
            </label>
            <Input
              id="confirmPassword"
              name="confirmPassword"
              type="password"
              autoComplete="new-password"
              required
              placeholder="Confirm password"
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
            {isSubmitting ? "Creating account..." : "Create account"}
          </Button>
        </Form>

        <p className="text-center text-sm text-gray-400">
          Already have an account?{" "}
          <Link to="/sign-in" className="text-primary-400 hover:text-primary-300 font-medium">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
