import { Welcome } from "~/welcome/welcome";
import type { Route } from "./+types/home";
import CriticalLandingPage from "~/components/CriticalLandingPage";
import ProjectContainer from "~/components/ProjectContainer";
import { Accessibility } from "lucide-react";
import TopAppHeader from "~/toolkit/TopAppHeader";
import { LogoCriticalAnimated } from "~/toolkit/LogoCritical";
import Button from "~/toolkit/Button";
import { Link } from "react-router";

export function meta({ }: Route.MetaArgs) {
  return [
    { title: "New React Router App" },
    { name: "description", content: "Welcome to React Router!" },
  ];
}

export default function Home() {
  return <>
    <TopAppHeader LeftContent={<LogoCriticalAnimated />} NavContent={<div />} RightContent={<Link to="/auth">Log in</Link>}>
    </TopAppHeader>
    <ProjectContainer
      projectName="my-cool-project"
      icon={<Accessibility />}
      likes={123}
      otherMetadata="Updated July 2025"
    >
      <p>This is a demo paragraph in the dark content area.</p>
      <p>Scroll to see the parallax and shrink effects.</p>
      <Link to="/project">Go go</Link>
      <Link to="/auth">Log in</Link>
      {/* Add more content to allow scrolling */}
      {Array.from({ length: 30 }, (_, i) => (
        <p key={i}>Content line {i + 1}</p>
      ))}
    </ProjectContainer>
    {/* <Welcome />
    <CriticalLandingPage /> */}

  </>;
}
