import { useState, useEffect } from "react";
import { Outlet, useLocation } from "react-router";
import { motion } from "framer-motion";
import { TopBar } from "~/components/TopBar";
import { SideMenu } from "~/components/SideMenu";

const TOPBAR_FULL = 56;
const TOPBAR_COMPACT = 40;

export default function AppLayout() {
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
      <TopBar isOpen={isOpen} onToggle={() => setIsOpen((v) => !v)} scrolled={scrolled} />

      <SideMenu
        isOpen={isOpen}
        isDesktop={isDesktop}
        onClose={() => setIsOpen(false)}
        topOffset={topbarHeight}
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
