import type { Route } from "./+types/home";
import { Link } from "react-router";
import { Welcome } from "../welcome/welcome";
import { Button } from "~/components";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Critical - Project Management" },
    { name: "description", content: "Welcome to Critical!" },
  ];
}

export default function Home() {
  return (
    <div className="space-y-8">
      <Welcome />
      <div className="flex justify-center gap-4">
        <Link to="/groups">
          <Button variant="primary" size="lg">
            View Groups
          </Button>
        </Link>
        <Link to="/ui-gallery">
          <Button variant="secondary" size="lg">
            View UI Component Gallery
          </Button>
        </Link>
      </div>
    </div>
  );
}
