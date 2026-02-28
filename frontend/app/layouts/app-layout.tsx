import { useState, useEffect } from "react";
import { Outlet, useLocation } from "react-router";
import { motion } from "framer-motion";
import { TopBar } from "~/components/TopBar";
import { SideMenu } from "~/components/SideMenu";

export default function AppLayout() {
  const [isOpen, setIsOpen] = useState(false);
  const [isDesktop, setIsDesktop] = useState(false);
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

  // Close on navigation (mobile only)
  useEffect(() => {
    if (!isDesktop) {
      setIsOpen(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [location.pathname]);

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      <TopBar isOpen={isOpen} onToggle={() => setIsOpen((v) => !v)} />

      <SideMenu isOpen={isOpen} isDesktop={isDesktop} onClose={() => setIsOpen(false)} />

      {/* Main content — shifts right on desktop when sidebar is open */}
      <motion.main
        initial={false}
        animate={{ marginLeft: isOpen && isDesktop ? 256 : 0 }}
        transition={{ duration: 0.25, ease: "easeInOut" }}
        className="pt-14 min-h-screen"
      >
        <Outlet />
      </motion.main>
    </div>
  );
}
