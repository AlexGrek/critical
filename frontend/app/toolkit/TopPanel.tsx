import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence, LayoutGroup } from 'framer-motion';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { useFloating, offset, flip, shift, useClick, useDismiss, useRole, useInteractions } from '@floating-ui/react';

// Define the props for the TopPanel component
interface TopPanelProps {
    children?: React.ReactNode;
}


// User icon SVG path data
const userIconPath = "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z";

// TopPanel component
const TopPanel: React.FC<TopPanelProps> = ({ children }) => {
    // State to track if the user has scrolled down
    const [scrolled, setScrolled] = useState(false);

    // Ref for the panel element to measure its height if needed
    const panelRef = useRef<HTMLHTMLDivElement>(null);

    // Effect to add and remove scroll event listener
    useEffect(() => {
        const handleScroll = () => {
            // Check if the scroll position is greater than a threshold (e.g., 50px)
            // Adjust this threshold as needed for when the background should start appearing
            const isScrolled = window.scrollY > 50;
            if (isScrolled !== scrolled) {
                setScrolled(isScrolled);
            }
        };

        // Add the scroll event listener when the component mounts
        window.addEventListener('scroll', handleScroll);

        // Clean up the event listener when the component unmounts
        return () => {
            window.removeEventListener('scroll', handleScroll);
        };
    }, [scrolled]); // Re-run effect only if 'scrolled' state changes

    // Floating UI hooks for the dropdown menu
    const [isOpen, setIsOpen] = useState(false);
    const { refs, floatingStyles, context } = useFloating({
        open: isOpen,
        onOpenChange: setIsOpen,
        middleware: [offset(10), flip(), shift()], // Offset the dropdown by 10px, flip if no space, shift if needed
    });

    // Floating UI interaction hooks
    const click = useClick(context);
    const dismiss = useDismiss(context);
    const role = useRole(context);

    // Combine interaction props
    const { getReferenceProps, getFloatingProps } = useInteractions([
        click,
        dismiss,
        role,
    ]);

    return (
        // The main panel container
        <motion.header
            ref={panelRef}
            className={`
        fixed top-0 left-0 right-0 z-50
        flex items-center justify-between px-4 py-3
        transition-colors duration-300 ease-in-out
        ${scrolled ? 'bg-gray-900/70 backdrop-blur-md shadow-lg' : 'bg-transparent'}
        text-white
      `}
            // Framer Motion initial and animate states for the background
            initial={false} // Don't animate on initial mount, let CSS handle it
            animate={{
                backgroundColor: scrolled ? 'rgba(17, 24, 39, 0.7)' : 'rgba(0, 0, 0, 0)', // Tailwind gray-900 with opacity
                backdropFilter: scrolled ? 'blur(12px)' : 'blur(0px)', // Tailwind backdrop-blur-md is roughly 12px
                boxShadow: scrolled ? '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)' : 'none',
            }}
            transition={{ duration: 0.3, ease: 'easeInOut' }}
        >
            {/* Left section of the panel */}
            <div className="flex items-center space-x-4">
                <h1 className="text-2xl font-bold text-indigo-400">My App</h1>
                {/* Example navigation links */}
                <nav className="hidden md:flex space-x-4">
                    <a href="#" className="text-gray-300 hover:text-white transition-colors duration-200">Home</a>
                    <a href="#" className="text-gray-300 hover:text-white transition-colors duration-200">Features</a>
                    <a href="#" className="text-gray-300 hover:text-white transition-colors duration-200">Pricing</a>
                </nav>
            </div>

            {/* Right section of the panel (e.g., user profile, actions) */}
            <div className="flex items-center space-x-4">
                {children} {/* Render any children passed to the component */}

                {/* LayoutGroup is crucial for animating elements with the same layoutId across different DOM positions (e.g., portals) */}
                <LayoutGroup>
                    {/* Radix UI Dropdown Menu for user profile */}
                    <DropdownMenu.Root open={isOpen} onOpenChange={setIsOpen}>
                        <DropdownMenu.Trigger asChild>
                            <button
                                ref={refs.setReference}
                                {...getReferenceProps()}
                                className="w-10 h-10 rounded-full bg-gray-700 flex items-center justify-center text-xl text-indigo-300 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-900 transition-all duration-200 relative"
                                aria-label="User menu"
                            >
                                {/* This icon is visible when the dropdown is NOT open */}
                                <AnimatePresence>
                                    {!isOpen && (
                                        <motion.svg
                                            layoutId="user-profile-icon" // Unique ID for layout animation
                                            xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}
                                            initial={{ opacity: 1 }}
                                            exit={{ opacity: 0 }}
                                            transition={{ duration: 0.2 }} // Slightly faster transition for disappearance
                                        >
                                            <path strokeLinecap="round" strokeLinejoin="round" d={userIconPath} />
                                        </motion.svg>
                                    )}
                                </AnimatePresence>
                            </button>
                        </DropdownMenu.Trigger>

                        <AnimatePresence>
                            {isOpen && (
                                <DropdownMenu.Portal>
                                    {/* Radix DropdownMenu.Content now directly wraps the motion.div */}
                                    <DropdownMenu.Content
                                        // Floating UI props are applied directly to motion.div via asChild
                                        asChild // Crucial: tells Radix to merge props with its child (motion.div)
                                        sideOffset={0} // Apply sideOffset here for correct positioning
                                    >
                                        <motion.div
                                            ref={refs.setFloating} // Apply ref here
                                            style={floatingStyles} // Apply style here
                                            {...getFloatingProps()} // Spread props here
                                            initial={{ opacity: 0, y: -100 }}
                                            animate={{ opacity: 1, y: 0, x: -300 }}
                                            exit={{ opacity: 0, y: -10 }}
                                            transition={{ duration: 0.2 }} // Match transition duration
                                            className="bg-gray-800 rounded-lg shadow-xl p-2 min-w-[180px] border border-gray-700"
                                        // Removed overflow-hidden from here, as it can cause clipping during layout animations.
                                        // If clipping is an issue, consider alternative solutions or apply it after animation.
                                        >
                                            {/* User Badge Section */}
                                            <div className="flex flex-col items-center p-4 border-b border-gray-700 mb-2">
                                                {/* This icon is visible when the dropdown IS open */}
                                                <AnimatePresence>
                                                    {isOpen && ( // Ensure the icon is only rendered when dropdown is open
                                                        <motion.svg
                                                            layoutId="user-profile-icon" // Same layoutId as the trigger icon
                                                            xmlns="http://www.w3.org/2000/svg" className="h-16 w-16 text-indigo-300 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}
                                                            initial={{ opacity: 0 }} // Initial opacity for entry
                                                            animate={{ opacity: 1 }} // Animate opacity for entry
                                                            exit={{ opacity: 0 }} // Exit opacity for disappearance
                                                            transition={{ delay: 0.1, duration: 0.2 }} // Slight delay to allow trigger icon to start animating out
                                                        >
                                                            <path strokeLinecap="round" strokeLinejoin="round" d={userIconPath} />
                                                        </motion.svg>
                                                    )}
                                                </AnimatePresence>
                                                <span className="font-semibold text-lg text-white">John Doe</span>
                                                <span className="text-gray-400">johndoe@example.com</span>
                                            </div>

                                            {/* Dropdown Menu Items */}
                                            <DropdownMenu.Item className="px-3 py-2 rounded-md hover:bg-indigo-600 hover:text-white cursor-pointer outline-none transition-colors duration-150">
                                                Profile
                                            </DropdownMenu.Item>
                                            <DropdownMenu.Item className="px-3 py-2 rounded-md hover:bg-indigo-600 hover:text-white cursor-pointer outline-none transition-colors duration-150">
                                                Settings
                                            </DropdownMenu.Item>
                                            <DropdownMenu.Separator className="h-px bg-gray-700 my-1" />
                                            <DropdownMenu.Item className="px-3 py-2 rounded-md hover:bg-red-600 hover:text-white cursor-pointer outline-none transition-colors duration-150">
                                                Logout
                                            </DropdownMenu.Item>
                                            <DropdownMenu.Arrow className="fill-gray-800" />
                                        </motion.div>
                                    </DropdownMenu.Content>
                                </DropdownMenu.Portal>
                            )}
                        </AnimatePresence>
                    </DropdownMenu.Root>
                </LayoutGroup>
            </div>
        </motion.header>
    );
};

export default TopPanel;
