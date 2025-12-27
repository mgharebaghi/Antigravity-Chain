import { Badge } from "../components/ui/badge";
import { ShieldCheck, ShieldAlert, Activity, Globe, Server, Network as NetworkIcon, Link, Wifi, Copy, Check, Cpu, Cloud } from "lucide-react";
import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "../lib/utils";
import { useApp } from "../context/AppContext";
import { useToast } from "../context/ToastContext";
import { listen } from "@tauri-apps/api/event";
import NetworkMap from "../components/NetworkMap";

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

const getPeerColor = (peerId: string) => {
    const hash = peerId.split('').reduce((acc, char) => char.charCodeAt(0) + ((acc << 5) - acc), 0);
    const h = Math.abs(hash % 360);
    return `hsl(${h}, 70%, 50%)`;
};

export default function Network() {
    const [peers, setPeers] = useState<PeerInfo[]>([]);
    const [dhtPeers, setDhtPeers] = useState<string[]>([]);
    const [selfInfo, setSelfInfo] = useState<SelfNodeInfo | null>(null);
    const [copiedId, setCopiedId] = useState<string | null>(null);
    const [syncInfo, setSyncInfo] = useState<{ state: string, current: number, target: number, peer: string } | null>(null);
    const { height, relayStatus, connectedRelay } = useApp();
    const { success } = useToast();

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

        // Listen for DHT updates
        const unlistenPromise = listen<string[]>("dht-peers-update", (event) => {
            setDhtPeers(event.payload);
        });

        const unlistenSyncPromise = listen<string>("sync-status", (event) => {
            try {
                const data = JSON.parse(event.payload);
                setSyncInfo(data);
            } catch (e) { console.error("Sync status parse error", e); }
        });

        return () => {
            clearInterval(interval);
            unlistenPromise.then(unlisten => unlisten());
            unlistenSyncPromise.then(unlisten => unlisten());
        };
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
        <div className="flex flex-col gap-4 h-full pb-6 overflow-x-hidden">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Network Status</h1>
                    <p className="text-muted-foreground mt-1 text-sm">Real-time P2P connectivity and node metrics.</p>
                </div>
                <div className="flex items-center gap-3 bg-secondary/30 px-4 py-2 rounded-full border border-border/50">
                    <div className={cn(
                        "w-2 h-2 rounded-full",
                        relayStatus.toLowerCase() === 'connected' ? "bg-emerald-500 animate-pulse" :
                            relayStatus.toLowerCase().includes('disconnected') ? "bg-red-500" : "bg-orange-500"
                    )} />
                    <span className={cn(
                        "text-sm font-medium",
                        relayStatus.toLowerCase().includes('disconnected') && "text-red-500"
                    )}>
                        {relayStatus || "Measuring..."}
                    </span>
                </div>
            </div>

            {/* [NEW] Network Map Visualization */}
            <div className="w-full">
                <NetworkMap />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">

                {/* Local Identity Card */}
                <div className="glass-card rounded-3xl p-6 flex flex-col justify-between border-primary/20 bg-primary/5 relative overflow-hidden group">
                    <div className="absolute top-0 right-0 w-32 h-32 bg-primary/20 rounded-full blur-3xl -mr-16 -mt-16 pointer-events-none group-hover:bg-primary/30 transition-colors" />
                    <div>
                        <div className="flex items-center gap-3 mb-6">
                            <div className="p-2.5 bg-background/50 rounded-xl backdrop-blur-sm border border-white/10 shadow-sm">
                                <Server className="w-5 h-5 text-primary" />
                            </div>
                            <h3 className="font-bold text-lg">Local Identity</h3>
                        </div>

                        {selfInfo ? (
                            <div className="space-y-6">
                                <div className="space-y-2">
                                    <label className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground pl-1">Peer ID</label>
                                    <div className="flex items-center gap-2 p-3 bg-background/40 hover:bg-background/60 backdrop-blur-sm rounded-xl border border-white/10 group/copy transition-colors cursor-pointer" onClick={() => copyToClipboard(selfInfo.peer_id, 'peer_id')}>
                                        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-500 shrink-0" />
                                        <code className="text-xs font-mono font-medium truncate flex-1 opacity-80 group-hover/copy:opacity-100 transition-opacity">{selfInfo.peer_id}</code>
                                        <div className="p-1.5 rounded-md hover:bg-white/10">
                                            {copiedId === 'peer_id' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-muted-foreground" />}
                                        </div>
                                    </div>
                                </div>
                                <div className="space-y-2">
                                    <label className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground pl-1">Entrypoints</label>
                                    <div className="bg-background/40 rounded-xl p-3 max-h-[120px] overflow-y-auto space-y-2 custom-scrollbar border border-white/10">
                                        {selfInfo.addresses.length > 0 ? selfInfo.addresses.map((addr, i) => (
                                            <div key={i} className="flex items-center gap-2 text-[10px] font-mono text-muted-foreground bg-white/5 p-1.5 rounded-lg">
                                                <Wifi className="w-3 h-3 opacity-50 shrink-0" />
                                                <span className="truncate">{addr}</span>
                                            </div>
                                        )) : (
                                            <div className="p-2 text-center text-xs text-muted-foreground italic">Resolving addresses...</div>
                                        )}
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="py-12 flex flex-col items-center justify-center gap-3 text-muted-foreground">
                                <Cpu className="w-10 h-10 opacity-20 animate-pulse" />
                                <span className="text-xs font-medium uppercase tracking-wider opacity-60">Initializing P2P Kernel...</span>
                            </div>
                        )}
                    </div>
                </div>

                {/* Gateway / Metrics Card */}
                <div className="lg:col-span-2 glass-card rounded-3xl p-6 flex flex-col justify-between relative overflow-hidden">
                    <div className="absolute inset-0 bg-gradient-to-br from-secondary/30 to-transparent pointer-events-none" />

                    <div className="relative z-10 flex flex-col h-full gap-8">
                        <div className="flex items-start justify-between">
                            <div className="flex items-center gap-3">
                                <div className="p-2.5 bg-emerald-500/10 rounded-xl border border-emerald-500/20 text-emerald-500">
                                    <Globe className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="font-bold text-lg">Network Gateway</h3>
                                    <p className="text-xs text-muted-foreground">Global mesh connectivity status</p>
                                </div>
                            </div>
                            <div className="text-right">
                                {syncInfo && syncInfo.state === 'syncing' ? (
                                    <div className="flex flex-col items-end gap-1">
                                        <p className="text-[10px] font-bold uppercase tracking-widest text-emerald-500 animate-pulse">Synchronizing Data</p>
                                        <p className="text-sm font-mono font-bold text-emerald-500/80">{syncInfo.current} / {syncInfo.target} Blocks</p>
                                        <p className="text-[8px] text-muted-foreground opacity-60">from {syncInfo.peer.substring(0, 8)}...</p>
                                    </div>
                                ) : (
                                    <>
                                        <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1">Ledger Height</p>
                                        <p className="text-2xl font-mono font-black text-foreground">#{height.toLocaleString()}</p>
                                    </>
                                )}
                            </div>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-auto">
                            <div className="p-5 rounded-2xl bg-secondary/30 border border-border/50 flex flex-col gap-2 relative overflow-hidden group hover:border-primary/20 transition-colors">
                                <div className="absolute right-0 top-0 p-4 opacity-10 group-hover:scale-110 transition-transform">
                                    <Link className="w-12 h-12" />
                                </div>
                                <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground z-10">Relay Connection</span>
                                <div className="flex items-center gap-2 z-10">
                                    <span className={cn(
                                        "text-xl font-bold",
                                        relayStatus.toLowerCase() === 'connected' ? "text-emerald-500" :
                                            relayStatus.toLowerCase().includes('disconnected') ? "text-red-500" : "text-orange-500"
                                    )}>
                                        {relayStatus.toLowerCase() === 'connected' ? "Active" :
                                            relayStatus.toLowerCase().includes('disconnected') ? "Offline" : "Connecting"}
                                    </span>
                                    {relayStatus.toLowerCase() === 'connected' ? (
                                        <ShieldCheck className="w-4 h-4 text-emerald-500" />
                                    ) : relayStatus.toLowerCase().includes('disconnected') ? (
                                        <ShieldAlert className="w-4 h-4 text-red-500" />
                                    ) : null}
                                </div>
                            </div>

                            <div className="p-5 rounded-2xl bg-secondary/30 border border-border/50 flex flex-col gap-2 relative overflow-hidden group hover:border-primary/20 transition-colors">
                                <div className="absolute right-0 top-0 p-4 opacity-10 group-hover:scale-110 transition-transform">
                                    <Activity className="w-12 h-12" />
                                </div>
                                <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground z-10">Network Latency</span>
                                <div className="flex items-center gap-2 z-10">
                                    <span className={cn("text-xl font-bold", avgLatency < 100 ? "text-emerald-500" : "text-amber-500")}>
                                        {avgLatency} <span className="text-sm">ms</span>
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Combined Peers & DHT View */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1 min-h-[400px]">

                {/* Active Connected Peers */}
                <div className="glass-card rounded-3xl border-none flex flex-col overflow-hidden shadow-none bg-background/30">
                    <div className="p-6 border-b border-border/50 flex items-center justify-between bg-secondary/20">
                        <h3 className="font-bold text-lg flex items-center gap-2">
                            <NetworkIcon className="w-5 h-5 text-primary" /> Connected Peers
                            <Badge variant="secondary" className="ml-2 font-mono text-xs">{validatorPeers.length}</Badge>
                        </h3>
                    </div>

                    <div className="flex-1 overflow-auto">
                        {validatorPeers.length === 0 ? (
                            <div className="h-full flex flex-col items-center justify-center p-12 text-muted-foreground gap-4">
                                <div className="w-16 h-16 bg-secondary/50 rounded-full flex items-center justify-center opacity-50">
                                    <Activity className="w-8 h-8 animate-pulse" />
                                </div>
                                <p className="text-sm opacity-70">No direct connections established.</p>
                            </div>
                        ) : (
                            <div className="divide-y divide-border/30">
                                {validatorPeers.map((peer) => (
                                    <div key={peer.peer_id} className="p-4 hover:bg-primary/5 transition-colors flex items-center gap-4">
                                        <div className="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-black text-white shadow-lg ring-2 ring-white/10 shrink-0"
                                            style={{ backgroundColor: getPeerColor(peer.peer_id) }}>
                                            {peer.peer_id.substring(peer.peer_id.length - 2).toUpperCase()}
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <p className="font-mono font-bold text-sm truncate">{peer.peer_id}</p>
                                            <p className="text-[10px] text-muted-foreground truncate">{peer.addresses[0] || 'Unknown address'}</p>
                                        </div>
                                        <Badge variant="outline" className="text-emerald-500 border-emerald-500/20 bg-emerald-500/5">Active</Badge>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Discovered / DHT Peers */}
                <div className="glass-card rounded-3xl border-none flex flex-col overflow-hidden shadow-none bg-background/30">
                    <div className="p-6 border-b border-border/50 flex items-center justify-between bg-secondary/20">
                        <h3 className="font-bold text-lg flex items-center gap-2">
                            <Cloud className="w-5 h-5 text-blue-400" /> Discovered (DHT)
                            <Badge variant="secondary" className="ml-2 font-mono text-xs">{dhtPeers.length}</Badge>
                        </h3>
                    </div>

                    <div className="flex-1 overflow-auto">
                        {dhtPeers.length === 0 ? (
                            <div className="h-full flex flex-col items-center justify-center p-12 text-muted-foreground gap-4">
                                <div className="w-16 h-16 bg-secondary/50 rounded-full flex items-center justify-center opacity-50">
                                    <Wifi className="w-8 h-8 opacity-50" />
                                </div>
                                <p className="text-sm opacity-70">Scanning network topology...</p>
                            </div>
                        ) : (
                            <div className="divide-y divide-border/30">
                                {dhtPeers.map((id) => (
                                    <div key={id} className="p-4 hover:bg-primary/5 transition-colors flex items-center gap-4 opacity-75 hover:opacity-100">
                                        <div className="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-black text-white shadow-lg ring-2 ring-white/10 shrink-0 bg-gray-500">
                                            ?
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <p className="font-mono font-bold text-sm truncate">{id}</p>
                                            <p className="text-[10px] text-muted-foreground">Discovered via Relay</p>
                                        </div>
                                        <Badge variant="secondary" className="opacity-50">Pending</Badge>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

            </div>
        </div>
    );
}
