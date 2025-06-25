import React, { type CSSProperties, type ButtonHTMLAttributes } from 'react';

// Define the props interface for the LiquidGlassButton component
interface LiquidGlassButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    children: React.ReactNode; // The content inside the button (e.g., text, icons)
    onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void; // Optional click handler
    style?: CSSProperties; // Allows for custom inline styles to be passed
    disabled?: boolean; // Disables the button when true
    primary?: boolean; // Applies primary styling when true
}

const LiquidGlassButton: React.FC<LiquidGlassButtonProps> = ({
    children,
    onClick,
    style,
    disabled,
    primary,
    className, // Allow external Tailwind classes to be merged
    ...rest // Capture any other standard button attributes
}) => {
    // Base Tailwind classes for the button
    const baseClasses = `
    relative
    px-6 py-3
    rounded-xl
    text-white
    font-inter
    text-base
    font-semibold
    transition-all
    duration-200
    ease-in-out
    overflow-hidden
    shadow-lg
    focus:outline-none
    focus:ring-2
    focus:ring-blue-400
    focus:ring-opacity-75
    group
  `;

    // Classes for the liquid glass effect
    const glassClasses = `
    // Base glass effect
    bg-white/[0.08]
    backdrop-blur-xl
    border
    border-white/[0.1]
    hover:border-white/[0.2]
    active:border-white/[0.15]
    transform-gpu
    overflow-hidden

    // Inner shadow for depth
    before:content-['']
    before:absolute
    before:inset-0
    before:rounded-xl
    before:bg-gradient-to-br
    before:from-transparent
    before:via-white/[0.05]
    before:to-transparent
    before:z-0

    // Subtle highlight on hover
    after:content-['']
    after:absolute
    after:inset-0
    after:rounded-xl
    after:bg-gradient-to-tr
    after:from-transparent
    after:via-white/[0.03]
    after:to-transparent
    after:opacity-0
    group-hover:after:opacity-100
    after:transition-opacity
    after:duration-200
  `;

    // Primary specific styles
    const primaryClasses = primary
        ? `
    bg-blue-500/[0.4]
    border-blue-300/[0.6]
    hover:bg-blue-500/[0.6]
    hover:border-blue-300/[0.8]
    active:bg-blue-500/[0.5]
    active:border-blue-300/[0.7]
    text-white
    shadow-xl
    shadow-blue-500/[0.3]
    `
        : '';

    // Disabled specific styles
    const disabledClasses = disabled
        ? `
    opacity-50
    cursor-not-allowed
    pointer-events-none
    // Override glass effects for disabled state
    !bg-gray-700/[0.3]
    !border-gray-500/[0.3]
    !shadow-none
    `
        : '';

    return (
        <button
            onClick={disabled ? undefined : onClick} // Prevent click when disabled
            className={`${baseClasses} ${glassClasses} ${primaryClasses} ${disabledClasses} ${className}`}
            style={style}
            disabled={disabled}
            aria-disabled={disabled} // ARIA attribute for accessibility
            {...rest} // Spread any other props like type, name etc.
        >
            <span className="relative z-10">{children}</span> {/* Ensure content is above glass effects */}
        </button>
    );
};

export default LiquidGlassButton;
