import React, { useEffect, useState } from 'react';
import { motion, useScroll, useTransform } from 'framer-motion';

type ProjectContainerProps = {
    projectName: string;
    icon: React.ReactNode;
    likes: number;
    otherMetadata?: string;
    children: React.ReactNode;
};

const ProjectContainer: React.FC<ProjectContainerProps> = ({
    projectName,
    icon,
    likes,
    otherMetadata,
    children
}) => {
    const [scrollY, setScrollY] = useState(0);
    const { scrollYProgress } = useScroll();

    useEffect(() => {
        const handleScroll = () => setScrollY(window.scrollY);
        window.addEventListener('scroll', handleScroll);
        return () => window.removeEventListener('scroll', handleScroll);
    }, []);

    // Transform values for parallax effects
    const headerBgY = useTransform(scrollYProgress, [0, 1], [0, 300]); // Header background moves slower
    const headerContentY = useTransform(scrollYProgress, [0, 1], [0, 200]); // Header content moves slower than normal scroll
    const headerOpacity = useTransform(scrollYProgress, [0, 0.6], [1, 0]);
    const iconScale = useTransform(scrollYProgress, [0, 0.5], [1, 0.6]);
    const iconOpacity = useTransform(scrollYProgress, [0, 0.4], [1, 0]);
    const metadataOpacity = useTransform(scrollYProgress, [0, 0.3], [1, 0]);

    // Project name animation - moves to top bar
    const nameY = useTransform(scrollYProgress, [0, 0.5], [0, -200]);
    const nameScale = useTransform(scrollYProgress, [0, 0.5], [1, 0.8]);

    return (
        <div className="relative min-h-screen bg-black">
            {/* Fixed Header Background */}
            <motion.div
                className="fixed inset-0 w-full h-64 overflow-hidden"
                style={{ y: headerBgY }}
            >
                {/* Background gradients */}
                <div className="absolute inset-0 bg-gradient-to-br from-purple-900/20 via-black to-black">
                    {/* Blurred gradient circles */}
                    <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-purple-600/30 rounded-full blur-3xl opacity-60"></div>
                    <div className="absolute top-1/3 right-1/3 w-80 h-80 bg-indigo-600/20 rounded-full blur-3xl opacity-40"></div>
                    <div className="absolute bottom-1/4 left-1/2 w-72 h-72 bg-violet-600/25 rounded-full blur-3xl opacity-50"></div>
                </div>

                {/* Apple glass effect overlay */}
                <div className="absolute inset-0 bg-black/20 backdrop-blur-sm"></div>
            </motion.div>

            {/* Top Bar - becomes visible on scroll */}
            <motion.div
                className="fixed top-0 left-0 right-0 z-50 h-[48px] bg-white/10 backdrop-blur-lg border-b border-white/20"
                style={{
                    opacity: useTransform(scrollYProgress, [0.3, 0.6], [0, 1])
                }}
            >
                <div className="flex items-center justify-center h-full">
                    <motion.h1
                        className="text-white font-mono text-lg font-medium"
                        style={{
                            opacity: useTransform(scrollYProgress, [0.4, 0.7], [0, 1])
                        }}
                    >
                        {projectName}
                    </motion.h1>
                </div>
            </motion.div>

            {/* Main Header Content */}
            <div className="relative z-20 h-64 px-8">
                <motion.div
                    className="flex flex-col items-center pt-12"
                    style={{
                        opacity: headerOpacity,
                        transform: useTransform(headerContentY, (value) => `translateY(${value}px)`)
                    }}
                >
                    {/* Icon */}
                    <motion.div
                        className="mb-4 p-4 rounded-2xl bg-white/10 backdrop-blur-lg border border-white/20 shadow-2xl"
                        style={{
                            scale: iconScale,
                            opacity: iconOpacity
                        }}
                    >
                        <div className="w-12 h-12 flex items-center justify-center text-white">
                            {icon}
                        </div>
                    </motion.div>

                    {/* Project Name */}
                    <motion.h1
                        className="text-2xl md:text-3xl font-mono font-bold text-white mb-3 text-center"
                        style={{
                            y: nameY,
                            scale: nameScale
                        }}
                    >
                        {projectName}
                    </motion.h1>

                    {/* Metadata */}
                    <motion.div
                        className="flex items-center gap-4 text-white/80"
                        style={{ opacity: metadataOpacity }}
                    >
                        <div className="flex items-center gap-2">
                            <span className="text-lg">❤️</span>
                            <span className="font-mono text-sm">{likes.toLocaleString()}</span>
                        </div>
                        {otherMetadata && (
                            <>
                                <span className="text-white/40">•</span>
                                <span className="font-mono text-sm">{otherMetadata}</span>
                            </>
                        )}
                    </motion.div>
                </motion.div>
            </div>

            {/* Content Section */}
            <div
                className="relative z-30 bg-gray-900 text-white rounded-t-3xl shadow-2xl min-h-screen"
            >
                <div className="p-8 md:p-12">
                    {children}
                </div>
            </div>
        </div>
    );
};

export default ProjectContainer;
