import { useState, useEffect } from "react";
import { Outlet, useLocation } from "react-router";
import { motion } from "framer-motion";
import { TopBar } from "~/components/TopBar";
import { SideMenu } from "~/components/SideMenu";
import type { Route } from "./+types/app-layout";

const TOPBAR_FULL = 56;
const TOPBAR_COMPACT = 40;

export interface AuthUser {
  id: string;
  name: string;
  avatarUrl: string | null;
}

export async function clientLoader() {
  try {
    // The auth cookie is HttpOnly, so the frontend cannot decode its own user
    // ID from document.cookie — /accesscheck/me/permissions doubles as a
    // lightweight "whoami" and echoes back `user_id` for authenticated callers.
    const res = await fetch("/api/v1/accesscheck/me/permissions", {
      credentials: "include",
    });
    if (!res.ok) return { isAuthenticated: false, user: null };

    const { user_id: userId } = (await res.json()) as { user_id?: string };
    if (!userId) return { isAuthenticated: true, user: null };

    const userRes = await fetch(
      `/api/v1/global/users/${userId}`,
      { credentials: "include" }
    );
    if (!userRes.ok) return { isAuthenticated: true, user: null };

    const userData = await userRes.json();
    const user: AuthUser = {
      id: userData.id,
      name: userData.personal?.name || userData.id.replace(/^u_/, ""),
      avatarUrl: userData.avatar_ulid
        ? `/api/v1/static/user_avatars/${userData.avatar_ulid}_thumb.webp`
        : null,
    };
    return { isAuthenticated: true, user };
  } catch {
    return { isAuthenticated: false, user: null };
  }
}

export default function AppLayout({ loaderData }: Route.ComponentProps) {
  const { isAuthenticated, user } = loaderData;
  const [isOpen, setIsOpen] = useState(false);
  const [isDesktop, setIsDesktop] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  const location = useLocation();

  // Sync open state with viewport width
  useEffect(() => {
    const mq = window.matchMedia("(min-width: 1024px)");
    const sync = (e: { matches: boolean }) => {
      setIsDesktop(e.matches);
      setIsOpen(e.matches);
    };
    sync(mq);
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  }, []);

  // Scroll tracking for topbar shrink
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 16);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  // Close on navigation (mobile only)
  useEffect(() => {
    if (!isDesktop) {
      setIsOpen(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [location.pathname]);

  const topbarHeight = scrolled ? TOPBAR_COMPACT : TOPBAR_FULL;

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      <TopBar isOpen={isOpen} onToggle={() => setIsOpen((v) => !v)} scrolled={scrolled} isAuthenticated={isAuthenticated} user={user} />

      <SideMenu
        isOpen={isOpen}
        isDesktop={isDesktop}
        onClose={() => setIsOpen(false)}
        topOffset={topbarHeight}
        isAuthenticated={isAuthenticated}
      />

      {/* Main content — shifts right on desktop when sidebar is open */}
      <motion.main
        initial={false}
        animate={{
          marginLeft: isOpen && isDesktop ? 256 : 0,
          paddingTop: topbarHeight,
        }}
        transition={{ duration: 0.25, ease: "easeInOut" }}
        className="min-h-screen"
      >
        <Outlet />
      </motion.main>
    </div>
  );
}
