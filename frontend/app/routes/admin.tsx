import type { Route } from "./+types/admin";
import { Header, Paragraph, Card, CardTitle, CardContent } from "~/components";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Admin Area - Critical" },
    { name: "description", content: "Administrator dashboard and controls" },
  ];
}

export default function AdminPage() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto">
        <Header level="h1">Admin Area</Header>
        <Paragraph className="text-gray-600 dark:text-gray-400 mb-8">
          Administrator dashboard for system management and configuration.
        </Paragraph>

        {/* Placeholder cards for future admin features */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <Card>
            <CardContent>
              <CardTitle>Users</CardTitle>
              <Paragraph size="sm" variant="muted">
                Manage user accounts and permissions
              </Paragraph>
            </CardContent>
          </Card>

          <Card>
            <CardContent>
              <CardTitle>System Settings</CardTitle>
              <Paragraph size="sm" variant="muted">
                Configure system-wide settings and preferences
              </Paragraph>
            </CardContent>
          </Card>

          <Card>
            <CardContent>
              <CardTitle>Audit Log</CardTitle>
              <Paragraph size="sm" variant="muted">
                View system audit logs and activity history
              </Paragraph>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
