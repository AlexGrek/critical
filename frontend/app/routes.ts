import { type RouteConfig, index, route } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("ui-gallery", "routes/ui-gallery.tsx"),
] satisfies RouteConfig;
