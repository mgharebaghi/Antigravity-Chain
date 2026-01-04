import { Activity, Layers, Pickaxe, RefreshCw, Globe, Cpu, Network, Zap } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatNumber } from '../utils/format';
import { StatCard } from '../components/dashboard/StatCard';
import { VdfVisualizer } from '../components/dashboard/VdfVisualizer';
import { ConsensusCard } from '../components/dashboard/ConsensusCard';

export default function NetworkDashboard() {
    const {
        wallet,
        nodeStatus,
        relayStatus, // Use relayStatus specifically for "Active" check logic
        connectedRelay,
        peers,
        startNode,
        stopNode,
        latestBlock,
        minedBlocks,
        vdfStatus,
        selfNodeInfo,
        consensusStatus
    } = useApp();

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

    // Strict Status Logic per User Requirement
    // "ta be relay connect nashode nabayad active bezane"
    // Use `relayStatus` ("Connected" or similar) AND `nodeStatus` ("Active")
    const isRelayConnected = relayStatus.toLowerCase().includes("connected") || !!connectedRelay;
    const isError = nodeStatus.toLowerCase().includes("error") || nodeStatus.includes("Unreachable") || relayStatus.includes("Failed");

    // Only show "Online" if Relay is strictly connected AND node is marked as Active
    const isOnline = nodeStatus.startsWith("Active") && isRelayConnected;
    // Granular connecting states
    const isConnecting = !isOnline && !isError && nodeStatus !== "Stopped";

    // "dar heyne sakhte block ham nabayad proof bezane"
    // Validated in ConsensusCard. If Leader -> Block Building. Else -> Proofing.

    // We only pass `isActive` to VDF visualizer if we are NOT building a block (Leader)
    // because if Leader, we are busy mining, not VDFing (usually).
    // Actually, VDF runs continuously in this Chain, but visual distinction is good.
    const isLeader = consensusStatus?.state === "Leader";
    const isVdfActive = isOnline && !isLeader;

    return (
        <div className="flex flex-col gap-4 lg:h-full w-full lg:overflow-hidden overflow-visible p-4 sm:p-5 lg:p-5 max-w-[1600px] mx-auto">
            {/* Header / Hero Section */}
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-white/5 backdrop-blur-3xl p-4 rounded-[2rem] border border-white/10 shadow-2xl relative overflow-hidden group">
                <div className="absolute inset-0 bg-gradient-to-r from-primary/10 via-transparent to-transparent opacity-50" />

                <div className="relative z-10 flex items-center gap-4">
                    <div className="p-2.5 bg-primary/20 rounded-2xl shadow-inner border border-white/5">
                        <Activity className="w-5 h-5 text-primary animate-pulse" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-black tracking-tight text-white uppercase italic">
                            Network <span className="text-primary tracking-normal not-italic">Overview</span>
                        </h1>
                        <p className="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-[0.2em] mt-0.5">Real-time Node Telemetry</p>
                    </div>
                </div>

                <div className="relative z-10 flex items-center gap-3">
                    {!wallet ? (
                        <Button asChild size="lg" className="rounded-2xl px-8 font-black uppercase tracking-widest bg-primary hover:bg-primary/90 shadow-xl shadow-primary/20 transition-all active:scale-95">
                            <Link to="/wallet">Initialize Wallet</Link>
                        </Button>
                    ) : (
                        isError ? (
                            <div className="flex items-center gap-3">
                                {nodeStatus.includes("Relay Not Found") && (
                                    <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-red-500/10 border border-red-500/50 text-red-500 font-black uppercase tracking-widest text-[10px] shadow-[0_0_20px_rgba(239,68,68,0.3)] animate-pulse">
                                        <Globe className="w-3 h-3" /> No Relay in Network
                                    </div>
                                )}
                                <Button onClick={handleRetry} variant="destructive" size="lg" className="rounded-2xl font-black uppercase tracking-widest animate-pulse shadow-lg shadow-red-500/20">
                                    <RefreshCw className="w-4 h-4 mr-2" /> {nodeStatus.includes("Relay") ? "Retry Connection" : "Recover System"}
                                </Button>
                            </div>
                        ) : nodeStatus === "Stopped" ? (
                            <Button onClick={handleStartNodeClick} size="lg" className="rounded-xl px-10 font-black uppercase tracking-widest bg-emerald-600 hover:bg-emerald-500 shadow-xl shadow-emerald-500/20 transition-all active:scale-95">
                                <Zap className="w-4 h-4 mr-2 fill-current" /> Ignite Node
                            </Button>
                        ) : isConnecting ? (
                            <div className="flex flex-col items-end gap-1">
                                <div className="flex items-center gap-3 px-6 py-2 rounded-2xl bg-orange-500/10 text-orange-400 border border-orange-500/20 text-xs font-black uppercase tracking-widest backdrop-blur-md">
                                    <RefreshCw className="w-4 h-4 animate-spin text-orange-500" /> {nodeStatus}
                                </div>
                                <span className="text-[10px] font-bold text-orange-500/60 uppercase tracking-tighter mr-2">
                                    {nodeStatus.includes("Relay") ? "Establishing Circuit..." : "Discovering Topology..."}
                                </span>
                            </div>
                        ) : (
                            <div className="group relative">
                                <div className="absolute inset-0 bg-emerald-500 blur-xl opacity-20 group-hover:opacity-40 transition-opacity" />
                                <div className="relative flex items-center gap-3 px-6 py-2 rounded-2xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 text-xs font-black uppercase tracking-tighter backdrop-blur-md">
                                    <span className="relative flex h-2.5 w-2.5">
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                        <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                                    </span>
                                    {nodeStatus}
                                </div>
                            </div>
                        )
                    )}
                </div>
            </div>

            {/* Main Content Area */}
            <div className="lg:flex-1 lg:min-h-0 grid grid-cols-1 lg:grid-cols-12 gap-4 pb-4 lg:pb-0">
                {/* Left Column: Key Stats and Metrics */}
                <div className="lg:col-span-8 flex flex-col gap-4">
                    {/* Primary Grid */}
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        <StatCard
                            title="Network Health"
                            value={isOnline ? "OPERATIONAL" : nodeStatus.toUpperCase()}
                            icon={Globe}
                            statusColor={isError ? "text-red-500" : isOnline ? "text-emerald-500" : "text-orange-500"}
                            footerLabel="Latency Tunnel"
                            footerValue={isRelayConnected ? "Optimized" : "Degraded"}
                        />
                        <StatCard
                            title="Chain Height"
                            value={formatNumber(latestBlock?.index ?? 0, false)}
                            icon={Layers}
                            footerLabel="Current Hash"
                            footerValue={latestBlock?.hash ? `${latestBlock.hash.substring(0, 8)}` : "GENESIS"}
                        />
                        <StatCard
                            title="Peer Discovery"
                            value={peers}
                            icon={Network}
                            statusColor="text-blue-400"
                            footerLabel="Network Mesh"
                            footerValue="Kademlia DHT"
                        />
                    </div>

                    {/* VDF Visualizer Section */}
                    <div className="flex-1 lg:min-h-0 bg-black/10 rounded-[2.5rem] border border-white/5 overflow-hidden shadow-inner">
                        <VdfVisualizer vdfStatus={vdfStatus} isActive={isVdfActive} />
                    </div>
                </div>

                {/* Right Column: Consensus and Side Metrics */}
                <div className="lg:col-span-4 flex flex-col gap-4">
                    {/* Consensus Status - The Heart of the Dashboard */}
                    <div className="flex-1 lg:min-h-0">
                        <ConsensusCard consensusStatus={consensusStatus} nodeStatus={nodeStatus} />
                    </div>

                    {/* Secondary Metrics Group */}
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <StatCard
                            title="My Local Reach"
                            value={minedBlocks}
                            subValue="Blocks"
                            icon={Pickaxe}
                            footerLabel="Contribution"
                            footerValue={minedBlocks > 0 ? "Producer" : "Relay"}
                        />
                        <StatCard
                            title="Shard Power"
                            value={formatNumber(selfNodeInfo?.shard_tps_limit ?? 300)}
                            subValue="TPS"
                            icon={Cpu}
                            statusColor="text-orange-400"
                            footerLabel="Slot Velocity"
                            footerValue="2s Epoch"
                        />
                    </div>
                </div>
            </div>
        </div>
    );
}
