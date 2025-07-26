import React from 'react';
import { ChevronRight, Home } from 'lucide-react'; // Example icons, you can use any icon component

// TopAppHeader Component
// This component is designed to be a fixed header at the top of the screen.
// It has a transparent background and a fixed height of 18px.
// It takes an 'Icon' component as a prop to be displayed on the left.
// Any 'children' passed to it will be rendered on the right,
// arranged in a flex container with a defined spacing.
const TopAppHeader = ({ LeftContent, NavContent, RightContent }) => {
    return (
        <header className="fixed top-0 left-0 right-0 z-100 h-[48px] bg-transparent">
            {/* The main container uses flexbox to distribute the three content sections */}
            <div className="container mx-auto px-6 flex justify-between items-center h-full">
                {/* Left content section */}
                <div className="flex items-center h-full">
                    {LeftContent}
                </div>

                {/* Navigation content section (hidden on small screens, flex on medium and up) */}
                <nav className="hidden md:flex items-center space-x-8 font-mono text-sm h-full">
                    {NavContent}
                </nav>

                {/* Right content section */}
                <div className="flex items-center h-full">
                    {RightContent}
                </div>
            </div>
        </header>
    );
};

export default TopAppHeader;