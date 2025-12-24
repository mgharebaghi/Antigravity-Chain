import { Activity, Zap, Layers, Pickaxe, RefreshCw, Shield, Gauge, Globe, CheckCircle2, AlertCircle, ArrowRight } from 'lucide-react';
import { motion } from 'framer-motion';
import { Button } from './ui/button';
import { cn, formatPatience } from '../lib/utils';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatNumber } from '../utils/format';
// import { useTheme } from '../context/ThemeContext';

export default function Dashboard() {
    const { wallet, nodeStatus, patience, peers, startNode, stopNode, latestBlock, minedBlocks, vdfStatus } = useApp();
    // Theme unused
    // const { theme } = useTheme();

    const handleStartNodeClick = async () => {
        try {
            await startNode();
        } catch (e) {
            console.error(e);
        }
    };

    const handleRetry = async () => {
        await stopNode();
        setTimeout(() => startNode(), 2000);
    };

    // const isRunning = nodeStatus.startsWith('Active') || nodeStatus.includes('Syncing');
    const isError = nodeStatus === "Relay Unreachable" || nodeStatus === "Connection Lost";
    const isActive = nodeStatus === "Active";

    const StatCard = ({ title, value, subValue, icon: Icon, statusColor, footerLabel, footerValue }: any) => (
        <div className="glass-card p-6 rounded-2xl flex flex-col justify-between h-full group hover:bg-white/80 dark:hover:bg-black/50 transition-all duration-300">
            <div className="flex justify-between items-start mb-4">
                <div>
                    <p className="text-sm font-medium text-muted-foreground">{title}</p>
                    <div className="flex items-baseline gap-2 mt-2">
                        <h3 className={cn("text-3xl font-bold tracking-tight", statusColor)}>
                            {value}
                        </h3>
                        {subValue && <span className="text-xs text-muted-foreground">{subValue}</span>}
                    </div>
                </div>
                <div className="p-2.5 bg-secondary/50 rounded-xl group-hover:scale-110 transition-transform duration-300">
                    <Icon className="w-5 h-5 text-muted-foreground" />
                </div>
            </div>
            {footerLabel && (
                <div className="pt-4 border-t border-border/50 flex items-center justify-between text-xs sm:text-sm">
                    <span className="text-muted-foreground">{footerLabel}</span>
                    <span className="font-semibold px-2 py-0.5 rounded-lg bg-secondary/50">
                        {footerValue}
                    </span>
                </div>
            )}
        </div>
    );

    return (
        <div className="flex flex-col gap-6 sm:gap-8 pb-8">

            {/* Header Section */}
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-3xl sm:text-4xl font-extrabold tracking-tight text-foreground">
                        Dashboard
                    </h1>
                    <p className="text-muted-foreground mt-2 text-base max-w-2xl">
                        Real-time overview of your node's performance, network participation, and consensus status.
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    {!wallet ? (
                        <Button asChild size="lg" className="gap-2 shadow-lg shadow-primary/20 hover:shadow-primary/40 transition-all">
                            <Link to="/wallet">
                                <Zap className="w-4 h-4" /> Initialize Wallet
                            </Link>
                        </Button>
                    ) : (
                        nodeStatus === "Stopped" ? (
                            <Button onClick={handleStartNodeClick} size="lg" className="gap-2 shadow-lg shadow-primary/20 hover:shadow-primary/40 transition-all">
                                <Zap className="w-4 h-4 fill-current" /> Start Node
                            </Button>
                        ) : (
                            <Button variant="outline" className="gap-2 cursor-default bg-emerald-500/10 text-emerald-600 border-emerald-500/20 hover:bg-emerald-500/10">
                                <span className="relative flex h-2.5 w-2.5 mr-1">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                                </span>
                                Node Running
                            </Button>
                        )
                    )}
                </div>
            </div>

            {/* Status Grid */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 sm:gap-6">
                <StatCard
                    title="Network Status"
                    value={
                        <span className="flex items-center gap-2 text-lg sm:text-2xl">
                            {isError ? <AlertCircle className="w-5 h-5" /> : isActive ? <CheckCircle2 className="w-5 h-5" /> : <Activity className="w-5 h-5" />}
                            {nodeStatus}
                        </span>
                    }
                    icon={Globe}
                    statusColor={isError ? "text-destructive" : isActive ? "text-emerald-500" : "text-foreground"}
                    footerLabel="Peers Connected"
                    footerValue={`${peers} Active`}
                />

                <StatCard
                    title="Chain Height"
                    value={latestBlock ? `#${formatNumber(latestBlock.index, false)}` : "Syncing..."}
                    icon={Layers}
                    footerLabel="Latest Hash"
                    footerValue={latestBlock?.hash ? `${latestBlock.hash.substring(0, 8)}...` : "---"}
                />

                <StatCard
                    title="Blocks Mined"
                    value={formatNumber(minedBlocks, false)}
                    icon={Pickaxe}
                    footerLabel="Contribution"
                    footerValue={minedBlocks > 0 ? "Producer" : "Validator"}
                />
            </div>

            {/* Main Visualizer Area */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1 min-h-[400px]">
                {/* VDF Status Card */}
                <div className="glass-card p-0 overflow-hidden flex flex-col rounded-3xl border-0 ring-1 ring-border/50">
                    <div className="p-6 border-b border-border/50 bg-secondary/30 backdrop-blur-sm flex justify-between items-center">
                        <h3 className="font-bold flex items-center gap-2 text-lg">
                            <Shield className="w-5 h-5 text-primary" /> VDF Protocol
                        </h3>
                        {vdfStatus?.is_active && (
                            <span className="text-[10px] font-bold px-2 py-1 bg-primary/10 text-primary rounded-md uppercase tracking-wider">Active</span>
                        )}
                    </div>
                    <div className="p-8 flex-1 flex flex-col justify-center gap-8 relative bg-gradient-to-b from-transparent to-primary/5">
                        <div className="grid grid-cols-2 gap-6">
                            <div className="flex flex-col items-center text-center p-6 rounded-2xl bg-white/40 dark:bg-black/20 border border-white/10 shadow-sm">
                                <Gauge className="w-8 h-8 text-primary mb-3" />
                                <span className="text-3xl font-bold font-mono tracking-tight">{vdfStatus ? formatNumber(vdfStatus.iterations_per_second, false) : "0"}</span>
                                <span className="text-xs uppercase tracking-wider text-muted-foreground font-semibold mt-2">Iter/Sec</span>
                            </div>
                            <div className="flex flex-col items-center text-center p-6 rounded-2xl bg-white/40 dark:bg-black/20 border border-white/10 shadow-sm">
                                <Zap className="w-8 h-8 text-amber-500 mb-3" />
                                <span className="text-3xl font-bold font-mono tracking-tight">{vdfStatus ? formatNumber(vdfStatus.difficulty, false) : "---"}</span>
                                <span className="text-xs uppercase tracking-wider text-muted-foreground font-semibold mt-2">Difficulty</span>
                            </div>
                        </div>

                        <div className="space-y-3">
                            <div className="flex justify-between text-xs font-bold text-muted-foreground uppercase tracking-widest px-1">
                                <span>Computation Status</span>
                                <span className={vdfStatus?.is_active ? "text-emerald-500" : "text-muted-foreground"}>
                                    {vdfStatus?.is_active ? "RUNNING" : "IDLE"}
                                </span>
                            </div>
                            <div className="h-2 w-full bg-secondary/50 rounded-full overflow-hidden ring-1 ring-border/20">
                                {vdfStatus?.is_active && (
                                    <motion.div
                                        animate={{ x: ['-100%', '100%'] }}
                                        transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                                        className="w-1/2 h-full bg-gradient-to-r from-transparent via-primary to-transparent"
                                    />
                                )}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Patience Accumulator */}
                <div className="glass-card p-0 overflow-hidden flex flex-col rounded-3xl border-0 ring-1 ring-border/50 relative group">
                    <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none" />

                    <div className="p-6 border-b border-border/50 bg-secondary/30 backdrop-blur-sm">
                        <h3 className="font-bold flex items-center gap-2 text-lg">
                            <Activity className="w-5 h-5 text-primary" /> Patience Accumulator
                        </h3>
                    </div>

                    <div className="p-8 flex-1 flex flex-col items-center justify-center text-center relative z-10">
                        <div className="relative mb-8">
                            <div className="text-6xl md:text-7xl font-black tracking-tighter text-gradient drop-shadow-sm">
                                {formatPatience(patience)}
                            </div>
                            {isActive && (
                                <div className="absolute -right-4 -top-2">
                                    <span className="relative flex h-4 w-4">
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                        <span className="relative inline-flex rounded-full h-4 w-4 bg-emerald-500 shadow-lg shadow-emerald-500/50"></span>
                                    </span>
                                </div>
                            )}
                        </div>

                        <p className="text-sm text-balance text-muted-foreground max-w-sm mx-auto leading-relaxed font-medium">
                            {isActive
                                ? "Your node is actively participating in consensus and accumulating proof of patience."
                                : "Initialize your node to start participating in the network consensus."}
                        </p>

                        {!isActive && !isError && (
                            <motion.div
                                className="mt-8"
                                whileHover={{ scale: 1.05 }}
                                whileTap={{ scale: 0.95 }}
                            >
                                <Button onClick={handleStartNodeClick} className="rounded-full px-8 py-6 text-base shadow-xl shadow-primary/20">
                                    Start Node <ArrowRight className="w-4 h-4 ml-2" />
                                </Button>
                            </motion.div>
                        )}

                        {isError && (
                            <div className="mt-8 flex flex-col items-center gap-3">
                                <p className="text-destructive font-bold text-sm bg-destructive/10 px-4 py-2 rounded-lg">Connection interrupted</p>
                                <Button onClick={handleRetry} variant="ghost" size="sm" className="hover:bg-destructive/10 hover:text-destructive">
                                    <RefreshCw className="w-3 h-3 mr-2" /> Retry Connection
                                </Button>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
