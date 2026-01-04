import { motion } from 'framer-motion';
import { Zap, Clock, Shield } from 'lucide-react';
import { cn } from '../../lib/utils';
import { NodeConsensusStatus } from '../../context/AppContext';

interface ConsensusCardProps {
    consensusStatus: NodeConsensusStatus | null;
    nodeStatus: string;
}

export function ConsensusCard({ consensusStatus, nodeStatus }: ConsensusCardProps) {

    // Strict Logic for Visibility
    const isOffline = !consensusStatus || nodeStatus === "Stopped" || nodeStatus === "Relay Unreachable";
    const state = consensusStatus?.state || "Unknown";

    const isLeader = state === "Leader";
    const isQueue = state === "Queue";
    const isPatience = state === "Patience";
    const isConnecting = state === "Connecting" || !consensusStatus;

    const cardClass = cn(
        "relative overflow-hidden glass-card rounded-3xl p-5 flex flex-col justify-between h-full shadow-2xl transition-all duration-700 border border-white/10",
        isLeader ? "bg-emerald-500/10 shadow-emerald-500/5" :
            isQueue ? "bg-orange-500/10 shadow-orange-500/5" :
                isPatience ? "bg-blue-500/10 shadow-blue-500/5" :
                    "bg-white/5"
    );

    return (
        <div className={cardClass}>
            {/* Ambient Background Glow */}
            {!isOffline && (
                <div className={cn(
                    "absolute -top-20 -right-20 w-64 h-64 blur-[100px] opacity-20 pointer-events-none transition-colors duration-1000",
                    isLeader ? "bg-emerald-500" : isQueue ? "bg-orange-500" : "bg-blue-500"
                )} />
            )}

            <div className="flex justify-between items-center relative z-10">
                <span className="text-[9px] font-black text-muted-foreground/30 uppercase tracking-[0.2em]">Consensus Engine</span>
                {isLeader ? (
                    <div className="flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-emerald-500/20 border border-emerald-500/30 text-emerald-400 text-[9px] font-black uppercase tracking-widest">
                        <Zap className="w-2.5 h-2.5 fill-emerald-400" /> Mining
                    </div>
                ) : isQueue ? (
                    <div className="flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-orange-500/20 border border-orange-500/30 text-orange-400 text-[9px] font-black uppercase tracking-widest">
                        <Clock className="w-2.5 h-2.5" /> Queued
                    </div>
                ) : isPatience ? (
                    <div className="flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-blue-500/20 border border-blue-500/30 text-blue-400 text-[9px] font-black uppercase tracking-widest">
                        <Shield className="w-2.5 h-2.5" /> Patience
                    </div>
                ) : (
                    <div className="flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-white/5 border border-white/5 text-muted-foreground/30 text-[9px] font-black uppercase tracking-widest">
                        Stalled
                    </div>
                )}
            </div>

            <div className="flex-1 flex flex-col justify-center items-center py-6 relative z-10">
                {isOffline || isConnecting ? (
                    <div className="flex flex-col items-center gap-4">
                        {isConnecting && !isOffline ? (
                            <>
                                <div className="relative">
                                    <div className="w-12 h-12 rounded-full border-[3px] border-white/5 border-t-primary animate-spin" />
                                    <div className="absolute inset-0 bg-primary/20 blur-xl rounded-full" />
                                </div>
                                <span className="text-[10px] font-black text-muted-foreground/50 tracking-[0.3em] uppercase">Syncing Protocol</span>
                            </>
                        ) : (
                            <div className="text-center space-y-1">
                                <span className="block text-[10px] font-black text-muted-foreground/20 tracking-[0.3em] uppercase">Standby Mode</span>
                                <span className="block text-base font-bold text-muted-foreground/15">Awaiting Signal</span>
                            </div>
                        )}
                    </div>
                ) : isLeader ? (
                    <div className="flex flex-col items-center gap-3 text-center">
                        <motion.div
                            animate={{ scale: [1, 1.03, 1] }}
                            transition={{ duration: 3, repeat: Infinity }}
                            className="relative"
                        >
                            <div className="absolute inset-0 bg-emerald-500 blur-2xl opacity-20" />
                            <Zap className="w-14 h-14 text-emerald-400 drop-shadow-[0_0_15px_rgba(52,211,153,0.4)] fill-emerald-400/10" />
                        </motion.div>
                        <div className="space-y-0.5">
                            <span className="block text-3xl font-black text-emerald-400 tracking-tighter uppercase">Leader</span>
                            <span className="text-[9px] font-black text-emerald-500/50 uppercase tracking-[0.2em]">Block Production Active</span>
                        </div>
                    </div>
                ) : isQueue ? (
                    <div className="flex flex-col items-center">
                        <span className="text-[9px] font-black text-orange-500/40 uppercase tracking-[0.3em] mb-2">Slot Ranking</span>
                        <span className="text-7xl font-black text-transparent bg-clip-text bg-gradient-to-br from-orange-400 to-orange-600 leading-none">
                            #{consensusStatus.queue_position}
                        </span>
                        <div className="mt-5 px-4 py-2 bg-white/5 border border-white/5 rounded-xl backdrop-blur-xl">
                            <div className="flex items-center gap-2 text-xs font-bold text-orange-400">
                                <Clock className="w-3.5 h-3.5" />
                                <span className="tracking-tight">Wait: <span className="text-foreground text-sm ml-1 font-black">{consensusStatus.remaining_seconds}s</span></span>
                            </div>
                        </div>
                    </div>
                ) : (
                    // Patience State
                    <div className="flex flex-col items-center text-center">
                        <div className="relative mb-5">
                            <motion.div
                                animate={{ rotate: 360 }}
                                transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
                                className="absolute inset-[-10px] w-28 h-28 rounded-full border border-dashed border-blue-500/10"
                            />
                            <div className="flex flex-col items-center justify-center w-24 h-24 rounded-full bg-blue-500/5 border border-white/10 shadow-inner">
                                <Shield className="w-8 h-8 text-blue-500 mb-0.5" />
                            </div>
                        </div>

                        <div className="space-y-0.5">
                            <span className="text-xl font-black text-foreground uppercase tracking-tight">Proof Period</span>
                            <div className="text-[9px] font-black text-blue-500/50 uppercase tracking-[0.2em]">
                                Verification Remaining: {consensusStatus.remaining_seconds}s
                            </div>
                        </div>
                    </div>
                )}
            </div>

            <div className="space-y-3 shrink-0 relative z-10">
                {isPatience && (
                    <div className="space-y-1.5">
                        <div className="flex justify-between text-[9px] font-black uppercase tracking-widest">
                            <span className="text-muted-foreground/30">Maturity Progress</span>
                            <span className="text-blue-500">{((consensusStatus?.patience_progress || 0) * 100).toFixed(1)}%</span>
                        </div>
                        <div className="w-full h-1 bg-white/5 rounded-full overflow-hidden border border-white/5">
                            <motion.div
                                initial={{ width: 0 }}
                                animate={{ width: `${(consensusStatus?.patience_progress || 0) * 100}%` }}
                                className="h-full bg-gradient-to-r from-blue-600 to-blue-400 rounded-full"
                            />
                        </div>
                    </div>
                )}

                <div className="flex justify-between items-center pt-3 border-t border-white/5">
                    <span className="text-[9px] font-black text-muted-foreground/20 uppercase tracking-widest">Active Shard</span>
                    <span className="text-[10px] font-black px-2 py-0.5 rounded-md bg-white/5 border border-white/5">#{consensusStatus?.shard_id || 0}</span>
                </div>
            </div>
        </div>
    );
}
