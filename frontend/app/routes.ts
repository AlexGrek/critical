import { type RouteConfig, index, route } from "@react-router/dev/routes";

export default [
  route("/", "routes/home.tsx"),
  route("/auth", "routes/auth.tsx"),
  route("/dashboard", "routes/dashboard.tsx"),
  route("project/:projectId", "routes/project.tsx", [
    index("routes/project/index.tsx"),           // /project/:projectId
    route("tickets", "routes/project/tickets.tsx"), // /project/:projectId/tickets
    route("settings", "routes/project/settings.tsx"), // /project/:projectId/settings
    route("pipelines", "routes/project/pipelines.tsx"), // /project/:projectId/pipelines
  ]),
] satisfies RouteConfig;