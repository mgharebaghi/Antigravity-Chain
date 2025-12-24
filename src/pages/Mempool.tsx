import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
// import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Badge } from "../components/ui/badge";
import {
    Clock,
    Zap,
    ArrowRight,
    User,
    ChevronRight,
    Shield,
    X,
    Terminal,
    Activity,
    Search,
    Dna
} from "lucide-react";
import PageTransition from "../components/PageTransition";
import { Transaction } from '../context/AppContext';
import { formatNumber, calculateFee } from '../utils/format';
// import { cn } from "../lib/utils";

export default function Mempool() {
    const [mempool, setMempool] = useState<Transaction[]>([]);
    const [selectedTx, setSelectedTx] = useState<Transaction | null>(null);

    useEffect(() => {
        fetchMempool();
        const interval = setInterval(fetchMempool, 2000);
        return () => clearInterval(interval);
    }, []);

    async function fetchMempool() {
        try {
            const txs = await invoke<Transaction[]>('get_mempool_transactions');
            const sortedTxs = txs.sort((a, b) => b.timestamp - a.timestamp);
            setMempool(sortedTxs);
        } catch (err) {
            console.error('Failed to fetch mempool:', err);
        }
    }

    return (
        <PageTransition>
            <div className="flex flex-col gap-6 min-h-full lg:h-full lg:overflow-hidden container mx-auto p-4 sm:p-6 lg:max-w-7xl pb-10 lg:pb-0 px-2 sm:px-4">

                {/* Header Section */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 shrink-0 px-2">
                    <div className="flex items-center gap-5">
                        <div className="p-3.5 glass-panel rounded-2xl border-primary/20 bg-primary/5 shadow-inner">
                            <Clock className="w-7 h-7 text-orange-500 animate-pulse" />
                        </div>
                        <div>
                            <h1 className="text-3xl font-black tracking-tightest uppercase italic text-foreground">Pending Pulse</h1>
                            <p className="text-[10px] text-muted-foreground flex items-center gap-2 font-black uppercase tracking-[0.25em] mt-1">
                                <Activity className="w-3.5 h-3.5 text-orange-500/60" /> Real-time Transaction Stream
                            </p>
                        </div>
                    </div>
                    <div className="px-8 py-3 rounded-full glass-panel border-orange-500/20 bg-orange-500/5">
                        <span className="text-[11px] font-black text-orange-500 tracking-[0.2em] uppercase italic">
                            Pipeline Occupancy: {mempool.length} TXS
                        </span>
                    </div>
                </div>

                {/* Main Mempool Content */}
                <div className="flex-1 glass-card border-white/20 dark:border-white/5 rounded-3xl relative overflow-hidden flex flex-col min-h-0">
                    <div className="absolute top-0 right-0 w-64 h-64 bg-orange-500/5 rounded-full -mr-32 -mt-32 blur-3xl opacity-50 pointer-events-none" />

                    <div className="p-6 border-b border-border/50 shrink-0 flex items-center justify-between z-10 bg-secondary/10">
                        <div className="flex items-center gap-4">
                            <div className="p-2.5 glass-panel rounded-xl border-white/10 dark:border-white/5">
                                <Search className="w-5 h-5 text-orange-500" />
                            </div>
                            <span className="text-[11px] font-black uppercase tracking-[0.3em] text-muted-foreground italic">Unconfirmed Sequencer</span>
                        </div>
                        <div className="flex items-center gap-3">
                            <Badge variant="outline" className="border-orange-500/20 text-orange-500 font-black text-[9px] px-4 py-1.5 rounded-full uppercase tracking-widest italic bg-orange-500/5">
                                Live Ingress
                            </Badge>
                        </div>
                    </div>

                    <div className="flex-1 min-h-0 overflow-y-auto hidden-scrollbar relative z-10">
                        {mempool.length === 0 ? (
                            <div className="h-full flex flex-col items-center justify-center py-32 text-muted-foreground gap-8">
                                <div className="relative">
                                    <Clock className="w-20 h-20 opacity-20" />
                                    <div className="absolute inset-0 border-2 border-dashed border-orange-500/20 rounded-full animate-spin p-12 -m-4" style={{ animationDuration: '10s' }} />
                                </div>
                                <div className="text-center space-y-2">
                                    <p className="font-black text-xl uppercase tracking-[0.3em] italic">Pipeline Clear</p>
                                    <p className="text-[10px] uppercase font-bold tracking-widest opacity-50">Zero unconfirmed transfers detected</p>
                                </div>
                            </div>
                        ) : (
                            <div className="divide-y divide-border/50">
                                {mempool.map((tx) => (
                                    <div
                                        key={tx.id}
                                        className="group/tx p-6 md:p-8 hover:bg-orange-500/[0.03] transition-all cursor-pointer relative overflow-hidden flex flex-col md:flex-row md:items-center justify-between gap-6"
                                        onClick={() => setSelectedTx(tx)}
                                    >
                                        <div className="flex items-center gap-6 md:gap-8 flex-1 min-w-0">
                                            <div className="w-14 h-14 rounded-2xl glass-panel bg-orange-500/5 flex items-center justify-center shrink-0 group-hover/tx:scale-110 group-hover/tx:bg-orange-500/10 transition-all duration-500 shadow-inner">
                                                <Zap className="w-6 h-6 text-orange-500" />
                                            </div>
                                            <div className="flex flex-col gap-3 min-w-0 flex-1">
                                                <div className="flex items-center gap-3">
                                                    <span className="text-[10px] font-black text-orange-500/60 uppercase tracking-widest leading-none">Signature Identification</span>
                                                    <Badge variant="secondary" className="text-[9px] font-mono font-black opacity-60 px-2 h-5">PENDING</Badge>
                                                </div>
                                                <div className="font-mono text-sm font-black text-foreground/80 truncate group-hover/tx:text-orange-500 transition-colors" title={tx.id}>
                                                    {tx.id}
                                                </div>
                                                <div className="flex items-center gap-4 text-[10px] font-bold text-muted-foreground uppercase tracking-tight">
                                                    <div className="flex items-center gap-2">
                                                        <User className="w-3.5 h-3.5 opacity-50" />
                                                        <span className="truncate max-w-[120px] italic">{tx.sender}</span>
                                                    </div>
                                                    <ChevronRight className="w-3.5 h-3.5" />
                                                    <div className="flex items-center gap-2">
                                                        <User className="w-3.5 h-3.5 opacity-50" />
                                                        <span className="truncate max-w-[120px] italic">{tx.receiver}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div className="flex items-center justify-between md:justify-end gap-10 shrink-0 md:pl-6 border-t md:border-t-0 md:border-l border-border/50 pt-6 md:pt-0">
                                            <div className="text-right">
                                                <span className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] block mb-2">Transfer Quantum</span>
                                                <div className="text-3xl font-black text-foreground italic tracking-tightest leading-none">
                                                    {formatNumber(tx.amount)} <span className="text-lg text-orange-500/50 ml-1">AGT</span>
                                                </div>
                                            </div>
                                            <div className="w-12 h-12 rounded-2xl glass-panel flex items-center justify-center text-muted-foreground/30 group-hover/tx:text-orange-500 transition-all group-hover:rotate-12">
                                                <Terminal className="w-6 h-6" />
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Transaction Detail Overlay */}
                {selectedTx && (
                    <div className="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-background/80 backdrop-blur-xl animate-in fade-in duration-300">
                        <div
                            className="glass-card bg-popover/90 border-white/10 shadow-2xl rounded-[3rem] w-full max-w-3xl overflow-hidden animate-in slide-in-from-bottom-12 duration-500 flex flex-col max-h-[90vh]"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <div className="absolute top-0 right-0 p-8 z-20">
                                <button
                                    onClick={() => setSelectedTx(null)}
                                    className="p-3 rounded-2xl glass-panel hover:bg-destructive/10 hover:text-destructive transition-all group"
                                >
                                    <X className="w-6 h-6 group-hover:rotate-90 transition-transform" />
                                </button>
                            </div>

                            <div className="p-8 md:p-12 space-y-10 overflow-y-auto">
                                {/* Header */}
                                <div className="flex items-center gap-6">
                                    <div className="p-5 glass-panel rounded-[2rem] bg-orange-500/10 border-orange-500/20 text-orange-500">
                                        <Dna className="w-8 h-8" />
                                    </div>
                                    <div>
                                        <h2 className="text-3xl font-black tracking-tightest text-foreground italic uppercase">Mempool Receipt</h2>
                                        <p className="text-[10px] text-orange-500/60 uppercase tracking-[0.4em] font-black mt-1 italic">Awaiting Ledger Confirmation</p>
                                    </div>
                                </div>

                                {/* Flow Visualization */}
                                <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-8 items-center glass-panel bg-secondary/20 p-8 rounded-[2.5rem] border-white/5 relative">
                                    <div className="space-y-4 min-w-0">
                                        <span className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.3em] italic pl-2">Source Node</span>
                                        <div className="p-5 rounded-3xl bg-background/50 border border-border flex items-center gap-4">
                                            <div className="w-10 h-10 rounded-2xl bg-orange-500/10 flex items-center justify-center shrink-0">
                                                <User className="w-5 h-5 text-orange-500" />
                                            </div>
                                            <span className="text-xs font-mono font-black truncate italic text-foreground" title={selectedTx.sender}>{selectedTx.sender}</span>
                                        </div>
                                    </div>
                                    <div className="flex md:flex-col items-center justify-center gap-4">
                                        <ArrowRight className="w-6 h-6 text-orange-500 animate-pulse hidden md:block" />
                                        <div className="block md:hidden rotate-90"><ArrowRight className="w-6 h-6 text-orange-500 animate-pulse" /></div>
                                    </div>
                                    <div className="space-y-4 min-w-0">
                                        <span className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.3em] italic pl-2">Target Node</span>
                                        <div className="p-5 rounded-3xl bg-background/50 border border-border flex items-center gap-4">
                                            <div className="w-10 h-10 rounded-2xl bg-emerald-500/10 flex items-center justify-center shrink-0">
                                                <User className="w-5 h-5 text-emerald-500" />
                                            </div>
                                            <span className="text-xs font-mono font-black truncate italic text-foreground" title={selectedTx.receiver}>{selectedTx.receiver}</span>
                                        </div>
                                    </div>
                                </div>

                                {/* Core Metadata */}
                                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                                    <div className="space-y-6">
                                        <div className="flex flex-col gap-3 min-w-0">
                                            <div className="flex items-center gap-3 text-[10px] font-black uppercase text-muted-foreground tracking-[0.3em] italic">
                                                <Shield className="w-4 h-4 text-orange-500" />
                                                <span>Pending Signature</span>
                                            </div>
                                            <div className="font-mono text-xs font-medium text-muted-foreground bg-secondary/30 p-4 rounded-2xl border border-border break-all">
                                                {selectedTx.id}
                                            </div>
                                        </div>
                                        <div className="p-4 glass-panel bg-orange-500/5 border-orange-500/10 rounded-2xl flex items-center gap-4 animate-pulse">
                                            <Shield className="w-5 h-5 text-orange-500" />
                                            <span className="text-[10px] font-black text-orange-500/80 uppercase tracking-widest italic">Stationed in transit pipeline</span>
                                        </div>
                                    </div>
                                    <div className="p-8 glass-panel bg-orange-500/5 rounded-[2.5rem] border-orange-500/10 flex flex-col justify-center relative overflow-hidden">
                                        <span className="text-[10px] font-black text-orange-500 uppercase tracking-[0.4em] mb-4 italic z-10">Ingress Magnitude</span>
                                        <div className="flex items-baseline gap-3 z-10">
                                            <span className="text-5xl font-black text-foreground tracking-tightest italic leading-none">
                                                {formatNumber(selectedTx.amount)}
                                            </span>
                                            <span className="text-xl font-black text-orange-500/40 uppercase">AGT</span>
                                        </div>
                                        <div className="mt-4 flex items-center gap-3 text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] italic z-10">
                                            <Zap className="w-4 h-4 text-orange-500" />
                                            <span>Est. Fee: {formatNumber(calculateFee(selectedTx.amount))} AGT</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </PageTransition>
    );
}
