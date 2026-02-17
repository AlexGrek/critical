import React from "react";
import { motion } from "framer-motion";
import { cn } from "~/lib/utils";

interface LogoCriticalProps extends React.HTMLAttributes<HTMLDivElement> {
  size?: "sm" | "md" | "lg";
}

/**
 * Renders the Cr!tical logo: {!}
 * @param {string} [className] - Additional classes for styling.
 * @param {'sm'|'md'|'lg'} [size='md'] - The size of the logo.
 */
const LogoCritical: React.FC<LogoCriticalProps> = ({
  className,
  size = "md",
  ...props
}) => {
  const sizeClasses = {
    sm: "text-xl",
    md: "text-2xl",
    lg: "text-4xl",
  };

  return (
    <div
      className={cn(
        "font-mono font-bold tracking-tighter",
        sizeClasses[size],
        className
      )}
      {...props}
    >
      <span className="text-gray-900 dark:text-white">{"{"}</span>
      <span className="text-red-500">!</span>
      <span className="text-gray-900 dark:text-white">{"}"}</span>
    </div>
  );
};

export const LogoCriticalAnimated: React.FC<LogoCriticalProps> = ({
  className,
  size = "md",
  onClick,
  onMouseEnter,
  onMouseLeave,
}) => {
  const sizeClasses = {
    sm: "text-xl",
    md: "text-2xl",
    lg: "text-4xl",
  };

  // Adjust spacing based on size
  const spacing = {
    sm: 4,
    md: 6,
    lg: 8,
  };

  return (
    <motion.div
      className={cn(
        "font-mono font-bold tracking-tighter flex items-center cursor-pointer",
        sizeClasses[size],
        className
      )}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      initial="rest"
      whileHover="hover"
      variants={{
        rest: {},
        hover: {},
      }}
    >
      <motion.span
        className="text-gray-900 dark:text-white"
        variants={{
          rest: { x: 0 },
          hover: { x: -spacing[size] }, // Move left bracket slightly left
        }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        {"{"}
      </motion.span>
      <motion.span
        className="text-red-500"
        variants={{
          rest: { scale: 1 },
          hover: { scale: 1.1 }, // Optional: slightly scale the exclamation mark
        }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        !
      </motion.span>
      <motion.span
        className="text-gray-900 dark:text-white"
        variants={{
          rest: { x: 0 },
          hover: { x: spacing[size] }, // Move right bracket slightly right
        }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      >
        {"}"}
      </motion.span>
    </motion.div>
  );
};

export default LogoCritical;
