import { useState, useEffect } from 'react';
import { useTheme } from '../context/ThemeContext';
import { useApp } from '../context/AppContext';
import { Button } from './ui/button';
import {
    Moon,
    Sun,
    Menu,
    LogOut,
    LayoutDashboard,
    Wallet,
    Network,
    Settings,
    X,
    Coins,
    Activity,
    Layers,
    Search,
    ChevronRight
} from 'lucide-react';
import { cn } from '../lib/utils';
import { motion, AnimatePresence } from 'framer-motion';
import { NavLink, useLocation } from 'react-router-dom';
import { getCurrentWindow } from '@tauri-apps/api/window';
import ConfirmationModal from './modals/ConfirmationModal';

export default function Layout({ children }: { children: React.ReactNode }) {
    const { theme, setTheme } = useTheme();
    const { exitApp, nodeStatus } = useApp(); // Get nodeStatus
    const [sidebarOpen, setSidebarOpen] = useState(false);
    const [showExitModal, setShowExitModal] = useState(false);
    const location = useLocation();

    // Critical Error State
    const isCriticalError = nodeStatus === "Relay Unreachable";

    // Handle Window Close Request
    useEffect(() => {
        let unlisten: (() => void) | undefined;
        async function setupCloseListener() {
            const window = getCurrentWindow();
            const u = await window.onCloseRequested((event) => {
                event.preventDefault();
                setShowExitModal(true);
            });
            unlisten = u;
        }
        setupCloseListener();
        return () => { if (unlisten) unlisten(); };
    }, []);

    const handleExitConfirm = async () => {
        setShowExitModal(false);
        await exitApp();
    };

    const navItems = [
        { name: 'Overview', path: '/', icon: LayoutDashboard },
        { name: 'Wallet', path: '/wallet', icon: Wallet },
        { name: 'Network', path: '/network', icon: Network },
        { name: 'Tokenomics', path: '/tokenomics', icon: Coins },
        { name: 'Explorer', path: '/explorer', icon: Search },
        { name: 'Mempool', path: '/mempool', icon: Layers },
        { name: 'Activity', path: '/transactions', icon: Activity },
        { name: 'Settings', path: '/settings', icon: Settings },
    ];

    const currentRouteName = navItems.find(item => item.path === location.pathname)?.name || "Dashboard";

    return (
        <div className={cn(
            "flex h-screen w-full bg-background text-foreground font-sans overflow-hidden selection:bg-primary/20 selection:text-primary transition-colors duration-500",
            isCriticalError && "border-[8px] border-red-600 bg-red-950/10",
        )}>

            {/* Desktop Sidebar - Glassmorphism */}
            <aside className="hidden md:flex w-72 flex-col border-r border-border/50 bg-card/30 backdrop-blur-xl relative z-20">
                <div className="h-20 flex items-center px-8 border-b border-border/50 gap-4">
                    <div className="relative group">
                        <div className="absolute -inset-1.5 bg-gradient-to-r from-primary to-purple-600 rounded-lg blur opacity-40 group-hover:opacity-75 transition duration-1000 group-hover:duration-200"></div>
                        <div className="relative h-9 w-9 bg-background rounded-lg flex items-center justify-center ring-1 ring-white/10">
                            <Activity className="h-5 w-5 text-primary" />
                        </div>
                    </div>
                    <div>
                        <span className="font-bold text-xl tracking-tight block">Centichain</span>
                        <span className="text-[10px] text-muted-foreground uppercase tracking-widest font-semibold text-gradient">Chain Node</span>
                    </div>
                </div>

                <div className="flex-1 py-6 px-4 space-y-1 overflow-y-auto scrollbar-thin">
                    <div className="text-xs font-semibold text-muted-foreground mb-4 px-4 tracking-wider uppercase">Menu</div>
                    {navItems.map((item) => (
                        <NavLink
                            key={item.name}
                            to={item.path}
                            className={({ isActive }) => cn(
                                "flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 group relative overflow-hidden",
                                isActive
                                    ? "bg-primary/10 text-primary shadow-[0_0_20px_-12px_rgba(124,58,237,0.5)]"
                                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground hover:translate-x-1"
                            )}
                        >
                            {({ isActive }) => (
                                <>
                                    <item.icon className={cn("h-5 w-5 transition-colors", isActive ? "text-primary" : "text-muted-foreground group-hover:text-foreground")} />
                                    <span className="relative z-10">{item.name}</span>
                                    {isActive && (
                                        <motion.div
                                            layoutId="activeNavIndicator"
                                            className="absolute left-0 w-1 h-8 bg-primary rounded-r-full"
                                            initial={{ opacity: 0 }}
                                            animate={{ opacity: 1 }}
                                            exit={{ opacity: 0 }}
                                        />
                                    )}
                                </>
                            )}
                        </NavLink>
                    ))}
                </div>

                <div className="p-6 border-t border-border/50 bg-gradient-to-b from-transparent to-background/50">
                    <div className="glass-panel p-4 rounded-2xl border border-white/5 shadow-lg">
                        <div className="flex items-center gap-3 mb-3">
                            <div className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_10px_rgba(16,185,129,0.5)]" />
                            <span className="text-sm font-medium">Mainnet Alpha</span>
                        </div>
                        <div className="h-1.5 w-full bg-muted/50 rounded-full overflow-hidden">
                            <div className="h-full w-[98%] bg-emerald-500 rounded-full" />
                        </div>
                        <div className="flex justify-between mt-2 text-[10px] text-muted-foreground font-mono">
                            <span>Block #849201</span>
                            <span>99.9% Uptime</span>
                        </div>
                    </div>
                </div>
            </aside>

            {/* Main Content Area */}
            <main className="flex-1 flex flex-col min-w-0 bg-background/50 backdrop-blur-[2px] relative">

                {/* Critical Error Banner */}
                <AnimatePresence>
                    {isCriticalError && (
                        <motion.div
                            initial={{ height: 0, opacity: 0 }}
                            animate={{ height: 'auto', opacity: 1 }}
                            exit={{ height: 0, opacity: 0 }}
                            className="bg-red-600 text-white px-6 py-2 flex items-center justify-center gap-3 font-bold text-sm shadow-xl z-50"
                        >
                            <Activity className="h-4 w-4 animate-pulse" />
                            CRITICAL NETWORK ERROR: Relay Node Unreachable. Please check your internet connection or relay status.
                        </motion.div>
                    )}
                </AnimatePresence>

                {/* Header */}
                <header className="h-20 border-b border-border/50 bg-background/60 backdrop-blur-xl flex items-center justify-between px-6 sticky top-0 z-30 transition-all duration-300">
                    <div className="flex items-center gap-4">
                        <Button
                            variant="ghost"
                            size="icon"
                            className="md:hidden -ml-2 hover:bg-muted/50"
                            onClick={() => setSidebarOpen(true)}
                        >
                            <Menu className="h-5 w-5" />
                        </Button>

                        <div className="flex items-center text-sm breadcrumbs text-muted-foreground">
                            <span className="hidden md:inline hover:text-foreground transition-colors cursor-pointer">App</span>
                            <ChevronRight className="h-4 w-4 mx-2 opacity-50" />
                            <span className="font-semibold text-foreground play-font">{currentRouteName}</span>
                        </div>
                    </div>

                    <div className="flex items-center gap-3">
                        <div className="hidden md:flex items-center px-3 py-1.5 rounded-full bg-secondary/50 border border-border/50 text-xs font-medium text-muted-foreground">
                            <span className="w-2 h-2 rounded-full bg-primary/70 mr-2"></span>
                            v2.1.0
                        </div>

                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                            className="rounded-full hover:bg-secondary/80 transition-all duration-300 w-10 h-10"
                        >
                            <AnimatePresence mode="wait" initial={false}>
                                <motion.div
                                    key={theme}
                                    initial={{ y: -20, opacity: 0 }}
                                    animate={{ y: 0, opacity: 1 }}
                                    exit={{ y: 20, opacity: 0 }}
                                    transition={{ duration: 0.2 }}
                                >
                                    {theme === 'dark' ? <Sun className="h-5 w-5 text-amber-300" /> : <Moon className="h-5 w-5 text-slate-700" />}
                                </motion.div>
                            </AnimatePresence>
                        </Button>

                        <div className="h-8 w-px bg-border/50 mx-1" />

                        <Button
                            variant="ghost"
                            size="sm"
                            className="text-destructive hover:bg-destructive/10 hover:text-destructive gap-2 rounded-full px-4"
                            onClick={() => setShowExitModal(true)}
                        >
                            <LogOut className="h-4 w-4" />
                            <span className="hidden sm:inline font-medium">Exit</span>
                        </Button>
                    </div>
                </header>

                {/* Content Scroll Area */}
                <div className="flex-1 overflow-y-auto overflow-x-hidden scrollbar-thin">
                    <motion.div
                        key={location.pathname}
                        initial={{ opacity: 0, scale: 0.98, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.98, y: -10 }}
                        transition={{ duration: 0.3, ease: "easeOut" }}
                        className="mx-auto w-full h-full p-4 sm:p-6 lg:p-8"
                    >
                        {children}
                    </motion.div>
                </div>
            </main>

            {/* Mobile Sidebar Sheet */}
            <AnimatePresence>
                {sidebarOpen && (
                    <>
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            className="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden"
                            onClick={() => setSidebarOpen(false)}
                        />
                        <motion.aside
                            initial={{ x: "-100%" }}
                            animate={{ x: 0 }}
                            exit={{ x: "-100%" }}
                            transition={{ type: "spring", stiffness: 300, damping: 30 }}
                            className="fixed inset-y-0 left-0 w-[85%] max-w-xs bg-background/95 backdrop-blur-2xl border-r border-border z-50 flex flex-col md:hidden shadow-2xl"
                        >
                            <div className="h-20 flex items-center justify-between px-6 border-b border-border/50">
                                <div className="flex items-center gap-3">
                                    <div className="h-9 w-9 bg-primary/20 rounded-xl flex items-center justify-center border border-primary/20">
                                        <Activity className="h-5 w-5 text-primary" />
                                    </div>
                                    <span className="font-bold text-xl">Centichain</span>
                                </div>
                                <Button variant="ghost" size="icon" onClick={() => setSidebarOpen(false)}>
                                    <X className="h-5 w-5" />
                                </Button>
                            </div>

                            <nav className="flex-1 p-4 space-y-1 overflow-y-auto scrollbar-thin">
                                {navItems.map((item) => (
                                    <NavLink
                                        key={item.path}
                                        to={item.path}
                                        onClick={() => setSidebarOpen(false)}
                                        className={({ isActive }) => cn(
                                            "flex items-center gap-3 px-4 py-3.5 rounded-xl text-sm font-medium transition-all duration-200",
                                            isActive
                                                ? "bg-primary/10 text-primary border border-primary/10 shadow-sm"
                                                : "text-muted-foreground hover:bg-muted hover:text-foreground"
                                        )}
                                    >
                                        <item.icon className="h-5 w-5" />
                                        {item.name}
                                    </NavLink>
                                ))}
                            </nav>

                            <div className="p-4 border-t border-border/50 bg-muted/20">
                                <Button
                                    variant="destructive"
                                    className="w-full justify-start gap-3 rounded-xl h-11"
                                    onClick={() => {
                                        setSidebarOpen(false);
                                        setShowExitModal(true);
                                    }}
                                >
                                    <LogOut className="h-4 w-4" />
                                    Exit Application
                                </Button>
                            </div>
                        </motion.aside>
                    </>
                )}
            </AnimatePresence>

            <ConfirmationModal
                isOpen={showExitModal}
                onClose={() => setShowExitModal(false)}
                onConfirm={handleExitConfirm}
                title="Exit Application"
                description="Are you sure you want to quit? Your node will stop validating transactions."
                confirmText="Yes, Exit"
                variant="danger"
            />
        </div>
    );
}
