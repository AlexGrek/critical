import React, { useState } from 'react';

interface LogoCriticalProps extends React.HTMLAttributes<HTMLDivElement> {
    size?: 'sm' | 'md' | 'lg';
}

/**
 * Renders the Cr!tical logo: {!}
 * @param {string} [className] - Additional classes for styling.
 * @param {'sm'|'md'|'lg'} [size='md'] - The size of the logo.
 */
const LogoCritical: React.FC<LogoCriticalProps> = ({ className, size = 'md', ...props }) => {
    const sizeClasses = {
        sm: 'text-xl',
        md: 'text-2xl',
        lg: 'text-4xl',
    };

    return (
        <div className={`font-mono font-bold tracking-tighter ${sizeClasses[size]} ${className}`} {...props}>
            <span className="text-white">{'{'}</span>
            <span className="text-red-500">!</span>
            <span className="text-white">{'}'}</span>
        </div>
    );
};

export default LogoCritical;
