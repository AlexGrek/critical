import React, { useState, useEffect, useRef } from 'react';
import { ChevronRight, Github, Terminal, Feather, Zap, Server, Shield } from 'lucide-react';
import UiGallery from '~/toolkit/UiGallery';


// Landing Page Component
const CriticalLandingPage = () => {
    return (
        <div className="bg-black text-white font-sans antialiased">
            <Header />
            <main className="overflow-x-hidden">
                <HeroSection />
                <FeaturesSection />
                <CliSection />
                <HostingSection />
                <CallToActionSection />
            </main>
            <Footer />
        </div>
    );
};

export default CriticalLandingPage;

// Header Component
const Header = () => {
    return (
        <header className="fixed top-0 left-0 right-0 z-50">
            <div className="container mx-auto px-6 py-4 flex justify-between items-center">
                <div className="text-2xl font-mono font-bold tracking-tighter">
                    <span className="text-white">{'{'}</span>
                    <span className="text-red-500">!</span>
                    <span className="text-white">{'}'}</span>
                </div>
                <nav className="hidden md:flex items-center space-x-8 font-mono text-sm">
                    <a href="#features" className="hover:text-red-500 transition-colors">Features</a>
                    <a href="#cli" className="hover:text-red-500 transition-colors">CLI</a>
                    <a href="https://github.com/your-repo" target="_blank" rel="noopener noreferrer" className="hover:text-red-500 transition-colors">Docs</a>
                    <a href="https://github.com/your-repo" target="_blank" rel="noopener noreferrer" className="hover:text-red-500 transition-colors">GitHub</a>
                </nav>
                <button className="bg-white text-black font-bold py-2 px-4 text-sm font-sans flex items-center hover:bg-gray-200 transition-colors">
                    Download
                    <ChevronRight size={16} className="ml-1" />
                </button>
            </div>
        </header>
    );
};

// Hero Section Component
const HeroSection = () => {
    return (
        <section className="min-h-screen flex items-center justify-center text-center relative overflow-hidden pt-20">
            <div className="absolute inset-0 bg-black opacity-50 z-10"></div>
            <div className="absolute -top-1/4 -left-1/4 w-1/2 h-1/2 bg-red-900/50 rounded-full filter blur-3xl animate-pulse"></div>
            <div className="absolute -bottom-1/4 -right-1/4 w-1/2 h-1/2 bg-gray-900/50 rounded-full filter blur-3xl animate-pulse delay-1000"></div>

            <div className="container mx-auto px-6 relative z-20">
                <h1 className="text-6xl md:text-8xl font-bold font-mono tracking-tighter mb-4">
                    <span className="text-white">Cr</span>
                    <span className="text-red-500">!</span>
                    <span className="text-white">tical</span>
                </h1>
                <p className="text-xl md:text-2xl text-gray-300 max-w-3xl mx-auto font-light">
                    The ultimate open-source toolkit for developers. Manage bugs, repositories, pipelines, and tests with unparalleled speed and efficiency.
                </p>
                <div className="mt-10 flex justify-center items-center space-x-4">
                    <a href="https://github.com/your-repo" target="_blank" rel="noopener noreferrer" className="bg-white text-black font-bold py-3 px-8 font-sans flex items-center hover:bg-gray-200 transition-colors">
                        <Github size={20} className="mr-2" />
                        Star on GitHub
                    </a>
                    <a href="#cli" className="border border-gray-600 text-gray-300 font-bold py-3 px-8 font-sans flex items-center hover:bg-gray-800 hover:border-gray-500 transition-colors">
                        <Terminal size={20} className="mr-2" />
                        Explore CLI
                    </a>
                </div>
            </div>
        </section>
    );
};

// Features Section Component
const FeatureCard = ({ icon, title, children }) => (
    <div className="border border-gray-800 p-8 backdrop-blur-sm bg-white/5 hover:border-red-500/50 transition-all duration-300">
        <div className="flex items-center mb-4">
            <div className="text-red-500 mr-4">{icon}</div>
            <h3 className="text-xl font-bold font-mono">{title}</h3>
        </div>
        <p className="text-gray-400 font-light">{children}</p>
    </div>
);

const FeaturesSection = () => {
    return (
        <section id="features" className="py-20">
            <div className="container mx-auto px-6">
                <div className="text-center mb-12">
                    <h2 className="text-4xl font-bold font-mono">Everything you need. Nothing you don't.</h2>
                    <p className="text-gray-400 mt-2 max-w-2xl mx-auto font-light">
                        Cr!tical is built from the ground up to be the perfect companion for modern development workflows.
                    </p>
                </div>
                <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-px bg-gray-800">
                    <FeatureCard icon={<Feather size={24} />} title="Open Source">
                        Built for the community, by the community. Cr!tical is fully open-source with an MIT license. Fork it, extend it, make it yours.
                    </FeatureCard>
                    <FeatureCard icon={<Zap size={24} />} title="Extremely Fast">
                        Written in Rust, Cr!tical delivers blazing-fast performance. No more waiting for slow, bloated applications.
                    </FeatureCard>
                    <FeatureCard icon={<Server size={24} />} title="Easy Self-Hosting">
                        Deploy with a single binary. No external databases or complex dependencies required. It's self-hosting, simplified.
                    </FeatureCard>
                    <FeatureCard icon={<Terminal size={24} />} title="Powerful CLI">
                        A first-class, full-featured command-line interface that lets you manage everything without leaving your terminal.
                    </FeatureCard>
                    <FeatureCard icon={<Shield size={24} />} title="Free Forever">
                        Free for self-hosting with no strings attached. Get all the power of an enterprise-grade tool without the enterprise price tag.
                    </FeatureCard>
                    <FeatureCard icon={<div className="font-bold text-2xl font-mono">UI</div>} title="Modern Web UI">
                        A sleek, intuitive, and fast web interface that makes managing your projects a pleasure, not a chore.
                    </FeatureCard>
                </div>
            </div>
            <UiGallery/>
        </section>
    );
};


