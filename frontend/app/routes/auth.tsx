import { Welcome } from "~/welcome/welcome";
import type { Route } from "./+types/home";
import CriticalLandingPage from "~/components/CriticalLandingPage";
import ProjectContainer from "~/components/ProjectContainer";
import { Accessibility } from "lucide-react";
import TopAppHeader from "~/toolkit/TopAppHeader";
import { LogoCriticalAnimated } from "~/toolkit/LogoCritical";
import Button from "~/toolkit/Button";
import { Link } from "react-router";
import AuthPage from "~/components/AuthPage";

export function meta({ }: Route.MetaArgs) {
    return [
        { title: "New React Router App" },
        { name: "description", content: "Welcome to React Router!" },
    ];
}

export default function Auth() {
    return <>
        <AuthPage />
    </>;
}
