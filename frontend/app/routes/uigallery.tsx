import UiGallery from "~/toolkit/UiGallery";
import type { Route } from "./+types/home";
import CriticalLandingPage from "~/components/CriticalLandingPage";

export function meta({ }: Route.MetaArgs) {
  return [
    { title: "New React Router App" },
    { name: "uigallery", content: "Welcome to React Router!" },
  ];
}

export default function Home() {
  return <UiGallery />;
}
