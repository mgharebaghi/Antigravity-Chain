import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Clock, Hash } from "lucide-react";
import PageTransition from "../components/PageTransition";

import { Transaction } from '../context/AppContext';
import { formatNumber } from '../utils/format';
import TxDetailsModal from '../components/TxDetailsModal';

// import { useToast } from "../context/ToastContext";

export default function Mempool() {
    const [mempool, setMempool] = useState<Transaction[]>([]);
    const [selectedTx, setSelectedTx] = useState<Transaction | null>(null);
    // const { error } = useToast();

    useEffect(() => {
        fetchMempool();
        const interval = setInterval(fetchMempool, 2000);
        return () => clearInterval(interval);
    }, []);

    async function fetchMempool() {
        try {
            const txs = await invoke<Transaction[]>('get_mempool_transactions');
            // Sort by timestamp descending (newest first)
            const sortedTxs = txs.sort((a, b) => b.timestamp - a.timestamp);
            setMempool(sortedTxs);
        } catch (err) {
            // Only show error if it's not just an empty mempool issue
            console.error('Failed to fetch mempool:', err);
            // Optional: Uncomment if you want to spam the user with connection errors
            // error(`Failed to fetch mempool: ${err}`);
        }
    }

    return (
        <PageTransition>
            <div className="flex flex-col min-h-full gap-3 sm:gap-6 container mx-auto p-4 sm:p-6 lg:max-w-6xl pb-10">
                <div className="flex items-center gap-2 sm:gap-3 shrink-0 px-1">
                    <div className="p-1.5 sm:p-2 bg-primary/10 rounded-lg">
                        <Clock className="w-5 h-5 sm:w-6 sm:h-6 text-primary" />
                    </div>
                    <div>
                        <h1 className="text-xl sm:text-2xl font-bold tracking-tight">Mempool Explorer</h1>
                        <p className="text-[10px] sm:text-sm text-muted-foreground font-medium">Technical view of pending mesh transactions</p>
                    </div>
                </div>

                <Card className="flex flex-col border-border/50 shadow-2xl bg-card/30 backdrop-blur-xl rounded-2xl sm:rounded-[2rem]">
                    <CardHeader className="border-b border-border/50 pb-2.5 sm:pb-4 bg-muted/20 shrink-0 px-4 sm:px-8">
                        <div className="flex items-center justify-between">
                            <CardTitle className="text-xs sm:text-lg flex items-center gap-2 font-black uppercase tracking-widest opacity-80">
                                <Clock className="w-4 h-4 sm:w-5 h-5 text-orange-500" />
                                Pipeline <span className="hidden xs:inline">Congestion</span>
                            </CardTitle>
                            <div className="px-2 sm:px-3 py-0.5 sm:py-1 rounded-full bg-orange-500/10 text-orange-500 text-[9px] sm:text-xs font-black border border-orange-500/20 uppercase tracking-tighter sm:tracking-widest">
                                {formatNumber(mempool.length, false)} <span className="hidden sm:inline">Active</span> TXS
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="p-0 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent min-h-0">
                        {mempool.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-16 text-muted-foreground opacity-50 space-y-3 h-full">
                                <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center">
                                    <Clock className="w-6 h-6" />
                                </div>
                                <p>No pending transactions</p>
                            </div>
                        ) : (
                            <div className="divide-y divide-border/40">
                                {mempool.map((tx) => (
                                    <div
                                        key={tx.id}
                                        className="p-4 hover:bg-muted/30 transition-colors group cursor-pointer"
                                        onClick={() => setSelectedTx(tx)}
                                    >
                                        <div className="flex justify-between items-start mb-2">
                                            <div className="flex items-center gap-2">
                                                <div className="p-1.5 rounded-md bg-orange-500/10 text-orange-500">
                                                    <Hash className="w-3.5 h-3.5" />
                                                </div>
                                                <span className="font-mono text-xs text-muted-foreground group-hover:text-foreground transition-colors truncate w-full sm:w-auto" title={tx.id}>
                                                    {tx.id.substring(0, 24)}...
                                                </span>
                                            </div>
                                            <span className="font-bold text-green-500 flex items-center bg-green-500/5 px-2 py-0.5 rounded border border-green-500/10 shrink-0 ml-2">
                                                +{formatNumber(tx.amount)} AG
                                            </span>
                                        </div>

                                        <div className="text-xs space-y-1.5 pl-8">
                                            <div className="flex items-center justify-between text-muted-foreground gap-4">
                                                <div className="flex items-center gap-1 min-w-0">
                                                    <span>From:</span>
                                                    <span className="font-mono bg-muted px-1.5 py-0.5 rounded text-[10px] sm:text-xs text-foreground/80 truncate max-w-[120px] sm:max-w-[200px]" title={tx.sender}>
                                                        {tx.sender}
                                                    </span>
                                                </div>
                                                <div className="flex items-center gap-1 min-w-0">
                                                    <span>To:</span>
                                                    <span className="font-mono bg-muted px-1.5 py-0.5 rounded text-[10px] sm:text-xs text-foreground/80 truncate max-w-[120px] sm:max-w-[200px]" title={tx.receiver}>
                                                        {tx.receiver}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>

            <TxDetailsModal
                tx={selectedTx}
                onClose={() => setSelectedTx(null)}
            />
        </PageTransition>
    );
}
