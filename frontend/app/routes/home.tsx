import type { Route } from "./+types/home";
import { Link } from "react-router";
import { LogoCritical, ThemeCombobox } from "~/components";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "{!} Critical" },
    { name: "description", content: "Critical navigation" },
  ];
}

const routes = [
  { to: "/sign-in", label: "Sign In", description: "Login page" },
  { to: "/sign-up", label: "Sign Up", description: "Registration page" },
  { to: "/groups", label: "Groups", description: "Groups listing with ACL display" },
  { to: "/ui-gallery", label: "UI Gallery", description: "Component showcase with theme switcher" },
];

export default function Home() {
  return (
    <div className="min-h-screen px-4 py-12">
      <div className="fixed top-4 right-4 z-10">
        <ThemeCombobox />
      </div>

      <div className="max-w-160 mx-auto space-y-8">
        <div className="flex flex-col items-center gap-3 text-center">
          <LogoCritical size="lg" />
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
              Critical
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Test navigation
            </p>
          </div>
        </div>

        <nav className="space-y-2" aria-label="App routes">
          {routes.map(({ to, label, description }) => (
            <Link
              key={to}
              to={to}
              className="flex items-center justify-between px-4 py-3 rounded-(--radius-component-lg) border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 hover:border-primary-400 dark:hover:border-primary-600 hover:bg-primary-50 dark:hover:bg-primary-950/30 transition-colors group"
              data-testid={`nav-link-${to.slice(1)}`}
            >
              <div>
                <div className="text-sm font-medium text-gray-900 dark:text-gray-100 group-hover:text-primary-700 dark:group-hover:text-primary-300">
                  {label}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  {description}
                </div>
              </div>
              <span className="text-gray-400 dark:text-gray-600 group-hover:text-primary-500 text-sm font-mono">
                {to}
              </span>
            </Link>
          ))}
        </nav>
      </div>
    </div>
  );
}
