import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { cva, type VariantProps } from 'class-variance-authority';
import { X } from 'lucide-react';

/**
 * A container for a main section of the page.
 */
export const Section: React.FC<React.HTMLAttributes<HTMLElement>> = ({ className, children, ...props }) => {
    return (
        <section className={`py-20 ${className}`} {...props}>
            <div className="container mx-auto px-6">
                {children}
            </div>
        </section>
    );
};

/**
 * A styled paragraph component for readable body text.
 */
export const Paragraph: React.FC<React.HTMLAttributes<HTMLParagraphElement>> = ({ className, ...props }) => {
    return <p className={`text-gray-400 font-light text-lg leading-relaxed ${className}`} {...props} />;
};

/**
 * A styled container for displaying code blocks.
 */
export const CodeBlock: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({ className, children, ...props }) => {
    return (
        <div className={`font-mono text-sm border border-gray-800 bg-gray-900/50 p-4 ${className}`} {...props}>
            <div className="flex items-center pb-3 mb-4 border-b border-gray-700">
                <div className="w-3 h-3 bg-red-500"></div>
                <div className="w-3 h-3 bg-yellow-500 ml-2"></div>
                <div className="w-3 h-3 bg-green-500 ml-2"></div>
            </div>
            <pre className="text-gray-400 whitespace-pre-wrap">
                {children}
            </pre>
        </div>
    );
};
