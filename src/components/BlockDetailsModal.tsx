import { motion, AnimatePresence } from "framer-motion";
import { X, Hash, Clock, User, Layers, ArrowRight, ShieldCheck } from "lucide-react";
import { Block } from "../context/AppContext";
import { formatDistanceToNow } from "date-fns";
import { formatNumber } from "../utils/format";

interface BlockDetailsModalProps {
    block: Block | null;
    onClose: () => void;
}

export default function BlockDetailsModal({ block, onClose }: BlockDetailsModalProps) {
    if (!block) return null;

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 20 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 20 }}
                    className="bg-card border border-primary/20 rounded-xl w-full max-w-2xl overflow-hidden shadow-2xl shadow-primary/10"
                >
                    <div className="flex items-center justify-between p-6 border-b border-primary/10 bg-primary/5">
                        <div className="flex items-center gap-3">
                            <Layers className="text-primary w-6 h-6" />
                            <h2 className="text-xl font-bold">Block #{formatNumber(block.index, false)} Details</h2>
                        </div>
                        <button onClick={onClose} className="p-2 hover:bg-primary/10 rounded-full transition-colors">
                            <X className="w-5 h-5 text-muted-foreground" />
                        </button>
                    </div>

                    <div className="p-6 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar">
                        {/* Summary Cards */}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-4 rounded-lg bg-primary/5 border border-primary/10">
                                <span className="text-xs text-muted-foreground uppercase tracking-wider block mb-1">Timestamp</span>
                                <div className="flex items-center gap-2 text-sm">
                                    <Clock className="w-4 h-4 text-primary" />
                                    {formatDistanceToNow(new Date(block.timestamp * 1000))} ago
                                </div>
                            </div>
                            <div className="p-4 rounded-lg bg-primary/5 border border-primary/10">
                                <span className="text-xs text-muted-foreground uppercase tracking-wider block mb-1">Transactions</span>
                                <div className="flex items-center gap-2 text-sm font-mono">
                                    <Hash className="w-4 h-4 text-primary" />
                                    {formatNumber(block.transactions.length, false)} items
                                </div>
                            </div>
                        </div>

                        {/* Hash Info */}
                        <div className="space-y-4">
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wider block mb-1">Block Hash</label>
                                <div className="p-3 bg-muted rounded border font-mono text-xs break-all text-primary">
                                    {block.hash}
                                </div>
                            </div>
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wider block mb-1">Previous Hash</label>
                                <div className="p-3 bg-muted/50 rounded border font-mono text-xs break-all">
                                    {block.previous_hash}
                                </div>
                            </div>
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wider block mb-1">Author (Miner)</label>
                                <div className="p-3 bg-muted/50 rounded border font-mono text-xs flex items-center gap-2">
                                    <User className="w-3 h-3 text-muted-foreground" />
                                    {block.author}
                                </div>
                            </div>
                        </div>

                        {/* Transactions List */}
                        <div>
                            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                                <ShieldCheck className="w-4 h-4 text-green-500" /> Transactions
                            </h3>
                            <div className="space-y-2">
                                {block.transactions.map((tx) => (
                                    <div key={tx.id} className="p-3 rounded-lg border bg-card/50 text-xs font-mono">
                                        <div className="flex justify-between mb-2">
                                            <span className="text-muted-foreground">ID: {tx.id.substring(0, 16)}...</span>
                                            <span className="text-primary font-bold">+{formatNumber(tx.amount)} AGT</span>
                                        </div>
                                        <div className="flex items-center gap-2 text-[10px] opacity-70">
                                            <span className="truncate max-w-[120px]">{tx.sender}</span>
                                            <ArrowRight className="w-2 h-2" />
                                            <span className="truncate max-w-[120px]">{tx.receiver}</span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>

                    <div className="p-4 bg-muted/30 border-t border-primary/10 flex justify-end">
                        <button
                            onClick={onClose}
                            className="bg-primary text-primary-foreground px-6 py-2 rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
                        >
                            Close Details
                        </button>
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}
