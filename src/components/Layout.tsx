import { useState } from 'react';
import { useTheme } from '../context/ThemeContext';
import { useApp } from '../context/AppContext';
import { Button } from './ui/button';
import { Moon, Sun, Monitor, Menu, Box, LogOut, LayoutDashboard, Wallet, Network, Database, History, Settings, X, Coins } from 'lucide-react';
import { cn } from '../lib/utils';
import { motion, AnimatePresence } from 'framer-motion';
import { NavLink } from 'react-router-dom';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';
import ConfirmationModal from './ConfirmationModal';

export default function Layout({ children }: { children: React.ReactNode }) {
    const { theme, setTheme } = useTheme();
    const { exitApp } = useApp();
    const [sidebarOpen, setSidebarOpen] = useState(false);
    const [showExitModal, setShowExitModal] = useState(false);

    const handleExitConfirm = async () => {
        setShowExitModal(false);
        await exitApp();
    };

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        async function setupCloseListener() {
            const window = getCurrentWindow();
            // In Tauri v2, onCloseRequested takes a callback that receives an event object
            // We can prevent the close by calling event.preventDefault()
            const u = await window.onCloseRequested((event) => {
                event.preventDefault();
                setShowExitModal(true);
            });
            unlisten = u;
        }

        setupCloseListener();

        // Cleanup listener on unmount
        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    const navItems = [
        { name: 'Dashboard', path: '/', icon: LayoutDashboard },
        { name: 'Wallet', path: '/wallet', icon: Wallet },
        { name: 'Network', path: '/network', icon: Network },
        { name: 'Tokenomics', path: '/tokenomics', icon: Coins },
        { name: 'Explorer', path: '/explorer', icon: Box },
        { name: 'Mempool', path: '/mempool', icon: Database },
        { name: 'Transactions', path: '/transactions', icon: History },
        { name: 'Settings', path: '/settings', icon: Settings },
    ];

    return (
        <div className="flex bg-background min-h-screen font-sans text-foreground overflow-hidden">
            {/* Sidebar (Desktop) */}
            <aside className="hidden md:flex w-64 flex-col border-r bg-card/50 backdrop-blur-xl">
                <div className="p-6 flex items-center gap-3 border-b border-border/50">
                    <div className="w-9 h-9 rounded-xl bg-primary flex items-center justify-center shadow-lg shadow-primary/20">
                        <Box className="w-5 h-5 text-primary-foreground" />
                    </div>
                    <div className="flex flex-col">
                        <span className="font-bold text-lg leading-none tracking-tight">Antigravity</span>
                        <span className="text-[10px] text-muted-foreground font-medium uppercase tracking-widest mt-1">Mainnet Alpha</span>
                    </div>
                </div>
                <nav className="flex-1 p-4 space-y-1">
                    {navItems.map((item) => (
                        <NavLink
                            key={item.name}
                            to={item.path}
                            className={({ isActive }) => cn(
                                "flex items-center gap-3 px-4 py-3 text-sm font-medium rounded-xl transition-all duration-200",
                                isActive
                                    ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20"
                                    : "hover:bg-primary/10 text-muted-foreground hover:text-primary"
                            )}
                        >
                            <item.icon className="w-4 h-4" />
                            {item.name}
                        </NavLink>
                    ))}
                </nav>
                <div className="p-4 border-t border-border/50">
                    <div className="p-3 rounded-xl bg-secondary/30 flex items-center justify-between">
                        <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">v0.1.0</span>
                        <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]" />
                    </div>
                </div>
            </aside>

            {/* Mobile Sidebar Overlay */}
            <AnimatePresence>
                {sidebarOpen && (
                    <>
                        <motion.div
                            initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                            className="fixed inset-0 bg-background/80 backdrop-blur-sm z-40 md:hidden"
                            onClick={() => setSidebarOpen(false)}
                        />
                        <motion.aside
                            initial={{ x: "-100%" }} animate={{ x: 0 }} exit={{ x: "-100%" }}
                            transition={{ type: "spring", damping: 25, stiffness: 200 }}
                            className="fixed inset-y-0 left-0 w-72 bg-card z-50 border-r md:hidden shadow-2xl flex flex-col"
                        >
                            <div className="p-6 flex items-center justify-between border-b">
                                <div className="flex items-center gap-3">
                                    <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                                        <Box className="w-4 h-4 text-primary-foreground" />
                                    </div>
                                    <span className="font-bold text-lg">Antigravity</span>
                                </div>
                                <Button variant="ghost" size="icon" className="rounded-full" onClick={() => setSidebarOpen(false)}>
                                    <X className="w-5 h-5" />
                                </Button>
                            </div>
                            <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
                                {navItems.map((item) => (
                                    <NavLink
                                        key={item.name}
                                        to={item.path}
                                        onClick={() => setSidebarOpen(false)}
                                        className={({ isActive }) => cn(
                                            "flex items-center gap-4 px-4 py-4 text-base font-medium rounded-2xl transition-all",
                                            isActive
                                                ? "bg-primary text-primary-foreground shadow-xl shadow-primary/25"
                                                : "hover:bg-secondary text-muted-foreground"
                                        )}
                                    >
                                        <item.icon className="w-5 h-5" />
                                        {item.name}
                                    </NavLink>
                                ))}
                            </nav>
                            <div className="p-6 border-t">
                                <Button variant="outline" className="w-full justify-start gap-3 rounded-xl py-6" onClick={() => setShowExitModal(true)}>
                                    <LogOut className="w-5 h-5 text-red-500" />
                                    <span className="text-red-500">Exit Application</span>
                                </Button>
                            </div>
                        </motion.aside>
                    </>
                )}
            </AnimatePresence>

            {/* Main Content */}
            <main className="flex-1 flex flex-col relative h-screen overflow-hidden">
                {/* Header */}
                <header className="h-16 border-b bg-background/50 backdrop-blur-md flex items-center justify-between px-4 sm:px-6 sticky top-0 z-30 flex-shrink-0">
                    <div className="flex items-center gap-4">
                        <Button variant="ghost" size="icon" className="md:hidden rounded-xl border border-border/50" onClick={() => setSidebarOpen(true)}>
                            <Menu className="w-5 h-5" />
                        </Button>
                        <div className="md:hidden flex items-center gap-2">
                            <div className="w-6 h-6 rounded bg-primary flex items-center justify-center">
                                <Box className="w-3.5 h-3.5 text-primary-foreground" />
                            </div>
                            <span className="font-bold text-sm tracking-tight">Antigravity</span>
                        </div>
                    </div>

                    {/* Desktop Theme Switcher & Actions */}
                    <div className="flex items-center gap-2">
                        <div className="hidden md:flex items-center gap-1 bg-secondary/30 p-1 rounded-xl">
                            <Button
                                variant="ghost" size="icon"
                                onClick={() => setTheme("light")}
                                className={cn("h-8 w-8 rounded-lg", theme === 'light' && "bg-background shadow-sm text-primary")}
                            >
                                <Sun className="h-4 w-4" />
                            </Button>
                            <Button
                                variant="ghost" size="icon"
                                onClick={() => setTheme("dark")}
                                className={cn("h-8 w-8 rounded-lg", theme === 'dark' && "bg-background shadow-sm text-primary")}
                            >
                                <Moon className="h-4 w-4" />
                            </Button>
                            <Button
                                variant="ghost" size="icon"
                                onClick={() => setTheme("system")}
                                className={cn("h-8 w-8 rounded-lg", theme === 'system' && "bg-background shadow-sm text-primary")}
                            >
                                <Monitor className="h-4 w-4" />
                            </Button>
                        </div>

                        <div className="w-px h-6 bg-border mx-2 hidden sm:block" />

                        <Button
                            variant="ghost"
                            size="sm"
                            className="text-red-500 hover:text-red-600 hover:bg-red-500/10 gap-2 h-9 px-4 rounded-xl hidden md:flex"
                            onClick={() => setShowExitModal(true)}
                        >
                            <LogOut className="w-4 h-4" />
                            <span className="font-medium">Exit</span>
                        </Button>

                        {/* Mobile Theme Toggle (Simple) */}
                        <Button
                            variant="ghost" size="icon"
                            className="md:hidden rounded-xl"
                            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                        >
                            {theme === 'dark' ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
                        </Button>
                    </div>
                </header>

                <div className="flex-1 overflow-y-auto p-4 sm:p-6 min-h-0">
                    <div className="max-w-7xl mx-auto">
                        {children}
                    </div>
                </div>
            </main>

            <ConfirmationModal
                isOpen={showExitModal}
                onClose={() => setShowExitModal(false)}
                onConfirm={handleExitConfirm}
                title="Exit Application?"
                description="Are you sure you want to close the node and exit? This will stop your contribution to the network and you will stop earning Patience."
                confirmText="Yes, Exit"
                variant="danger"
            />
        </div>
    );
}
