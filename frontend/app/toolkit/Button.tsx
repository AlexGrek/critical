
import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { cva, type VariantProps } from 'class-variance-authority';
import { X } from 'lucide-react';

const buttonVariants = cva(
    'inline-flex items-center justify-center font-sans font-bold text-sm transition-colors focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:ring-offset-black disabled:opacity-50 disabled:pointer-events-none',
    {
        variants: {
            appearance: {
                primary: 'bg-white text-black hover:bg-gray-200',
                red: 'bg-red-600 text-white hover:bg-red-700',
                subtle: 'border border-gray-600 text-gray-300 hover:bg-gray-800 hover:border-gray-500',
                ghost: 'text-gray-300 hover:bg-gray-800 hover:text-white',
                link: 'text-red-500 underline-offset-4 hover:underline',
            },
            size: {
                default: 'py-2 px-4',
                lg: 'py-3 px-8',
            },
        },
        defaultVariants: {
            appearance: 'primary',
            size: 'default',
        },
    }
);

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> { }

/**
 * A versatile button component with multiple appearances.
 */
const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
    ({ className, appearance, size, ...props }, ref) => {
        return (
            <button
                className={buttonVariants({ appearance, size, className })}
                ref={ref}
                {...props}
            />
        );
    }
);
Button.displayName = 'Button';

export default Button;
