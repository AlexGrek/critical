import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { cva, type VariantProps } from 'class-variance-authority';
import { X } from 'lucide-react';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  monospace?: boolean;
}

/**
 * A styled input component.
 * @param {boolean} [monospace=false] - Whether to use a monospace font.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, monospace, ...props }, ref) => {
    return (
      <input
        className={`
          w-full bg-gray-900/50 border border-gray-700 text-white
          px-3 py-2 text-sm
          placeholder:text-gray-500
          focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-red-500
          disabled:cursor-not-allowed disabled:opacity-50
          ${monospace ? 'font-mono' : 'font-sans'}
          ${className}
        `}
        ref={ref}
        {...props}
      />
    );
  }
);
Input.displayName = 'Input';

export default Input;