// CLI Section Component
const CliSection = () => {
    const [command, setCommand] = useState('crit issues list --project "WebApp"');

    return (
        <section id="cli" className="py-20 bg-black">
            <div className="container mx-auto px-6">
                <div className="grid md:grid-cols-2 gap-16 items-center">
                    <div className="prose prose-invert max-w-none">
                        <h2 className="text-4xl font-bold font-mono">Work at the speed of thought.</h2>
                        <p className="text-gray-400 font-light text-lg">
                            Cr!tical's CLI is not an afterthought—it's a core part of the experience.
                            Manage your entire workflow from the command line with a tool that's fast, intuitive, and scriptable.
                        </p>
                        <ul className="mt-6 space-y-2 font-light">
                            <li className="flex items-start"><ChevronRight size={20} className="text-red-500 mt-1 mr-2 flex-shrink-0" /> Blazing fast, built in Rust.</li>
                            <li className="flex items-start"><ChevronRight size={20} className="text-red-500 mt-1 mr-2 flex-shrink-0" /> Intuitive commands and flags.</li>
                            <li className="flex items-start"><ChevronRight size={20} className="text-red-500 mt-1 mr-2 flex-shrink-0" /> Pipeable output (JSON, table).</li>
                            <li className="flex items-start"><ChevronRight size={20} className="text-red-500 mt-1 mr-2 flex-shrink-0" /> Easily scriptable for automation.</li>
                        </ul>
                    </div>
                    <div className="font-mono text-sm border border-gray-800 bg-gray-900/50 p-4">
                        <div className="flex items-center pb-3 border-b border-gray-700">
                            <div className="w-3 h-3 bg-red-500"></div>
                            <div className="w-3 h-3 bg-yellow-500 ml-2"></div>
                            <div className="w-3 h-3 bg-green-500 ml-2"></div>
                        </div>
                        <div className="pt-4">
                            <div className="flex items-center">
                                <span className="text-green-400 mr-2">$</span>
                                <span className="text-white">{command}</span>
                                <span className="bg-white w-2 h-4 ml-1 animate-pulse"></span>
                            </div>
                            <pre className="text-gray-400 mt-4 whitespace-pre-wrap">
                                {`
 ID   | TITLE                  | STATUS      | ASSIGNEE
------|------------------------|-------------|-----------
 101  | Fix login button style | In Progress | @jane_doe
 102  | API rate limit issue   | Open        | @john_dev
 105  | Update documentation   | Open        | -
`}
                            </pre>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
};


// Hosting Section Component
const HostingSection = () => {
    return (
        <section className="py-20">
            <div className="container mx-auto px-6 text-center">
                <div className="max-w-3xl mx-auto">
                    <Server size={48} className="mx-auto text-red-500 mb-6" />
                    <h2 className="text-4xl font-bold font-mono">Deploy in Minutes. Not Days.</h2>
                    <p className="text-gray-400 mt-4 text-lg font-light">
                        Forget complex setups and endless configuration. Cr!tical is designed for simplicity.
                        It's a single, self-contained Rust binary with an embedded database.
                        Just download, run, and you're live.
                    </p>
                    <div className="mt-8 font-mono text-sm inline-block bg-gray-900 border border-gray-800 p-4 text-left">
                        <p className="text-gray-500"># Download the latest release</p>
                        <p><span className="text-green-400">$</span> wget https://github.com/your-repo/releases/latest/critical-linux-amd64</p>
                        <p className="text-gray-500 mt-2"># Make it executable</p>
                        <p><span className="text-green-400">$</span> chmod +x critical-linux-amd64</p>
                        <p className="text-gray-500 mt-2"># Run it!</p>
                        <p><span className="text-green-400">$</span> ./critical-linux-amd64 serve</p>
                        <p className="text-blue-400 mt-2">==&gt; Cr!tical server listening on http://0.0.0.0:8080</p>
                    </div>
                </div>
            </div>
        </section>
    );
};


// Call To Action Section
const CallToActionSection = () => {
    return (
        <section className="py-20 bg-gray-900/50 border-t border-b border-gray-800">
            <div className="container mx-auto px-6 text-center">
                <h2 className="text-4xl font-bold font-mono">Ready to take control?</h2>
                <p className="text-gray-400 mt-3 max-w-xl mx-auto font-light">
                    Get started with Cr!tical today. It's free, open-source, and ready for your next project.
                </p>
                <div className="mt-8 flex justify-center items-center space-x-4">
                    <a href="https://github.com/your-repo" target="_blank" rel="noopener noreferrer" className="bg-red-600 text-white font-bold py-3 px-8 font-sans flex items-center hover:bg-red-700 transition-colors">
                        <Github size={20} className="mr-2" />
                        View on GitHub
                    </a>
                </div>
            </div>
        </section>
    );
};

// Footer Component
const Footer = () => {
    return (
        <footer className="py-12">
            <div className="container mx-auto px-6 text-center text-gray-500 font-light text-sm">
                <div className="text-2xl font-mono font-bold tracking-tighter mb-4">
                    <span className="text-white">{'{'}</span>
                    <span className="text-red-500">!</span>
                    <span className="text-white">{'}'}</span>
                </div>
                <p>Released under the MIT License.</p>
                <p>Copyright &copy; {new Date().getFullYear()} Cr!tical Project.</p>
            </div>
        </footer>
    );
};