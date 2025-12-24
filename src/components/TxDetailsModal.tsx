import { motion, AnimatePresence } from "framer-motion";
import { X, Hash, Clock, User } from "lucide-react";
import { Transaction } from "../context/AppContext";
import { formatDistanceToNow } from "date-fns";
import { formatNumber, calculateFee } from "../utils/format";

interface TxDetailsModalProps {
    tx: Transaction | null;
    onClose: () => void;
}

export default function TxDetailsModal({ tx, onClose }: TxDetailsModalProps) {
    if (!tx) return null;

    const fee = calculateFee(tx.amount);
    const total = tx.amount + fee;

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 20 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 20 }}
                    className="bg-card border border-primary/20 rounded-2xl w-full max-w-xl overflow-hidden shadow-2xl shadow-primary/10"
                >
                    <div className="flex items-center justify-between p-6 border-b border-primary/10 bg-primary/5">
                        <div className="flex items-center gap-3">
                            <Hash className="text-primary w-6 h-6" />
                            <h2 className="text-xl font-bold">Transaction Details</h2>
                        </div>
                        <button onClick={onClose} className="p-2 hover:bg-primary/10 rounded-full transition-colors">
                            <X className="w-5 h-5 text-muted-foreground" />
                        </button>
                    </div>

                    <div className="p-6 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar">
                        {/* Status & ID */}
                        <div className="space-y-4">
                            <div className="flex items-center justify-between">
                                <span className="text-xs text-muted-foreground uppercase tracking-wider">Transaction ID</span>
                                <div className="px-2 py-0.5 rounded-full bg-orange-500/10 text-orange-500 text-[10px] font-bold border border-orange-500/20">
                                    PENDING
                                </div>
                            </div>
                            <div className="p-3 bg-muted rounded-xl border border-primary/5 font-mono text-[11px] break-all text-primary/80">
                                {tx.id}
                            </div>
                        </div>

                        {/* Amount & Fee Summary */}
                        <div className="grid grid-cols-1 gap-4">
                            <div className="p-5 rounded-2xl bg-primary/5 border border-primary/10 space-y-3">
                                <div className="flex justify-between items-center text-sm">
                                    <span className="text-muted-foreground">Amount</span>
                                    <span className="font-bold text-foreground">{formatNumber(tx.amount)} AGT</span>
                                </div>
                                <div className="flex justify-between items-center text-sm">
                                    <span className="text-muted-foreground">Network Fee (0.01%)</span>
                                    <span className="font-bold text-foreground">{formatNumber(fee)} AGT</span>
                                </div>
                                <div className="h-px bg-primary/10" />
                                <div className="flex justify-between items-center">
                                    <span className="text-xs font-black text-primary uppercase tracking-widest">Total to Deduct</span>
                                    <span className="text-2xl font-black text-foreground">{formatNumber(total)} AGT</span>
                                </div>
                            </div>
                        </div>

                        {/* Sender & Receiver */}
                        <div className="space-y-4">
                            <div className="p-4 rounded-xl bg-muted/30 border border-primary/5 space-y-3">
                                <div>
                                    <label className="text-[10px] text-muted-foreground uppercase tracking-widest block mb-1">Sender Address</label>
                                    <div className="flex items-center gap-2 font-mono text-[11px] text-foreground/80">
                                        <User className="w-3 h-3 text-primary/60" />
                                        <span className="truncate">{tx.sender}</span>
                                    </div>
                                </div>
                                <div className="flex justify-center h-4 relative">
                                    <div className="absolute top-0 bottom-0 left-1/2 -ml-px w-0.5 bg-gradient-to-b from-primary/20 via-primary/40 to-primary/20" />
                                </div>
                                <div>
                                    <label className="text-[10px] text-muted-foreground uppercase tracking-widest block mb-1">Receiver Address</label>
                                    <div className="flex items-center gap-2 font-mono text-[11px] text-foreground/80">
                                        <User className="w-3 h-3 text-orange-400/60" />
                                        <span className="truncate">{tx.receiver}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Timestamp */}
                        <div className="flex items-center gap-2 text-xs text-muted-foreground justify-center">
                            <Clock className="w-3.5 h-3.5" />
                            {formatDistanceToNow(new Date(tx.timestamp * 1000))} ago ({new Date(tx.timestamp * 1000).toLocaleString()})
                        </div>
                    </div>

                    <div className="p-4 bg-muted/30 border-t border-primary/10 flex justify-end">
                        <button
                            onClick={onClose}
                            className="bg-primary text-primary-foreground px-8 py-2.5 rounded-xl text-sm font-bold shadow-lg shadow-primary/20 hover:opacity-90 transition-opacity"
                        >
                            Close Details
                        </button>
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}
