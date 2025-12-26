import { motion, AnimatePresence } from "framer-motion";
import { Box, X, Clock, Activity, Layers, ShieldCheck, Copy, Check, User, ArrowRight, Hash } from "lucide-react";
import { Block, Transaction } from "../context/AppContext";
import { formatNumber } from "../utils/format";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { useState } from "react";
import { format } from "date-fns";

interface BlockDetailsModalProps {
    block: Block | null;
    onClose: () => void;
    onTxClick?: (tx: Transaction) => void;
}

export default function BlockDetailsModal({ block, onClose, onTxClick }: BlockDetailsModalProps) {
    if (!block) return null;

    const [copied, setCopied] = useState(false);

    const copyHash = () => {
        navigator.clipboard.writeText(block.hash);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 20 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 20 }}
                    className="bg-white dark:bg-black rounded-3xl w-full max-w-4xl overflow-hidden shadow-2xl relative max-h-[90vh] flex flex-col border border-border/50"
                >
                    {/* Header */}
                    <div className="px-6 py-5 border-b border-border/50 flex items-center justify-between shrink-0">
                        <div className="flex items-center gap-4">
                            <div className="w-12 h-12 rounded-2xl bg-primary/5 flex items-center justify-center text-primary border border-primary/10">
                                <Box className="w-6 h-6" />
                            </div>
                            <div>
                                <h2 className="text-xl font-bold flex items-center gap-2">
                                    Block #{formatNumber(block.index, false)}
                                    <Badge variant="outline" className="text-[10px] font-normal h-5">v{block.version || 1}</Badge>
                                    <Badge variant="secondary" className="text-[10px] font-mono h-5 bg-primary/10 text-primary border-primary/20">
                                        Shard #{block.shard_id ?? 0}
                                    </Badge>
                                </h2>
                                <p className="text-xs text-muted-foreground">Detailed historical record</p>
                            </div>
                        </div>
                        <button onClick={onClose} className="p-2 hover:bg-muted/50 rounded-full transition-colors group">
                            <X className="w-5 h-5 text-muted-foreground group-hover:text-foreground" />
                        </button>
                    </div>

                    <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar">

                        {/* Summary Stats */}
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div className="p-4 rounded-2xl bg-muted/30 border border-border/50 flex flex-col gap-1">
                                <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Time</span>
                                <div className="flex items-center gap-2 font-medium text-sm text-foreground">
                                    <Clock className="w-3.5 h-3.5 text-primary" />
                                    {format(new Date(block.timestamp * 1000), "yyyy/MM/dd, h:mm:ss a")}
                                </div>
                            </div>
                            <div className="p-4 rounded-2xl bg-muted/30 border border-border/50 flex flex-col gap-1">
                                <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Transfers</span>
                                <div className="flex items-center gap-2 font-medium text-sm text-foreground">
                                    <Activity className="w-3.5 h-3.5 text-blue-500" />
                                    {block.transactions.length}
                                </div>
                            </div>
                            <div className="p-4 rounded-2xl bg-muted/30 border border-border/50 flex flex-col gap-1">
                                <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Size</span>
                                <div className="flex items-center gap-2 font-medium text-sm text-foreground">
                                    <Layers className="w-3.5 h-3.5 text-orange-500" />
                                    {((block.size || 0) / 1024).toFixed(2)} KB
                                </div>
                            </div>
                            <div className="p-4 rounded-2xl bg-muted/30 border border-border/50 flex flex-col gap-1">
                                <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Reward</span>
                                <div className="flex items-center gap-2 font-bold text-sm text-emerald-500">
                                    <ShieldCheck className="w-3.5 h-3.5" />
                                    {formatNumber(block.total_reward || 0)} AGT
                                </div>
                            </div>
                            <div className="p-4 rounded-2xl col-span-2 md:col-span-4 bg-primary/5 border border-primary/10 flex flex-col gap-2">
                                <span className="text-[10px] font-bold text-primary uppercase tracking-wider">Block Hash</span>
                                <div className="flex items-center justify-between gap-4 bg-background p-3 rounded-xl border border-border/50 shadow-sm">
                                    <code className="text-xs font-mono break-all text-foreground/80">{block.hash}</code>
                                    <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0 hover:bg-primary/10 hover:text-primary" onClick={copyHash}>
                                        {copied ? <Check className="w-4 h-4 text-emerald-500" /> : <Copy className="w-4 h-4" />}
                                    </Button>
                                </div>
                            </div>
                        </div>

                        {/* Technical Details */}
                        <div className="space-y-4">
                            <h3 className="text-xs font-bold flex items-center gap-2 text-muted-foreground uppercase tracking-wider pl-1">
                                <ShieldCheck className="w-4 h-4" /> Validation Data
                            </h3>
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div className="p-5 rounded-2xl bg-muted/20 border border-border/50 space-y-4">
                                    <div>
                                        <span className="text-[10px] font-bold text-muted-foreground block mb-1.5 uppercase tracking-wide">Merkle Root</span>
                                        <code className="text-[10px] font-mono block truncate bg-background p-2 rounded-lg border border-border/50 text-foreground/80">{block.merkle_root || "N/A"}</code>
                                    </div>
                                    <div>
                                        <span className="text-[10px] font-bold text-muted-foreground block mb-1.5 uppercase tracking-wide">VDF Proof</span>
                                        <div className="flex items-center gap-2 bg-background p-2 rounded-lg border border-border/50">
                                            <code className="text-[10px] font-mono truncate text-foreground/80 flex-1">{block.vdf_proof || "N/A"}</code>
                                            {block.vdf_proof && (
                                                <button
                                                    onClick={() => navigator.clipboard.writeText(block.vdf_proof!)}
                                                    className="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                    <div>
                                        <span className="text-[10px] font-bold text-muted-foreground block mb-1.5 uppercase tracking-wide">State Root</span>
                                        <code className="text-[10px] font-mono block truncate bg-background p-2 rounded-lg border border-border/50 text-foreground/80">{block.state_root || "N/A"}</code>
                                    </div>
                                </div>
                                <div className="p-5 rounded-2xl bg-muted/20 border border-border/50 space-y-4">
                                    <div>
                                        <span className="text-[10px] font-bold text-muted-foreground block mb-1.5 uppercase tracking-wide">Validator (Proposer)</span>
                                        <div className="flex items-center gap-2 bg-background p-2 rounded-lg border border-border/50">
                                            <div className="w-5 h-5 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                                                <User className="w-3 h-3 text-primary" />
                                            </div>
                                            <code className="text-[10px] font-mono truncate text-foreground/90">{block.author}</code>
                                        </div>
                                    </div>
                                    <div>
                                        <span className="text-[10px] font-bold text-muted-foreground block mb-1.5 uppercase tracking-wide">Nonce</span>
                                        <code className="text-[10px] font-mono block bg-background p-2 rounded-lg border border-border/50 text-foreground/80">{block.nonce}</code>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Transactions List */}
                        <div className="space-y-4">
                            <h3 className="text-xs font-bold flex items-center gap-2 text-muted-foreground uppercase tracking-wider pl-1">
                                <Activity className="w-4 h-4" /> Transactions ({block.transactions.length})
                            </h3>
                            <div className="rounded-2xl border border-border/50 overflow-hidden bg-muted/10">
                                {block.transactions.length > 0 ? (
                                    <div className="divide-y divide-border/50">
                                        {block.transactions.map((tx: any) => (
                                            <div
                                                key={tx.id}
                                                onClick={() => onTxClick?.(tx)}
                                                className="p-4 hover:bg-muted/50 transition-all cursor-pointer group flex items-center justify-between gap-4"
                                            >
                                                <div className="flex flex-col gap-1.5 min-w-0">
                                                    <div className="flex items-center gap-2">
                                                        <div className="p-1.5 rounded-full bg-background border border-border/50 group-hover:border-primary/30 transition-colors">
                                                            <Hash className="w-3.5 h-3.5 text-muted-foreground group-hover:text-primary transition-colors" />
                                                        </div>
                                                        <span className="text-xs font-mono font-bold text-foreground truncate max-w-[140px] sm:max-w-md">{tx.id}</span>
                                                    </div>
                                                    <div className="text-[10px] text-muted-foreground flex items-center gap-1.5 pl-8">
                                                        <span className="truncate max-w-[100px]">{tx.sender.substring(0, 12)}...</span>
                                                        <ArrowRight className="w-3 h-3 opacity-50" />
                                                        <span className="truncate max-w-[100px]">{tx.receiver.substring(0, 12)}...</span>
                                                    </div>
                                                </div>
                                                <div className="flex items-center gap-4">
                                                    <div className="font-mono text-sm font-bold tabular-nums text-right">
                                                        {formatNumber(tx.amount)} <span className="text-[10px] text-muted-foreground font-normal">AGT</span>
                                                    </div>
                                                    <div className="p-2 rounded-full text-muted-foreground/20 group-hover:text-primary/50 group-hover:bg-primary/5 transition-all">
                                                        <ArrowRight className="w-4 h-4" />
                                                    </div>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div className="p-12 text-center flex flex-col items-center gap-3">
                                        <div className="p-4 rounded-full bg-background border border-border/20">
                                            <Layers className="w-6 h-6 text-muted-foreground/50" />
                                        </div>
                                        <p className="text-sm text-muted-foreground italic">No user transactions recorded in this block.</p>
                                    </div>
                                )}
                            </div>
                        </div>

                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}
