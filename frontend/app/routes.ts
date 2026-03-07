import { type RouteConfig, index, route, layout } from "@react-router/dev/routes";

export default [
  layout("layouts/app-layout.tsx", [
    index("routes/home.tsx"),
    route("sign-in", "routes/sign-in.tsx"),
    route("sign-up", "routes/sign-up.tsx"),
    route("ui-gallery", "routes/ui-gallery.tsx"),
    route("groups", "routes/groups.tsx"),
    route("projects", "routes/projects-list.tsx"),
    route("p/:project_id", "routes/projects.tsx"),
    route("users", "routes/users.tsx"),
    route("u/:user_id", "routes/user-detail.tsx"),
    route("settings", "routes/settings.tsx"),
    route("admin", "routes/admin.tsx"),
  ]),
] satisfies RouteConfig;
