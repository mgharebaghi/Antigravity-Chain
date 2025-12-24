import { Badge } from "../components/ui/badge";
import {
    PieChart,
    Timer,
    Calendar,
    Lock,
    Unlock,
    Activity,
    Gauge,
    Coins,
    TrendingUp
} from "lucide-react";
import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "../lib/utils";
import { useApp } from "../context/AppContext";

interface TokenomicsInfo {
    total_supply: number;
    max_supply: number;
    circulating_supply: number;
    remaining_supply: number;
    next_halving_at: number;
    blocks_until_halving: number;
    current_reward: number;
    halving_interval: number;
}

interface PeerInfo {
    peer_id: string;
    is_verified: boolean;
}

export default function Tokenomics() {
    const [tokenomics, setTokenomics] = useState<TokenomicsInfo | null>(null);
    const [peers, setPeers] = useState<PeerInfo[]>([]);
    const { height } = useApp();

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [tInfo, pInfo] = await Promise.all([
                    invoke<TokenomicsInfo>("get_tokenomics_info"),
                    invoke<PeerInfo[]>("get_network_info")
                ]);
                setTokenomics(tInfo);
                setPeers(pInfo);
            } catch (e) {
                console.error("Failed to fetch tokenomics:", e);
            }
        };

        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    const AGT_DIVISOR = 1000000;

    const currentTPS = useMemo(() => {
        if (peers.length === 0) return 0;
        const verifiedCount = peers.filter(p => p.is_verified).length;
        return 50000 + (verifiedCount * 5000);
    }, [peers]);

    const supplyPercentage = useMemo(() => {
        if (!tokenomics) return 0;
        const circulating = tokenomics.circulating_supply || 0;
        const total = tokenomics.total_supply || 21000000 * AGT_DIVISOR; // fallback if total is 0
        return (circulating / total) * 100;
    }, [tokenomics]);

    const halvingProgress = useMemo(() => {
        if (!tokenomics) return 0;
        const currentInInterval = tokenomics.halving_interval - tokenomics.blocks_until_halving;
        return (currentInInterval / tokenomics.halving_interval) * 100;
    }, [tokenomics]);

    const timeToHalving = useMemo(() => {
        if (!tokenomics) return "Estimating...";
        const totalSeconds = tokenomics.blocks_until_halving * 10;
        const days = Math.floor(totalSeconds / (24 * 3600));
        const hours = Math.floor((totalSeconds % (24 * 3600)) / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);

        if (days > 0) return `${days}d ${hours}h`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        return `${minutes}m`;
    }, [tokenomics]);

    return (
        <div className="flex flex-col gap-4 h-full pb-2 overflow-hidden">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0">
                <div>
                    <h1 className="text-2xl font-bold tracking-tight">Tokenomics</h1>
                    <p className="text-muted-foreground text-xs">Economic supply parameters and protocol emission data.</p>
                </div>
                <div className="flex items-center gap-2 bg-secondary/30 px-3 py-1 rounded-full border border-border/50">
                    <Coins className="w-3.5 h-3.5 text-primary" />
                    <span className="text-xs font-semibold text-foreground">
                        {((tokenomics?.circulating_supply || 0) / AGT_DIVISOR).toLocaleString(undefined, { maximumFractionDigits: 0 })} AGT
                    </span>
                    <span className="text-[10px] text-muted-foreground uppercase ml-1">Circulating</span>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1 min-h-0">

                {/* Supply Allocation Card */}
                <div className="lg:col-span-2 glass-card rounded-2xl p-5 flex flex-col gap-4 relative overflow-hidden justify-between">
                    <div className="absolute top-0 right-0 w-64 h-64 bg-primary/5 rounded-full -mr-32 -mt-32 blur-3xl pointer-events-none" />

                    <div className="flex items-center justify-between z-10 shrink-0">
                        <div className="flex items-center gap-2">
                            <div className="p-2 bg-primary/10 text-primary rounded-lg border border-primary/20 shadow-sm">
                                <PieChart className="w-4 h-4" />
                            </div>
                            <div>
                                <h3 className="font-bold text-sm leading-tight">Supply Distribution</h3>
                                <p className="text-[10px] text-muted-foreground">Verification of total cap</p>
                            </div>
                        </div>
                        <Badge variant="outline" className="font-mono text-[10px] bg-secondary/50 backdrop-blur-md">Max Cap: 21M AGT</Badge>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 items-center z-10 flex-1">
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <div className="flex justify-between items-baseline">
                                    <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">Circulating Supply</span>
                                    <span className="text-2xl font-black font-mono tracking-tight text-foreground">
                                        {((tokenomics?.circulating_supply || 0) / AGT_DIVISOR).toLocaleString(undefined, { maximumFractionDigits: 0 })}
                                    </span>
                                </div>
                                <div className="h-3 w-full bg-secondary/50 rounded-full overflow-hidden border border-white/5 shadow-inner">
                                    <div
                                        className="h-full bg-gradient-to-r from-primary to-purple-400 rounded-full transition-all duration-1000 shadow-[0_0_20px_rgba(168,85,247,0.4)]"
                                        style={{ width: `${supplyPercentage}%` }}
                                    />
                                </div>
                            </div>

                            <div className="grid grid-cols-2 gap-3">
                                <div className="p-3 bg-secondary/20 rounded-xl border border-white/5 hover:bg-secondary/30 transition-colors">
                                    <div className="flex items-center gap-1.5 mb-1 text-muted-foreground">
                                        <Unlock className="w-3 h-3" />
                                        <span className="text-[9px] font-black uppercase tracking-wider">Mined</span>
                                    </div>
                                    <div className="text-lg font-black text-foreground">{supplyPercentage.toFixed(1)}%</div>
                                </div>
                                <div className="p-3 bg-secondary/20 rounded-xl border border-white/5 hover:bg-secondary/30 transition-colors">
                                    <div className="flex items-center gap-1.5 mb-1 text-muted-foreground">
                                        <Lock className="w-3 h-3" />
                                        <span className="text-[9px] font-black uppercase tracking-wider">Locked</span>
                                    </div>
                                    <div className="text-lg font-black text-muted-foreground/50">{(100 - supplyPercentage).toFixed(1)}%</div>
                                </div>
                            </div>
                        </div>

                        <div className="bg-background/30 backdrop-blur-sm p-4 rounded-xl border border-white/10 space-y-3 shadow-xl relative overflow-hidden h-full flex flex-col justify-center">
                            <div className="absolute inset-0 bg-gradient-to-br from-white/5 to-transparent pointer-events-none" />
                            <h4 className="text-xs font-bold flex items-center gap-2 uppercase tracking-wide opacity-80 z-10 relative">
                                <Activity className="w-3.5 h-3.5 text-primary" /> Velocity Metrics
                            </h4>
                            <div className="space-y-2 z-10 relative">
                                <div className="flex justify-between items-center py-1 border-b border-border/30">
                                    <span className="text-[10px] text-muted-foreground font-medium">Genesis Premine</span>
                                    <span className="font-mono text-xs font-bold">5,000,000 AGT</span>
                                </div>
                                <div className="flex justify-between items-center py-1 border-b border-border/30">
                                    <span className="text-[10px] text-muted-foreground font-medium">Block Reward</span>
                                    <div className="flex items-center gap-2">
                                        <span className="font-mono text-xs font-bold text-emerald-500">+{((tokenomics?.current_reward || 0) / AGT_DIVISOR).toFixed(2)} AGT</span>
                                    </div>
                                </div>
                                <div className="flex justify-between items-center pt-1">
                                    <span className="text-[10px] text-muted-foreground font-medium">Scarcity Multiplier</span>
                                    <span className="font-mono text-base font-black text-primary">
                                        {(21000000 / ((tokenomics?.circulating_supply || 5000000) / AGT_DIVISOR)).toFixed(2)}x
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Halving Countdown Card */}
                <div className="glass-card rounded-2xl p-5 flex flex-col justify-between border-orange-500/20 bg-orange-500/5 relative overflow-hidden">
                    <div className="absolute inset-0 bg-gradient-to-b from-orange-500/10 to-transparent pointer-events-none" />

                    <div className="relative z-10">
                        <div className="flex items-center gap-2 border-b border-orange-500/20 pb-4 mb-4">
                            <div className="p-2 bg-orange-500/10 text-orange-500 rounded-lg border border-orange-500/20 shadow-glow-orange-sm">
                                <Timer className="w-4 h-4" />
                            </div>
                            <h3 className="font-bold text-sm text-foreground">Halving Event</h3>
                        </div>

                        <div className="text-center mb-6">
                            <span className="text-4xl font-black font-mono tracking-tighter tabular-nums block text-foreground mb-1 drop-shadow-sm">
                                {tokenomics?.blocks_until_halving?.toLocaleString()}
                            </span>
                            <span className="text-[9px] font-black uppercase tracking-[0.2em] text-orange-500 opacity-80">Blocks Remaining</span>
                        </div>

                        <div className="space-y-2 mb-6">
                            <div className="flex justify-between text-[9px] font-bold uppercase tracking-wider text-muted-foreground px-1">
                                <span>Cycle Progress</span>
                                <span>{halvingProgress.toFixed(1)}%</span>
                            </div>
                            <div className="h-2 w-full bg-background/50 rounded-full overflow-hidden border border-orange-500/20">
                                <div
                                    className="h-full bg-orange-500 rounded-full transition-all duration-1000 shadow-[0_0_15px_rgba(249,115,22,0.6)]"
                                    style={{ width: `${halvingProgress}%` }}
                                />
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-2">
                            <div className="p-3 rounded-xl bg-background/40 border border-orange-500/20 text-center backdrop-blur-sm">
                                <div className="text-[8px] font-black uppercase tracking-wider text-muted-foreground mb-1">ETA</div>
                                <div className="font-mono font-bold text-orange-500 text-xs whitespace-nowrap">{timeToHalving}</div>
                            </div>
                            <div className="p-3 rounded-xl bg-background/40 border border-border/50 text-center backdrop-blur-sm">
                                <div className="text-[8px] font-black uppercase tracking-wider text-muted-foreground mb-1">Next Drop</div>
                                <div className="font-mono font-bold text-foreground text-xs">
                                    {((tokenomics?.current_reward || 0) / (2 * AGT_DIVISOR)).toFixed(2)} AGT
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Throughput Card */}
                <div className="glass-card rounded-2xl p-5 flex flex-col items-center justify-center relative overflow-hidden bg-emerald-500/5 border-emerald-500/10">
                    <div className="absolute inset-x-0 top-0 h-32 bg-gradient-to-b from-emerald-500/10 to-transparent pointer-events-none" />
                    <div className="absolute inset-0 bg-[url('/grid.svg')] opacity-10" />

                    <div className="relative z-10 w-full flex flex-col items-center justify-center h-full">
                        <div className="flex items-center gap-2 mb-4 opacity-80">
                            <Gauge className="w-4 h-4 text-emerald-500" />
                            <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-600 dark:text-emerald-400">Network Capacity</span>
                        </div>

                        <div className="relative w-32 h-32 flex items-center justify-center mb-4">
                            {/* Background Track */}
                            <svg className="absolute inset-0 w-full h-full transform -rotate-90">
                                <circle
                                    cx="50%" cy="50%" r="42%"
                                    fill="transparent"
                                    stroke="currentColor"
                                    strokeWidth="8"
                                    className="text-emerald-500/10"
                                />
                                <circle
                                    cx="50%" cy="50%" r="42%"
                                    fill="transparent"
                                    stroke="currentColor"
                                    strokeWidth="8"
                                    strokeDasharray="264%"
                                    strokeDashoffset={`${264 - (264 * (currentTPS / 150000))}%`}
                                    strokeLinecap="round"
                                    className="text-emerald-500 transition-all duration-1000 drop-shadow-[0_0_10px_rgba(16,185,129,0.5)]"
                                />
                            </svg>
                            <div className="text-center z-10 flex flex-col items-center">
                                <span className="text-2xl font-black tracking-tighter text-foreground">{(currentTPS / 1000).toFixed(0)}k</span>
                                <span className="text-[8px] font-black uppercase tracking-[0.2em] text-muted-foreground mt-0.5">TPS</span>
                            </div>
                        </div>

                        <div className="flex items-center gap-1.5 text-[10px] font-medium text-muted-foreground bg-background/50 px-3 py-1 rounded-full border border-border/50">
                            <TrendingUp className="w-3 h-3 text-emerald-500" />
                            Based on validator count
                        </div>
                    </div>
                </div>

                {/* Emission Schedule Table */}
                <div className="lg:col-span-2 glass-card rounded-2xl overflow-hidden flex flex-col">
                    <div className="p-4 border-b border-border/50 bg-secondary/20 flex items-center gap-2 shrink-0">
                        <div className="p-1.5 bg-background rounded-lg border border-border/50">
                            <Calendar className="w-4 h-4 text-primary" />
                        </div>
                        <h3 className="font-bold text-sm">Emission Schedule</h3>
                    </div>

                    <div className="flex-1 overflow-x-auto overflow-y-auto scrollbar-thin">
                        <table className="w-full text-xs text-left border-collapse min-w-[500px]">
                            <thead className="text-[9px] font-black font-mono text-muted-foreground uppercase bg-secondary/30 tracking-wider sticky top-0 z-10 backdrop-blur-md">
                                <tr>
                                    <th className="px-5 py-3">Phase Identifier</th>
                                    <th className="px-5 py-3">Activation Height</th>
                                    <th className="px-5 py-3">Mechanism</th>
                                    <th className="px-5 py-3 text-right">Block Subsidy (AGT)</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-border/30">
                                {[
                                    { era: "Genesis Event", height: 0, reward: 5_000_000, type: "PRE-MINE" },
                                    { era: "Early Era (Phase 1)", height: 100_000, reward: 20, type: "POW-MINING" },
                                    { era: "Growth Era (Phase 2)", height: 200_000, reward: 10, type: "POW-MINING" },
                                    { era: "Mature Era (Phase 3)", height: 300_000, reward: 5, type: "POW-MINING" },
                                    { era: "Scarce Era (Phase 4)", height: 400_000, reward: 2.5, type: "POW-MINING" },
                                    { era: "Tail Emission", height: 500_000, reward: 1.25, type: "POW-MINING" },
                                ].map((item, i) => {
                                    const isActive = height >= item.height && (i === 5 || height < (item.height + 100000)); // Rough logic for highlighting
                                    return (
                                        <tr key={i} className={cn(
                                            "transition-colors group hover:bg-white/5",
                                            isActive ? "bg-primary/5" : ""
                                        )}>
                                            <td className="px-5 py-2.5 font-medium flex items-center gap-3">
                                                <div className={cn("w-1.5 h-1.5 rounded-full", isActive ? "bg-emerald-500 animate-pulse" : "bg-muted-foreground/30")} />
                                                <span className={cn(isActive ? "text-foreground font-bold" : "text-muted-foreground")}>{item.era}</span>
                                            </td>
                                            <td className="px-5 py-2.5 font-mono text-[10px] text-muted-foreground group-hover:text-foreground transition-colors">#{item.height.toLocaleString()}</td>
                                            <td className="px-5 py-2.5">
                                                <Badge variant="outline" className="text-[8px] font-mono font-normal opacity-70 bg-background/50">{item.type}</Badge>
                                            </td>
                                            <td className="px-5 py-2.5 text-right font-mono font-bold text-foreground">
                                                {item.reward.toLocaleString()}
                                            </td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
}
