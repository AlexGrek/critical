import { type RouteConfig, index, route, layout } from "@react-router/dev/routes";

export default [
  layout("layouts/app-layout.tsx", [
    index("routes/home.tsx"),
    route("sign-in", "routes/sign-in.tsx"),
    route("sign-up", "routes/sign-up.tsx"),
    route("ui-gallery", "routes/ui-gallery.tsx"),
    route("groups", "routes/groups.tsx"),
  ]),
] satisfies RouteConfig;
