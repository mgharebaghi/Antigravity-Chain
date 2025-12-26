import { Activity, Zap, Layers, Pickaxe, RefreshCw, Shield, Gauge, Globe, Cpu, Server, Hash, Network, Clock } from 'lucide-react';
import { motion } from 'framer-motion';
import { Button } from './ui/button';
import { cn, formatPatience } from '../lib/utils';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatNumber } from '../utils/format';
// import { useTheme } from '../context/ThemeContext';

export default function Dashboard() {
    const { wallet, nodeStatus, patience, peers, startNode, stopNode, latestBlock, minedBlocks, vdfStatus, selfNodeInfo, consensusStatus } = useApp();
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

    const isConnecting = nodeStatus === "Connecting" || nodeStatus === "Connecting to Relay..." || nodeStatus === "Discovering Peers...";

    const StatCard = ({ title, value, subValue, icon: Icon, statusColor, footerLabel, footerValue }: any) => (
        <div className="glass-card p-5 lg:p-6 rounded-2xl flex flex-col justify-between h-full group hover:bg-white/80 dark:hover:bg-black/50 transition-all duration-300 shadow-xl shadow-primary/5">
            <div className="flex justify-between items-start mb-2">
                <div>
                    <p className="text-sm font-medium text-muted-foreground">{title}</p>
                    <div className="flex items-baseline gap-2 mt-1">
                        <h3 className={cn("text-2xl sm:text-3xl lg:text-4xl font-bold tracking-tight", statusColor)}>
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
        <div className="flex flex-col gap-4 lg:h-full w-full lg:overflow-hidden overflow-visible p-1">
            {/* Header - Fixed & Compact */}
            <div className="flex items-center justify-between bg-secondary/20 p-3 rounded-xl border border-border/50 shrink-0">
                <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
                    <Activity className="w-5 h-5 text-primary" /> Overview
                </h1>
                <div className="flex items-center gap-2">
                    {!wallet ? (
                        <Button asChild size="sm" className="h-8">
                            <Link to="/wallet">Init Wallet</Link>
                        </Button>
                    ) : (
                        nodeStatus === "Relay Unreachable" ? (
                            <Button onClick={handleRetry} variant="destructive" size="sm" className="h-8 animate-pulse text-xs">
                                <RefreshCw className="w-3 h-3 mr-1" /> Retry
                            </Button>
                        ) : nodeStatus === "Stopped" ? (
                            <Button onClick={handleStartNodeClick} size="sm" className="h-8 text-xs">
                                <Zap className="w-3 h-3 mr-1 fill-current" /> Start
                            </Button>
                        ) : isConnecting ? (
                            <div className="flex items-center gap-2 px-3 py-1 rounded-lg bg-orange-500/10 text-orange-500 border border-orange-500/20 text-[10px] font-bold uppercase">
                                <RefreshCw className="w-3 h-3 animate-spin" /> Connecting
                            </div>
                        ) : (
                            <div className="flex items-center gap-2 px-3 py-1 rounded-lg bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 text-[10px] font-bold uppercase">
                                <span className="relative flex h-1.5 w-1.5">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-emerald-500"></span>
                                </span>
                                Online
                            </div>
                        )
                    )}
                </div>
            </div>

            {/* Combined Metrics Grid - Flexible Height */}
            <div className="lg:flex-1 lg:min-h-0 flex flex-col gap-4">
                {/* 1st Row of Cards */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 lg:flex-1 lg:min-h-0">
                    <StatCard
                        title="Network Status"
                        value={
                            <span className={cn("text-lg truncate", isError ? "text-red-500" : isActive ? "text-emerald-500" : "text-muted-foreground")}>
                                {nodeStatus}
                            </span>
                        }
                        icon={Globe}
                        statusColor={isError ? "text-red-500" : isActive ? "text-emerald-500" : "text-foreground"}
                        footerLabel="Relay"
                        footerValue={peers > 0 ? "Connected" : isError ? "Failed" : "Waiting"}
                    />
                    <StatCard
                        title="Latest Block"
                        value={`#${formatNumber(latestBlock?.index ?? 0, false)}`}
                        icon={Layers}
                        footerLabel="Hash"
                        footerValue={latestBlock?.hash ? `${latestBlock.hash.substring(0, 6)}..` : "---"}
                    />
                    <StatCard
                        title="My Shard"
                        value={`#${selfNodeInfo?.shard_id ?? 0}`}
                        icon={Hash}
                        statusColor="text-blue-500"
                        footerLabel="Total Shards"
                        footerValue={selfNodeInfo?.total_shards ?? 1}
                    />
                    <StatCard
                        title="Local Mined"
                        value={formatNumber(minedBlocks, false)}
                        icon={Pickaxe}
                        footerLabel="Role"
                        footerValue={minedBlocks > 0 ? "Miner" : "Validator"}
                    />
                </div>

                {/* 2nd Row of Cards */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 lg:flex-1 lg:min-h-0">
                    <StatCard
                        title="Shard Speed"
                        value={`${formatNumber(selfNodeInfo?.shard_tps_limit ?? 300)}`}
                        subValue="TPS"
                        icon={Cpu}
                        statusColor="text-orange-500"
                        footerLabel="Slot Time"
                        footerValue="2s"
                    />
                    <StatCard
                        title="Global Speed"
                        value={`${formatNumber(selfNodeInfo?.global_tps_capacity ?? 300)}`}
                        subValue="TPS"
                        icon={Server}
                        statusColor="text-purple-500"
                        footerLabel="Capacity"
                        footerValue="Unbounded"
                    />
                    <StatCard
                        title="Active Peers"
                        value={peers}
                        icon={Network}
                        footerLabel="Topology"
                        footerValue="Mesh"
                    />
                    <StatCard
                        title="VDF Speed"
                        value={formatNumber(vdfStatus?.iterations_per_second ?? 0, false)}
                        subValue="iter/s"
                        icon={Gauge}
                        footerLabel="Difficulty"
                        footerValue={formatNumber(vdfStatus?.difficulty ?? 0, false)}
                    />
                </div>
            </div>

            {/* Large Visualizer Area - Occupies remaining 40-50% of screen */}
            <div className="lg:flex-[1.2] lg:min-h-0 grid grid-cols-1 lg:grid-cols-3 gap-4">
                {/* Patience Accumulator - 2/3 width */}
                <div className="lg:col-span-2 glass-card rounded-2xl p-6 flex flex-col justify-center items-center relative overflow-hidden h-full group shadow-xl shadow-primary/5">
                    <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-transparent opacity-50 group-hover:opacity-100 transition-opacity" />
                    <div className="absolute top-4 left-6 flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-widest">
                        <Activity className="w-4 h-4 text-primary" /> Patience Accumulator
                    </div>

                    <div className="relative z-10 flex flex-col items-center">
                        <div className="text-6xl sm:text-7xl lg:text-8xl font-black tracking-tighter text-foreground drop-shadow-md">
                            {formatPatience(patience)}
                        </div>
                        <div className="flex items-center gap-3 mt-4">
                            <span className={cn("h-2 w-2 rounded-full", isActive ? "bg-emerald-500 animate-pulse" : "bg-muted-foreground")}></span>
                            <p className="text-xs font-medium text-muted-foreground">
                                {isActive ? "Node is active and participating in network consensus" : "Connect to network to start accumulating patience"}
                            </p>
                        </div>
                    </div>

                    {vdfStatus?.is_active && (
                        <div className="absolute bottom-0 left-0 w-full h-1 bg-secondary overflow-hidden">
                            <motion.div
                                animate={{ x: ['-100%', '300%'] }}
                                transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
                                className="w-1/3 h-full bg-primary"
                            />
                        </div>
                    )}
                </div>

                {/* Consensus Status Card - Dynamic Replacement */}
                <div className={cn(
                    "glass-card rounded-2xl p-6 flex flex-col justify-between h-full shadow-xl transition-all duration-500 relative overflow-hidden",
                    consensusStatus?.state === "Leader" ? "bg-emerald-500/10 border-emerald-500/20 shadow-emerald-500/10 from-emerald-500/5 to-transparent bg-gradient-to-br" :
                        consensusStatus?.state === "Queue" ? "bg-orange-500/10 border-orange-500/20 shadow-orange-500/10 from-orange-500/5 to-transparent bg-gradient-to-br" :
                            "bg-red-500/10 border-red-500/20 shadow-red-500/10 from-red-500/5 to-transparent bg-gradient-to-br"
                )}>
                    {/* Background Texture/Glow */}
                    <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-white/5 to-transparent rounded-full blur-2xl -mr-10 -mt-10 pointer-events-none" />

                    <div className="flex justify-between items-center z-10">
                        <span className="text-xs font-bold text-muted-foreground uppercase tracking-widest">Consensus Status</span>
                        {consensusStatus?.state === "Leader" ? (
                            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/20 border border-emerald-500/20 text-emerald-400 text-[10px] font-bold uppercase animate-pulse shadow-[0_0_10px_rgba(16,185,129,0.2)]">
                                <Zap className="w-3 h-3" /> Active Miner
                            </div>
                        ) : consensusStatus?.state === "Queue" ? (
                            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-orange-500/20 border border-orange-500/20 text-orange-400 text-[10px] font-bold uppercase shadow-[0_0_10px_rgba(249,115,22,0.2)]">
                                <Clock className="w-3 h-3" /> Standby
                            </div>
                        ) : (
                            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-red-500/20 border border-red-500/20 text-red-500 text-[10px] font-bold uppercase shadow-[0_0_10px_rgba(239,68,68,0.2)]">
                                <Shield className="w-3 h-3" /> Quarantine
                            </div>
                        )}
                    </div>

                    <div className="flex-1 flex flex-col justify-center items-center gap-4 py-4 z-10">
                        {!consensusStatus ? (
                            <div className="flex flex-col items-center gap-3 animate-pulse">
                                <div className="w-12 h-12 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin" />
                                <span className="text-xs font-bold text-muted-foreground tracking-wider">SYNCHRONIZING NETWORK...</span>
                            </div>
                        ) : consensusStatus.state === "Leader" ? (
                            <div className="flex flex-col items-center gap-2">
                                <div className="relative">
                                    <div className="absolute inset-0 bg-emerald-500 blur-2xl opacity-20 animate-pulse" />
                                    <Zap className="w-16 h-16 text-emerald-400 drop-shadow-[0_0_15px_rgba(52,211,153,0.5)] fill-emerald-400/20" />
                                </div>
                                <div className="text-center">
                                    <span className="block text-3xl font-black text-emerald-400 drop-shadow-sm tracking-tight">LEADER NODE</span>
                                    <span className="text-xs font-mono text-emerald-500/80 mt-1 uppercase tracking-wider block">Block Production Active</span>
                                </div>
                            </div>
                        ) : consensusStatus.state === "Queue" ? (
                            <div className="flex flex-col items-center gap-1">
                                <span className="text-[10px] font-bold text-orange-500/70 uppercase tracking-widest mb-1">Current Position</span>
                                <span className="text-6xl font-black text-transparent bg-clip-text bg-gradient-to-b from-orange-400 to-orange-600 drop-shadow-sm leading-none">
                                    #{consensusStatus.queue_position}
                                </span>
                                <div className="mt-4 flex items-center gap-2 text-xs font-mono bg-orange-500/10 border border-orange-500/20 px-3 py-1.5 rounded-lg text-orange-400">
                                    <Clock className="w-3 h-3" />
                                    <span>Est. Wait: <span className="font-bold text-orange-300">{consensusStatus.estimated_blocks} Blocks</span></span>
                                </div>
                            </div>
                        ) : (
                            // Patience State with Circular Timer
                            <div className="relative flex items-center justify-center">
                                {/* Rotating Outer Ring */}
                                <motion.div
                                    animate={{ rotate: 360 }}
                                    transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
                                    className="absolute inset-[-10px] w-40 h-40 rounded-full border border-red-500/20 border-t-red-500/50"
                                />

                                {/* Circular Progress */}
                                <svg className="w-32 h-32 transform -rotate-90">
                                    <circle
                                        cx="64" cy="64" r="60"
                                        stroke="currentColor"
                                        strokeWidth="4"
                                        fill="transparent"
                                        className="text-red-900/20"
                                    />
                                    <circle
                                        cx="64" cy="64" r="60"
                                        stroke="currentColor"
                                        strokeWidth="4"
                                        fill="transparent"
                                        strokeDasharray={377} // 2 * pi * 60
                                        strokeDashoffset={377 * (1 - (consensusStatus.patience_progress || 0))}
                                        strokeLinecap="round"
                                        className="text-red-500 drop-shadow-[0_0_8px_rgba(239,68,68,0.5)] transition-all duration-1000 ease-linear"
                                    />
                                </svg>

                                <div className="absolute inset-0 flex flex-col items-center justify-center">
                                    <span className="text-[10px] font-bold text-red-500 uppercase tracking-widest mb-1">Patience</span>
                                    <span className="text-xl font-mono font-black text-red-400 tabular-nums tracking-tighter">
                                        {new Date((consensusStatus.remaining_seconds ?? 0) * 1000).toISOString().substring(11, 19)}
                                    </span>
                                    <span className="text-[9px] text-red-500/60 mt-1 uppercase font-bold">Time Remaining</span>
                                </div>
                            </div>
                        )}
                    </div>

                    <div className="space-y-3 shrink-0">
                        {consensusStatus?.state === "Patience" && (
                            <div className="space-y-1">
                                <div className="flex justify-between text-[10px] font-mono">
                                    <span className="text-muted-foreground">PATIENCE PROGRESS</span>
                                    <span className="text-red-500 font-bold">{((consensusStatus.patience_progress || 0) * 100).toFixed(1)}%</span>
                                </div>
                                <div className="w-full h-1.5 bg-muted/30 rounded-full overflow-hidden">
                                    <motion.div
                                        initial={{ width: 0 }}
                                        animate={{ width: `${(consensusStatus.patience_progress || 0) * 100}%` }}
                                        className="h-full bg-red-500"
                                    />
                                </div>
                            </div>
                        )}

                        <div className="flex justify-between text-[10px] font-mono pt-2 border-t border-border/30">
                            <span className="text-muted-foreground">REAL-TIME STATUS</span>
                            <span className="text-foreground animate-pulse">LIVE</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
