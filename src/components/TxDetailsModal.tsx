import { motion, AnimatePresence } from "framer-motion";
import { X, Clock, ArrowUpRight, CheckCircle2, Copy, Check, Info, User } from "lucide-react";
import { Transaction } from "../context/AppContext";
import { formatDistanceToNow } from "date-fns";
import { formatNumber, calculateFee } from "../utils/format";
import { Button } from "./ui/button";
import { useState } from "react";

interface TxDetailsModalProps {
    tx: Transaction | null;
    onClose: () => void;
}

export default function TxDetailsModal({ tx, onClose }: TxDetailsModalProps) {
    if (!tx) return null;

    const isSystemTx = tx.sender.toLowerCase().startsWith('system');
    const fee = isSystemTx ? 0 : calculateFee(tx.amount);
    const total = tx.amount + fee;
    const [copied, setCopied] = useState(false);

    const copyId = () => {
        navigator.clipboard.writeText(tx.id);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    }

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 30 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 30 }}
                    className="bg-white dark:bg-black rounded-3xl w-full max-w-lg overflow-hidden shadow-2xl relative border border-border/50"
                >
                    <div className="absolute top-0 right-0 p-4 z-10">
                        <button onClick={onClose} className="p-2 hover:bg-muted/50 rounded-full transition-colors">
                            <X className="w-5 h-5 text-muted-foreground" />
                        </button>
                    </div>

                    <div className="p-8 pb-10 flex flex-col items-center text-center relative">
                        <div className="w-20 h-20 rounded-full bg-emerald-500/10 flex items-center justify-center text-emerald-500 mb-4 border border-emerald-500/20 z-10">
                            <CheckCircle2 className="w-10 h-10" />
                        </div>

                        <h2 className="text-2xl font-bold text-foreground mb-1 z-10">Transaction Confirmed</h2>
                        <div className="flex items-center gap-2 text-xs text-muted-foreground font-medium mb-6 z-10 bg-muted/30 px-3 py-1 rounded-full border border-border/50">
                            <Clock className="w-3 h-3" />
                            {formatDistanceToNow(new Date(tx.timestamp * 1000))} ago
                        </div>

                        <div className="w-full bg-muted/20 rounded-3xl p-6 border border-border/50 z-10">
                            <div className="text-sm font-semibold text-muted-foreground mb-1">Total Amount</div>
                            <div className="text-4xl font-black tracking-tighter text-foreground mb-2">
                                {formatNumber(tx.amount)} <span className="text-xl text-primary font-bold">AGT</span>
                            </div>
                            <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
                                <span>Fee: {formatNumber(fee)} AGT</span>
                                <span className="w-1 h-1 rounded-full bg-muted-foreground/50" />
                                <span>Total Cost: {formatNumber(total)} AGT</span>
                            </div>
                        </div>
                    </div>

                    <div className="px-8 pb-8 space-y-6">
                        <div className="space-y-4">
                            <div>
                                <div className="flex justify-between items-center mb-2 px-1">
                                    <span className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Transaction ID</span>
                                    <Button variant="ghost" size="icon" className="h-6 w-6 -mr-2" onClick={copyId}>
                                        {copied ? <Check className="w-3 h-3 text-primary" /> : <Copy className="w-3 h-3 text-muted-foreground" />}
                                    </Button>
                                </div>
                                <div className="bg-muted/10 p-3 rounded-xl border border-border/50 font-mono text-xs break-all text-muted-foreground hover:text-foreground transition-colors text-center select-all">
                                    {tx.id}
                                </div>
                            </div>

                            <div className="flex flex-col md:flex-row gap-4 items-center md:items-end">
                                <div className="flex-1 w-full min-w-0 flex flex-col gap-2">
                                    <span className="text-xs font-bold text-muted-foreground uppercase tracking-wider pl-1">From</span>
                                    <div className="p-3 bg-muted/5 rounded-xl border border-border/50 flex items-center gap-2 group hover:bg-muted/10 transition-colors">
                                        <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                                            <User className="w-4 h-4 text-primary" />
                                        </div>
                                        <div className="flex-1 min-w-0 text-left">
                                            <div className="font-mono text-[10px] text-muted-foreground truncate group-hover:text-foreground transition-colors uppercase">{tx.sender}</div>
                                        </div>
                                    </div>
                                </div>

                                <div className="flex items-center justify-center h-10 w-10 shrink-0">
                                    <div className="w-8 h-8 rounded-full bg-muted text-muted-foreground flex items-center justify-center border border-border rotate-90 md:rotate-0">
                                        <ArrowUpRight className="w-4 h-4" />
                                    </div>
                                </div>

                                <div className="flex-1 w-full min-w-0 flex flex-col gap-2">
                                    <span className="text-xs font-bold text-muted-foreground uppercase tracking-wider pl-1">To</span>
                                    <div className="p-3 bg-muted/5 rounded-xl border border-border/50 flex items-center gap-2 group hover:bg-muted/10 transition-colors">
                                        <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                                            <User className="w-4 h-4 text-primary" />
                                        </div>
                                        <div className="flex-1 min-w-0 text-left">
                                            <div className="font-mono text-[10px] text-muted-foreground truncate group-hover:text-foreground transition-colors uppercase">{tx.receiver}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="p-4 rounded-xl bg-blue-500/5 border border-blue-500/10 flex gap-3 items-start">
                            <Info className="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
                            <p className="text-xs text-blue-600/80 dark:text-blue-400">
                                This transaction has been verified and permanently recorded on the block ledger. It cannot be reversed.
                            </p>
                        </div>
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}
