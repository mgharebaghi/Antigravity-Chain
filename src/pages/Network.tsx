import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Badge } from "../components/ui/badge.tsx";
import PageTransition from "../components/PageTransition";
import {
    ShieldCheck,
    ShieldAlert,
    Activity,
    Globe,
    Zap,
    Server,
    Network as NetworkIcon,
    Link,
    Wifi,
    Terminal,
    Copy,
    Check,
} from "lucide-react";
import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "../lib/utils";
import { useApp } from "../context/AppContext";
import { useToast } from "../context/ToastContext";

interface PeerInfo {
    peer_id: string;
    trust_score: number;
    is_verified: boolean;
    latency: number;
    addresses: string[];
}

interface SelfNodeInfo {
    peer_id: string;
    addresses: string[];
}


// Simple deterministic color generator for peer IDs
const getPeerColor = (peerId: string) => {
    const hash = peerId.split('').reduce((acc, char) => char.charCodeAt(0) + ((acc << 5) - acc), 0);
    const h = Math.abs(hash % 360);
    return `hsl(${h}, 70%, 60%)`;
};

export default function Network() {
    const [peers, setPeers] = useState<PeerInfo[]>([]);
    const [selfInfo, setSelfInfo] = useState<SelfNodeInfo | null>(null);
    const [copiedId, setCopiedId] = useState<string | null>(null);
    const { height, relayStatus, connectedRelay } = useApp();
    const { success } = useToast();

    // Mock TPS calculation based on node count and theoretical capacity

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [pInfo, sInfo] = await Promise.all([
                    invoke<PeerInfo[]>("get_network_info"),
                    invoke<SelfNodeInfo | null>("get_self_node_info"),
                ]);
                setPeers(pInfo);
                setSelfInfo(sInfo);
            } catch (e) {
                console.error(e);
            }
        };

        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    const validatorPeers = useMemo(() => {
        if (!connectedRelay) return peers;
        return peers.filter(p => p.peer_id !== connectedRelay);
    }, [peers, connectedRelay]);

    const avgLatency = useMemo(() => {
        if (validatorPeers.length === 0) return 0;
        return Math.round(validatorPeers.reduce((acc, p) => acc + (p.latency || 0), 0) / validatorPeers.length);
    }, [validatorPeers]);

    const copyToClipboard = (text: string, id: string) => {
        navigator.clipboard.writeText(text);
        setCopiedId(id);
        success("Copied to clipboard");
        setTimeout(() => setCopiedId(null), 2000);
    };

    return (
        <PageTransition>
            <div className="flex flex-col gap-3 sm:gap-6 min-h-full container mx-auto p-4 sm:p-6 lg:max-w-7xl pb-10">
                {/* Header Section */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 sm:gap-4 shrink-0 px-1 sm:px-2">
                    <div className="flex items-center gap-2 sm:gap-4">
                        <div className="p-1.5 sm:p-3 bg-primary/10 rounded-xl sm:rounded-2xl border border-primary/20 shadow-inner shrink-0">
                            <NetworkIcon className="w-4 h-4 sm:w-6 sm:h-6 text-primary" />
                        </div>
                        <div>
                            <h1 className="text-lg sm:text-2xl font-extrabold tracking-tight text-foreground leading-none">Mesh Infrastructure</h1>
                            <p className="text-[9px] sm:text-xs text-muted-foreground flex items-center gap-1.5 font-semibold mt-1">
                                <Globe className="w-2.5 h-2.5 sm:w-3 h-3" /> Real-time peer-to-peer technical diagnostics
                            </p>
                        </div>
                    </div>
                </div>

                <div className="flex flex-col sm:flex-row gap-3 sm:gap-4 shrink-0 px-1">
                    {/* Self Node Technical Details */}
                    <Card className="flex-1 border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[1.2rem] sm:rounded-[2rem] overflow-hidden min-h-0 flex flex-col">
                        <CardHeader className="bg-primary/5 py-1.5 sm:py-3 px-3 sm:px-5 border-b border-primary/10 flex flex-row items-center gap-2 sm:gap-3 shrink-0">
                            <Server className="w-3.5 h-3.5 sm:w-4 h-4 text-primary" />
                            <CardTitle className="text-[8px] sm:text-[10px] font-bold uppercase tracking-[0.1em] sm:tracking-[0.15em] text-primary/90">Identity</CardTitle>
                        </CardHeader>
                        <CardContent className="p-3 sm:p-4 space-y-2 sm:space-y-3 min-h-0 flex-1 flex flex-col overflow-hidden">
                            {selfInfo ? (
                                <>
                                    <div className="space-y-1.5">
                                        <div className="flex justify-between items-center px-1">
                                            <label className="text-[9px] font-black uppercase text-muted-foreground tracking-tighter">Your Peer ID</label>
                                            <button
                                                onClick={() => copyToClipboard(selfInfo.peer_id, 'peer_id')}
                                                className="hover:text-primary transition-colors"
                                            >
                                                {copiedId === 'peer_id' ? <Check className="w-2.5 h-2.5 text-green-500" /> : <Copy className="w-2.5 h-2.5" />}
                                            </button>
                                        </div>
                                        <div className="p-2 sm:p-3 bg-muted/30 rounded-lg sm:rounded-xl border border-primary/5 font-mono text-[9px] sm:text-[11px] font-bold text-foreground/80 break-all leading-tight sm:leading-relaxed">
                                            {selfInfo.peer_id}
                                        </div>
                                    </div>
                                    <div className="space-y-1 flex-1 min-h-0 flex flex-col overflow-hidden text-xs">
                                        <label className="text-[8px] font-bold uppercase text-muted-foreground/80 tracking-widest px-1 shrink-0">Multiaddresses ({selfInfo.addresses.length})</label>
                                        <div className="flex-1 overflow-y-auto pr-1 custom-scrollbar space-y-1 sm:space-y-1.5">
                                            {selfInfo.addresses.length > 0 ? selfInfo.addresses.map((addr, i) => (
                                                <div key={i} className="flex items-center gap-2 p-1 sm:p-2 bg-muted/20 rounded-md sm:rounded-lg border border-primary/5 group transition-colors hover:bg-muted/30">
                                                    <div className="w-4 h-4 rounded-md bg-green-500/10 flex items-center justify-center text-green-500 shrink-0">
                                                        <Wifi className="w-2.5 h-2.5" />
                                                    </div>
                                                    <span className="font-mono text-[7px] sm:text-[9px] text-muted-foreground truncate flex-1 leading-none">{addr}</span>
                                                </div>
                                            )) : (
                                                <div className="p-2 sm:p-3 bg-muted/10 rounded-xl border border-dashed border-primary/10 text-[8px] sm:text-[9px] text-muted-foreground text-center italic">
                                                    Fetching transport addresses...
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                </>
                            ) : (
                                <div className="h-full flex items-center justify-center animate-pulse text-muted-foreground text-[10px] font-bold">
                                    Initializing P2P Stack...
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    {/* Relay Connection Status */}
                    <Card className="flex-1 border-primary/20 bg-card/10 backdrop-blur-xl shadow-xl rounded-[1.2rem] sm:rounded-[2rem] overflow-hidden min-w-0 sm:min-w-[280px] min-h-0 flex flex-col">
                        <CardHeader className="bg-primary/5 py-1.5 sm:py-3 px-3 sm:px-5 border-b border-primary/10 flex flex-row items-center gap-2 sm:gap-3 shrink-0">
                            <Zap className="w-3.5 h-3.5 sm:w-4 h-4 text-primary" />
                            <CardTitle className="text-[8px] sm:text-[10px] font-bold uppercase tracking-[0.1em] sm:tracking-[0.15em] text-primary/90">Bridge Status</CardTitle>
                        </CardHeader>
                        <CardContent className="p-3 sm:p-4 space-y-2 sm:space-y-3 min-h-0 flex-1 flex flex-col overflow-hidden">
                            <div className="space-y-2 sm:space-y-4">
                                <div className="flex items-center gap-3 py-1.5 sm:py-2 px-1.5 sm:px-2 bg-muted/10 rounded-xl sm:rounded-2xl border border-primary/5">
                                    <div className={cn(
                                        "w-8 h-8 sm:w-12 sm:h-12 rounded-full flex items-center justify-center relative shadow-lg shrink-0",
                                        relayStatus.toLowerCase().includes('connected') ? "bg-green-500/10 text-green-500 shadow-green-500/10" : "bg-orange-500/10 text-orange-500 shadow-orange-500/10"
                                    )}>
                                        <div className={cn(
                                            "absolute inset-0 rounded-full animate-ping opacity-10",
                                            relayStatus.toLowerCase().includes('connected') ? "bg-green-500" : "bg-orange-500"
                                        )} />
                                        <Link className="w-4 h-4 sm:w-6 sm:h-6 relative z-10" />
                                    </div>
                                    <div className="min-w-0">
                                        <div className="text-[8px] font-black uppercase text-muted-foreground tracking-widest mb-0.5 leading-none">State</div>
                                        <div className={cn(
                                            "text-xs sm:text-sm font-black uppercase leading-tight truncate",
                                            relayStatus.toLowerCase().includes('connected') ? "text-emerald-500" : "text-orange-500"
                                        )}>
                                            {relayStatus}
                                        </div>
                                    </div>
                                </div>

                                <div className="space-y-1.5 pt-2 border-t border-primary/5 text-[9px] sm:text-[10px]">
                                    <div className="flex justify-between items-center px-1">
                                        <span className="font-bold uppercase text-muted-foreground/60 tracking-wider">Height</span>
                                        <span className="font-extrabold text-foreground">#{height}</span>
                                    </div>
                                    <div className="flex justify-between items-center px-1">
                                        <span className="font-bold uppercase text-muted-foreground/60 tracking-wider">Latency</span>
                                        <span className="font-extrabold text-foreground">{avgLatency}ms</span>
                                    </div>
                                    <div className="flex justify-between items-center px-1">
                                        <span className="font-bold uppercase text-muted-foreground/60 tracking-wider">Relay</span>
                                        <span className="font-mono font-bold text-primary truncate max-w-[80px] sm:max-w-[120px]">
                                            {connectedRelay ? `${connectedRelay.substring(0, 8)}...` : "none"}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* Detailed Mesh Grid */}
                <Card className="flex-1 min-h-0 flex flex-col border-primary/20 bg-card/5 backdrop-blur-2xl shadow-2xl rounded-2xl sm:rounded-[2.5rem]">
                    <CardHeader className="py-2.5 px-3 sm:py-6 sm:px-8 border-b border-primary/10 shrink-0 flex flex-row items-center justify-between gap-3 bg-muted/10">
                        <div className="flex items-center gap-2 sm:gap-4 min-w-0 flex-1">
                            <div className="p-1.5 sm:p-2.5 bg-primary/10 rounded-lg sm:rounded-xl border border-primary/20 shrink-0 flex">
                                <Terminal className="w-3.5 h-3.5 sm:w-5 sm:h-5 text-primary" />
                            </div>
                            <div className="min-w-0">
                                <CardTitle className="text-[8px] sm:text-[10px] font-bold uppercase text-primary/70 tracking-[0.15em] sm:tracking-[0.25em] mb-0.5 leading-none">
                                    Peer Matrix
                                </CardTitle>
                                <h3 className="text-xs sm:text-xl font-extrabold text-foreground tracking-tight leading-none truncate mt-0.5 sm:mt-1">
                                    DHT Bucket Nodes
                                </h3>
                            </div>
                        </div>
                        <Badge className="bg-primary/20 text-primary border-primary/20 font-black px-2 sm:px-4 py-1 sm:py-1.5 rounded-full text-[7px] sm:text-[10px] tracking-widest uppercase shrink-0">
                            {validatorPeers.length} <span className="hidden xs:inline">Validator {validatorPeers.length === 1 ? 'Node' : 'Nodes'}</span>
                            <span className="xs:hidden">VALS</span>
                        </Badge>
                    </CardHeader>
                    <CardContent className="flex-1 overflow-y-auto p-3 sm:p-8 custom-scrollbar min-h-0">
                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-3 sm:gap-6">
                            {validatorPeers.length === 0 ? (
                                <div className="col-span-full h-64 sm:h-80 flex flex-col items-center justify-center text-muted-foreground gap-6 sm:gap-8 opacity-40">
                                    <div className="relative w-16 h-16 sm:w-24 sm:h-24">
                                        <Activity className="w-full h-full animate-pulse" />
                                    </div>
                                    <div className="text-center space-y-1 sm:space-y-2 px-4">
                                        <h3 className="font-black text-foreground text-base sm:text-xl">Searching for validators...</h3>
                                        <p className="text-[10px] sm:text-xs max-w-sm mx-auto leading-relaxed italic">
                                            Local DHT is performing lookup for verified validator nodes. Connected nodes will appear here with technical metrics.
                                        </p>
                                    </div>
                                </div>
                            ) : (
                                validatorPeers.map((peer) => (
                                    <div key={peer.peer_id} className="group p-3 sm:p-6 rounded-xl sm:rounded-[2rem] border border-primary/5 bg-background/40 flex flex-col gap-3 sm:gap-6 hover:border-primary/40 hover:bg-card/40 transition-all shadow-sm hover:shadow-2xl">
                                        <div className="flex items-start justify-between gap-2">
                                            <div className="flex items-center gap-2 sm:gap-4 min-w-0">
                                                <div
                                                    className="w-8 h-8 sm:w-12 sm:h-12 rounded-lg sm:rounded-2xl flex items-center justify-center text-white font-black text-[10px] sm:text-lg shadow-lg shrink-0 group-hover:scale-110 transition-transform"
                                                    style={{ backgroundColor: getPeerColor(peer.peer_id) }}
                                                >
                                                    {peer.peer_id.substring(peer.peer_id.length - 2).toUpperCase()}
                                                </div>
                                                <div className="min-w-0">
                                                    <div className="text-[7px] sm:text-[9px] font-black uppercase text-muted-foreground tracking-tighter mb-0.5 leading-none">Identity</div>
                                                    <div className="text-[9px] sm:text-xs font-mono font-bold text-foreground/90 truncate max-w-[80px] sm:max-w-[140px]" title={peer.peer_id}>
                                                        {peer.peer_id.substring(0, 10)}...
                                                    </div>
                                                </div>
                                            </div>
                                            {peer.is_verified ? (
                                                <div className="p-1 sm:p-2.5 rounded-lg sm:rounded-2xl bg-emerald-500/10 text-emerald-500 border border-emerald-500/10 shadow-sm shrink-0" title="VDF Proof Verified - Sybil Protected">
                                                    <ShieldCheck className="w-3.5 h-3.5 sm:w-5 h-5" />
                                                </div>
                                            ) : (
                                                <div className="p-1 sm:p-2.5 rounded-lg sm:rounded-2xl bg-orange-500/10 text-orange-500 border border-orange-500/10 shadow-sm shrink-0" title="VDF Pendng - Quarantine Period">
                                                    <ShieldAlert className="w-3.5 h-3.5 sm:w-5 h-5" />
                                                </div>
                                            )}
                                        </div>

                                        <div className="space-y-3 sm:space-y-4">
                                            <div className="space-y-1.5 sm:space-y-2">
                                                <div className="flex justify-between items-center px-0.5">
                                                    <span className="text-[8px] sm:text-[10px] font-black uppercase text-muted-foreground/60 tracking-tighter">Connection Routes</span>
                                                    <span className="text-[8px] sm:text-[9px] font-black bg-primary/10 text-primary px-1.5 py-0.5 rounded-md">{peer.addresses.length} ADDR</span>
                                                </div>
                                                <div className="space-y-1 grayscale group-hover:grayscale-0 transition-all duration-500 max-h-[60px] sm:max-h-[80px] overflow-y-auto scrollbar-none">
                                                    {peer.addresses.map((addr, idx) => (
                                                        <div key={idx} className="font-mono text-[8px] sm:text-[9px] text-muted-foreground/70 bg-muted/20 p-1.5 sm:p-2 rounded-md sm:rounded-lg truncate border border-primary/5">
                                                            {addr}
                                                        </div>
                                                    ))}
                                                    {peer.addresses.length === 0 && (
                                                        <div className="text-[8px] text-muted-foreground italic px-1 pt-1 flex items-center gap-1.5">
                                                            <div className="w-1 h-1 rounded-full bg-orange-500 animate-pulse" />
                                                            Discovery via relay in progress...
                                                        </div>
                                                    )}
                                                </div>
                                            </div>

                                            <div className="pt-3 sm:pt-4 border-t border-primary/5 flex items-center justify-between">
                                                <div className="flex items-center gap-3">
                                                    <div className="flex flex-col">
                                                        <span className="text-[8px] sm:text-[9px] font-black uppercase text-muted-foreground/60">Trust</span>
                                                        <span className={cn(
                                                            "text-[10px] sm:text-xs font-black",
                                                            peer.trust_score > 0.8 ? "text-emerald-500" : "text-orange-500"
                                                        )}>
                                                            {peer.is_verified ? "Sybil Secure" : `${(peer.trust_score * 100).toFixed(0)}% Trust`}
                                                        </span>
                                                    </div>
                                                </div>
                                                <div className="text-right">
                                                    <span className="text-[8px] sm:text-[9px] font-black uppercase text-muted-foreground/60">Latency</span>
                                                    <div className="text-[10px] sm:text-xs font-black text-foreground">
                                                        {peer.latency || 0} <span className="text-[8px] sm:text-[9px] opacity-40 font-black">MS</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                ))
                            )}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </PageTransition>
    );
}
