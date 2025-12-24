import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Badge } from "../components/ui/badge.tsx";
import PageTransition from "../components/PageTransition";
import {
    Coins,
    TrendingDown,
    Zap,
    Gauge,
    PieChart,
    Timer,
    Calendar,
    ArrowUpRight,
    Lock,
    Unlock,
    Activity,
    ChevronRight
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

    // TPS Calculation logic (moved from Network.tsx)
    const currentTPS = useMemo(() => {
        if (peers.length === 0) return 0;
        const verifiedCount = peers.filter(p => p.is_verified).length;
        return 50000 + (verifiedCount * 5000);
    }, [peers]);

    const supplyPercentage = useMemo(() => {
        if (!tokenomics) return 0;
        return (tokenomics.circulating_supply / tokenomics.total_supply) * 100;
    }, [tokenomics]);

    const halvingProgress = useMemo(() => {
        if (!tokenomics) return 0;
        const currentInInterval = tokenomics.halving_interval - tokenomics.blocks_until_halving;
        return (currentInInterval / tokenomics.halving_interval) * 100;
    }, [tokenomics]);

    const timeToHalving = useMemo(() => {
        if (!tokenomics) return "Estimating...";
        // Assumes 10s block time
        const totalSeconds = tokenomics.blocks_until_halving * 10;
        const days = Math.floor(totalSeconds / (24 * 3600));
        const hours = Math.floor((totalSeconds % (24 * 3600)) / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);

        if (days > 0) return `${days}d ${hours}h ${minutes}m`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        return `${minutes}m`;
    }, [tokenomics]);

    return (
        <PageTransition>
            <div className="flex flex-col gap-4 sm:gap-5 min-h-full container mx-auto p-4 sm:p-6 lg:max-w-7xl pb-10">
                {/* Header Section */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 px-2">
                    <div className="flex items-center gap-4">
                        <div className="p-3 bg-primary/10 rounded-2xl border border-primary/20 shadow-inner">
                            <PieChart className="w-6 h-6 text-primary" />
                        </div>
                        <div>
                            <h1 className="text-2xl font-extrabold tracking-tight text-foreground">Economic Protocol</h1>
                            <p className="text-xs text-muted-foreground flex items-center gap-1.5 font-semibold">
                                <Coins className="w-3 h-3" /> Real-time scarcity analysis & supply dynamics
                            </p>
                        </div>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 md:gap-6">
                    {/* Supply Allocation */}
                    <Card className="lg:col-span-2 border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[2rem] overflow-hidden">
                        <CardHeader className="bg-primary/5 py-4 px-6 border-b border-primary/10 flex flex-row items-center justify-between">
                            <div className="flex items-center gap-3">
                                <PieChart className="w-5 h-5 text-primary" />
                                <CardTitle className="text-[10px] font-bold uppercase tracking-[0.2em] text-primary/90">Supply Distribution</CardTitle>
                            </div>
                            <Badge className="bg-emerald-500/10 text-emerald-500 border-emerald-500/20 font-black">FIXED CAP: 21M</Badge>
                        </CardHeader>
                        <CardContent className="p-4 md:p-5 lg:p-6 flex flex-col min-h-0">
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 lg:gap-6 flex-1 min-h-0">
                                <div className="space-y-3 md:space-y-4 flex flex-col justify-center">
                                    <div className="space-y-1">
                                        <div className="flex justify-between items-end">
                                            <label className="text-[9px] font-black text-muted-foreground uppercase tracking-widest">Circulating (Mined)</label>
                                            <span className="text-base md:text-lg font-black text-foreground">
                                                {((tokenomics?.circulating_supply || 0) / AGT_DIVISOR).toLocaleString(undefined, { maximumFractionDigits: 0 })} <span className="text-[9px] opacity-60">AGT</span>
                                            </span>
                                        </div>
                                        <div className="h-2 w-full bg-muted/30 rounded-full overflow-hidden border border-primary/5">
                                            <div
                                                className="h-full bg-gradient-to-r from-primary to-emerald-500 transition-all duration-1000 shadow-[0_0_15px_rgba(var(--primary),0.3)]"
                                                style={{ width: `${supplyPercentage}%` }}
                                            />
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-2 gap-3 md:gap-4">
                                        <div className="p-3 md:p-4 bg-primary/5 rounded-2xl border border-primary/10">
                                            <div className="flex items-center gap-2 mb-0.5 md:mb-1">
                                                <Unlock className="w-3 h-3 text-emerald-500" />
                                                <span className="text-[9px] font-black text-muted-foreground uppercase tracking-tighter sm:tracking-normal">Liquidity</span>
                                            </div>
                                            <div className="text-base md:text-lg font-black text-foreground">{supplyPercentage.toFixed(2)}%</div>
                                        </div>
                                        <div className="p-3 md:p-4 bg-orange-500/5 rounded-2xl border border-orange-500/10">
                                            <div className="flex items-center gap-2 mb-0.5 md:mb-1">
                                                <Lock className="w-3 h-3 text-orange-500" />
                                                <span className="text-[9px] font-black text-muted-foreground uppercase tracking-tighter sm:tracking-normal">Remaining</span>
                                            </div>
                                            <div className="text-base md:text-lg font-black text-foreground">{(100 - supplyPercentage).toFixed(2)}%</div>
                                        </div>
                                    </div>

                                    <div className="p-3 md:p-4 bg-muted/10 rounded-2xl border border-primary/5 space-y-1.5 md:space-y-2">
                                        <div className="flex justify-between text-[11px] items-center">
                                            <span className="font-bold text-muted-foreground">Original Genesis</span>
                                            <span className="font-black">5,000,000 AGT</span>
                                        </div>
                                        <div className="flex justify-between text-[11px] items-center">
                                            <span className="font-bold text-muted-foreground">Mining Allocation</span>
                                            <span className="font-black">16,000,000 AGT</span>
                                        </div>
                                        <div className="pt-1.5 border-t border-primary/5 flex justify-between text-[11px] items-center">
                                            <span className="font-bold text-primary">Total Hard Cap</span>
                                            <span className="font-black text-primary">21,000,000 AGT</span>
                                        </div>
                                    </div>
                                </div>

                                <div className="flex flex-col justify-center items-center p-3 lg:p-5 bg-primary/5 rounded-2xl lg:rounded-3xl border border-primary/10 relative group min-h-0">
                                    <Activity className="absolute top-2.5 right-2.5 lg:top-3 lg:right-3 w-4 lg:w-5 h-4 lg:h-5 text-primary cursor-help opacity-40" />
                                    <div className="text-[8px] font-black text-primary uppercase tracking-[0.2em] mb-1 md:mb-1.5">Market Multiplier</div>
                                    <div className="text-2xl md:text-3xl lg:text-4xl font-black text-foreground tracking-tighter mb-1.5 md:mb-3">
                                        {(21000000 / ((tokenomics?.circulating_supply || 5000000) / AGT_DIVISOR)).toFixed(1)}x
                                    </div>
                                    <p className="text-[8px] text-muted-foreground text-center font-bold px-4 leading-tight">
                                        Scarcity ratio based on block #{height?.toLocaleString()}. Sprint-restricted supply.
                                    </p>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* TPS Real-time Gauge */}
                    <Card className="border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[2rem] overflow-hidden flex flex-col min-h-0">
                        <CardHeader className="bg-primary/5 py-3 md:py-4 px-6 border-b border-primary/10 shrink-0">
                            <div className="flex items-center gap-3">
                                <Gauge className="w-5 h-5 text-primary" />
                                <CardTitle className="text-[10px] font-bold uppercase tracking-[0.2em] text-primary/90">Network Throughput</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent className="p-4 md:p-6 lg:p-8 flex flex-col items-center justify-center text-center space-y-3 md:space-y-4 flex-1 min-h-0">
                            <div className="relative shrink-0">
                                <div className="w-28 md:w-32 lg:w-40 h-28 md:h-32 lg:h-40 rounded-full border-[10px] md:border-[12px] border-muted/20 flex flex-col items-center justify-center overflow-hidden">
                                    <div className="text-2xl md:text-3xl font-black text-foreground tracking-tighter leading-none">
                                        {(currentTPS / 1000).toFixed(0)}k
                                    </div>
                                    <div className="text-[9px] font-black text-primary uppercase mt-1">TPS Cap</div>

                                    <svg className="absolute inset-0 -rotate-90 w-full h-full">
                                        <circle
                                            cx="50%" cy="50%" r="44%"
                                            fill="transparent"
                                            stroke="currentColor"
                                            strokeWidth="10"
                                            className="text-primary/20"
                                        />
                                        <circle
                                            cx="50%" cy="50%" r="44%"
                                            fill="transparent"
                                            stroke="url(#tpsGradient)"
                                            strokeWidth="10"
                                            strokeDasharray="270%"
                                            strokeDashoffset={`${270 - (270 * (currentTPS / 150000))}%`}
                                            strokeLinecap="round"
                                            className="transition-all duration-1000"
                                        />
                                    </svg>
                                </div>
                            </div>

                            <div className="space-y-3 w-full flex-1 flex flex-col justify-center min-h-0">
                                <div className="space-y-1 shrink-0">
                                    <div className="text-[9px] font-black text-muted-foreground uppercase tracking-widest">Real-time Performance</div>
                                    <div className="text-xs font-bold text-foreground flex items-center justify-center gap-2">
                                        <Zap className="w-3.5 h-3.5 text-emerald-500 fill-emerald-500/20" />
                                        Parallel Execution Active
                                    </div>
                                </div>
                                <div className="p-3 md:p-4 bg-muted/10 rounded-2xl border border-primary/5 text-left shrink-0">
                                    <div className="flex justify-between items-center text-[9px] font-bold uppercase text-muted-foreground mb-1.5 md:mb-2">
                                        <span>Capacity breakdown</span>
                                        <ArrowUpRight className="w-3 h-3" />
                                    </div>
                                    <div className="space-y-1 md:space-y-1.5">
                                        <div className="flex justify-between text-[11px]">
                                            <span>Base Performance</span>
                                            <span className="font-black italic">50k TPS</span>
                                        </div>
                                        <div className="flex justify-between text-[11px]">
                                            <span>Verified Nodes</span>
                                            <span className="font-black italic text-emerald-500">+{((peers.filter(p => p.is_verified).length) * 5000 / 1000).toLocaleString()}k TPS</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Halving Countdown */}
                    <Card className="lg:col-span-2 border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[2rem] overflow-hidden">
                        <CardHeader className="bg-primary/5 py-4 px-6 border-b border-primary/10">
                            <div className="flex items-center gap-3">
                                <Timer className="w-5 h-5 text-orange-500" />
                                <CardTitle className="text-[10px] font-bold uppercase tracking-[0.2em] text-orange-500/90">Sprint Halving Countdown</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent className="p-4 md:p-6 lg:p-8">
                            <div className="flex flex-col md:flex-row gap-6 lg:gap-10">
                                <div className="flex-1 space-y-4 md:space-y-6">
                                    <div className="space-y-1.5 md:space-y-2">
                                        <div className="flex justify-between items-end">
                                            <label className="text-[10px] font-black text-muted-foreground uppercase tracking-widest">Next Reward Reduction</label>
                                            <span className="text-base md:text-lg font-black text-foreground">
                                                {tokenomics?.blocks_until_halving?.toLocaleString()} <span className="text-[10px] opacity-60">Blocks Left</span>
                                            </span>
                                        </div>
                                        <div className="h-3 md:h-4 w-full bg-muted/30 rounded-full overflow-hidden border border-orange-500/5">
                                            <div
                                                className="h-full bg-gradient-to-r from-orange-500 to-amber-500 transition-all duration-1000 shadow-[0_0_15px_rgba(249,115,22,0.3)]"
                                                style={{ width: `${halvingProgress}%` }}
                                            />
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-2 gap-4 md:gap-6">
                                        <div className="space-y-0.5 md:space-y-1">
                                            <span className="text-[9px] font-black text-muted-foreground uppercase">Current Subsidy</span>
                                            <div className="text-xl md:text-2xl font-black text-foreground flex items-center gap-1.5">
                                                {((tokenomics?.current_reward || 0) / AGT_DIVISOR).toFixed(2)}
                                                <Badge className="bg-primary/10 text-primary border-none text-[7px] h-4">AGT</Badge>
                                            </div>
                                        </div>
                                        <div className="space-y-0.5 md:space-y-1">
                                            <span className="text-[9px] font-black text-muted-foreground uppercase">Estimated Time</span>
                                            <div className="text-xl md:text-2xl font-black text-orange-500">{timeToHalving}</div>
                                        </div>
                                    </div>
                                </div>

                                <div className="p-4 bg-orange-500/5 rounded-2xl md:rounded-3xl border border-orange-500/10 flex flex-col justify-between w-full md:w-56 lg:w-64">
                                    <div className="space-y-0.5">
                                        <div className="text-[9px] font-black text-orange-500 uppercase tracking-widest mb-0.5">Target Height</div>
                                        <div className="text-xl md:text-2xl font-black text-foreground">#{tokenomics?.next_halving_at?.toLocaleString()}</div>
                                    </div>
                                    <div className="pt-3 border-t border-orange-500/10 mt-2">
                                        <div className="flex items-center gap-1.5 text-[9px] font-bold text-muted-foreground mb-1.5">
                                            <TrendingDown className="w-3 h-3" />
                                            Reward drop: {((tokenomics?.current_reward || 0) / (2 * AGT_DIVISOR)).toFixed(2)} AGT
                                        </div>
                                        <div className="text-[8px] text-muted-foreground leading-tight italic">
                                            Sprint-based scarcity model rewarded early providers.
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Halving Schedule */}
                    <Card className="border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[2rem] overflow-hidden flex flex-col min-h-0">
                        <CardHeader className="bg-primary/5 py-3 md:py-4 px-6 border-b border-primary/10 shrink-0">
                            <div className="flex items-center gap-3">
                                <Calendar className="w-5 h-5 text-primary" />
                                <CardTitle className="text-[10px] font-bold uppercase tracking-[0.2em] text-primary/90">Protocol Schedule</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent className="p-0 flex-1 min-h-0 overflow-y-auto custom-scrollbar">
                            <div className="p-3 md:p-4 space-y-1.5">
                                {[
                                    { era: "Genesis", height: 0, reward: 5_000_000, type: "distribution" },
                                    { era: "Early Phase 1", height: 100_000, reward: 20, type: "halving" },
                                    { era: "Early Phase 2", height: 200_000, reward: 10, type: "halving" },
                                    { era: "Early Phase 3", height: 300_000, reward: 5, type: "halving" },
                                    { era: "Early Phase 4", height: 400_000, reward: 2.5, type: "halving" },
                                    { era: "Early Phase 5", height: 500_000, reward: 1.25, type: "halving" },
                                    { era: "Stable Phase 1", height: 900_000, reward: 0.625, type: "halving" },
                                    { era: "Stable Phase 2", height: 1_300_000, reward: 0.312, type: "halving" },
                                ].map((item, i) => (
                                    <div key={i} className={cn(
                                        "flex items-center justify-between p-3 rounded-xl border transition-all cursor-default",
                                        height >= item.height && item.height !== 0 ? "bg-primary/10 border-primary/20" : "bg-muted/5 border-transparent opacity-60 grayscale"
                                    )}>
                                        <div className="flex items-center gap-3">
                                            <div className={cn(
                                                "w-8 h-8 rounded-lg flex items-center justify-center text-[10px] font-black",
                                                height >= item.height && item.height !== 0 ? "bg-primary text-primary-foreground shadow-lg" : "bg-muted text-muted-foreground"
                                            )}>
                                                {i + 1}
                                            </div>
                                            <div>
                                                <div className="text-[10px] font-black uppercase tracking-tight">{item.era}</div>
                                                <div className="text-[9px] text-muted-foreground font-mono">#{item.height.toLocaleString()}</div>
                                            </div>
                                        </div>
                                        <div className="text-right">
                                            <div className="text-xs font-black text-foreground">{item.reward.toLocaleString()} AGT</div>
                                            <div className="text-[8px] font-black uppercase text-muted-foreground/60">{item.type}</div>
                                        </div>
                                    </div>
                                ))}
                                <div className="p-4 text-center">
                                    <button className="text-[10px] font-black text-primary uppercase flex items-center gap-2 mx-auto hover:gap-3 transition-all">
                                        View Full Economic Map <ChevronRight className="w-3 h-3" />
                                    </button>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </PageTransition>
    );
}
