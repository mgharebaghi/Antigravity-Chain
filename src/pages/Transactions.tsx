import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Send, Coins, RefreshCw, User, ShieldCheck, Activity, ArrowRight, ArrowUpRight, ArrowDownLeft, Search, Filter } from "lucide-react";
import PageTransition from "../components/PageTransition";
import { useToast } from "../context/ToastContext";
import { useApp } from "../context/AppContext";
import { formatNumber, calculateFee, parseAmount } from "../utils/format";
import { cn } from "../lib/utils";

export default function Transactions() {
    const { success, error } = useToast();
    const { wallet, refreshWallet, recentBlocks } = useApp();
    const [receiver, setReceiver] = useState('');
    const [amount, setAmount] = useState('');
    const [loading, setLoading] = useState(false);

    const atomicAmount = parseAmount(amount);
    const fee = calculateFee(atomicAmount);
    const total = atomicAmount + fee;
    const balance = wallet?.balance || 0;
    const isInsufficient = total > balance;

    const [currentPage, setCurrentPage] = useState(1);
    const ITEMS_PER_PAGE = 7;

    // Flatten and filter transactions related to this wallet
    const allTransactions = recentBlocks
        .flatMap(block => block.transactions.map(tx => ({ ...tx, blockIndex: block.index, blockTime: block.timestamp })))
        .filter(tx => tx.sender === wallet?.address || tx.receiver === wallet?.address)
        .sort((a, b) => b.blockTime - a.blockTime);

    const totalPages = Math.ceil(allTransactions.length / ITEMS_PER_PAGE);
    const paginatedTransactions = allTransactions.slice(
        (currentPage - 1) * ITEMS_PER_PAGE,
        currentPage * ITEMS_PER_PAGE
    );

    const handlePageChange = (newPage: number) => {
        setCurrentPage(newPage);
        // Scroll to top of list container if needed, but since it's inside a glass card it should be fine
    };

    async function sendTransaction(e: React.FormEvent) {
        e.preventDefault();

        if (!receiver.startsWith('12D3') || receiver.length < 40) {
            error("Invalid address format. Addresses should start with '12D3' and be 40+ characters.");
            return;
        }

        setLoading(true);
        try {
            const id = await invoke<string>('submit_transaction', {
                receiver,
                amount: atomicAmount
            });
            success(`Transaction successful! ID: ${id.substring(0, 8)}...`);
            setReceiver('');
            setAmount('');
            refreshWallet();
        } catch (err) {
            error(`Failed to send: ${err}`);
        } finally {
            setLoading(false);
        }
    }

    return (
        <PageTransition>
            <div className="flex flex-col h-[calc(100vh-8rem)] gap-6">

                {/* Header */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 shrink-0">
                    <div>
                        <h1 className="text-3xl font-extrabold tracking-tight">Activity</h1>
                        <p className="text-muted-foreground mt-1">Manage transfers and view your history.</p>
                    </div>
                    <div className="flex items-center gap-2">
                        <Badge variant="outline" className="h-9 px-3 gap-2 text-sm font-normal">
                            <ShieldCheck className="w-4 h-4 text-emerald-500" /> Secure Connection
                        </Badge>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">

                    {/* Left Panel: Send Transaction */}
                    <div className="lg:col-span-5 xl:col-span-4 flex flex-col gap-6">
                        <div className="glass-card p-6 rounded-3xl flex flex-col gap-6 h-full shadow-xl shadow-primary/5 border border-white/20 dark:border-white/10 relative overflow-hidden">
                            {/* Background decoration */}
                            <div className="absolute -top-20 -right-20 w-60 h-60 bg-primary/10 rounded-full blur-3xl pointer-events-none" />

                            <div className="relative z-10 flex flex-col gap-1">
                                <h2 className="text-lg font-bold flex items-center gap-2">
                                    <Send className="w-5 h-5 text-primary" /> Send Crypto
                                </h2>
                                <p className="text-xs text-muted-foreground">Transfer funds securely to any address.</p>
                            </div>

                            <form onSubmit={sendTransaction} className="flex-1 flex flex-col gap-6 relative z-10">
                                {/* Amount Input */}
                                <div className="space-y-3">
                                    <div className="flex justify-between text-xs font-semibold px-1">
                                        <span className="text-muted-foreground">Amount</span>
                                        <span className={cn("transition-colors", isInsufficient ? "text-destructive" : "text-primary cursor-pointer hover:underline")} onClick={() => setAmount(((balance - fee) > 0 ? (balance - fee) : 0).toString())}>
                                            Max: {formatNumber(balance)} AGT
                                        </span>
                                    </div>
                                    <div className="relative group">
                                        <div className="absolute left-4 top-1/2 -translate-y-1/2 p-1.5 rounded-lg bg-secondary text-primary">
                                            <Coins className="w-4 h-4" />
                                        </div>
                                        <input
                                            type="number"
                                            value={amount}
                                            onChange={(e) => setAmount(e.target.value)}
                                            className="w-full bg-secondary/30 border border-border focus:border-primary/50 rounded-2xl py-4 pl-14 pr-16 text-2xl font-bold placeholder:text-muted-foreground/30 focus:outline-none focus:ring-4 focus:ring-primary/10 transition-all"
                                            placeholder="0.00"
                                            min="0.000001"
                                            step="0.000001"
                                            required
                                        />
                                        <div className="absolute right-4 top-1/2 -translate-y-1/2 text-xs font-bold text-muted-foreground bg-secondary/50 px-2 py-1 rounded-md">AGT</div>
                                    </div>
                                    {amount && !isInsufficient && (
                                        <div className="flex justify-between px-2 text-xs text-muted-foreground">
                                            <span>Fee: {formatNumber(fee)}</span>
                                            <span>Total: {formatNumber(total)}</span>
                                        </div>
                                    )}
                                </div>

                                {/* Receiver Input */}
                                <div className="space-y-3">
                                    <label className="text-xs font-semibold text-muted-foreground px-1">Receiver Address</label>
                                    <div className="relative group">
                                        <div className="absolute left-4 top-1/2 -translate-y-1/2 p-1.5 rounded-lg bg-secondary text-muted-foreground group-focus-within:text-primary transition-colors">
                                            <User className="w-4 h-4" />
                                        </div>
                                        <input
                                            type="text"
                                            value={receiver}
                                            onChange={(e) => setReceiver(e.target.value)}
                                            className="w-full bg-secondary/30 border border-border focus:border-primary/50 rounded-2xl py-4 pl-14 pr-4 text-xs font-mono font-medium text-foreground focus:outline-none focus:ring-4 focus:ring-primary/10 transition-all placeholder:text-muted-foreground/50"
                                            placeholder="12D3..."
                                            required
                                        />
                                    </div>
                                </div>

                                <div className="flex-1" />

                                <Button
                                    type="submit"
                                    size="lg"
                                    className="w-full h-14 rounded-2xl text-base font-bold shadow-lg shadow-primary/20 hover:shadow-primary/30 transition-all"
                                    disabled={loading || isInsufficient || !amount || parseFloat(amount) <= 0}
                                >
                                    {loading ? (
                                        <>
                                            <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> Processing...
                                        </>
                                    ) : (
                                        <>
                                            Send Transaction <ArrowRight className="w-4 h-4 ml-2" />
                                        </>
                                    )}
                                </Button>
                            </form>
                        </div>
                    </div>

                    {/* Right Panel: Transaction History */}
                    <div className="lg:col-span-7 xl:col-span-8 flex flex-col gap-6 h-full overflow-hidden">
                        <div className="glass-card rounded-3xl flex flex-col h-full border border-white/20 dark:border-white/10 overflow-hidden">
                            {/* Toolbar */}
                            <div className="p-6 border-b border-border/50 flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-secondary/20">
                                <h2 className="text-lg font-bold flex items-center gap-2">
                                    <Activity className="w-5 h-5 text-primary" /> History
                                </h2>
                                <div className="flex items-center gap-2">
                                    <div className="relative">
                                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                                        <input
                                            placeholder="Search tx..."
                                            className="h-9 w-40 sm:w-64 bg-background border border-border rounded-xl pl-9 pr-3 text-xs focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all"
                                        />
                                    </div>
                                    <Button variant="ghost" size="icon" className="h-9 w-9 rounded-xl border border-border">
                                        <Filter className="w-4 h-4 text-muted-foreground" />
                                    </Button>
                                </div>
                            </div>

                            {/* List */}
                            <div className="flex-1 overflow-y-auto p-2 sm:p-4 space-y-2">
                                {paginatedTransactions.length > 0 ? (
                                    paginatedTransactions.map((tx: any) => {
                                        const isOutgoing = tx.sender === wallet?.address;
                                        return (
                                            <div
                                                key={tx.id}
                                                className="group p-4 rounded-2xl bg-background/40 hover:bg-background/80 border border-transparent hover:border-border transition-all flex items-center gap-4 cursor-default"
                                            >
                                                <div className={cn(
                                                    "w-12 h-12 rounded-xl flex items-center justify-center shrink-0 transition-colors",
                                                    isOutgoing ? "bg-red-500/10 text-red-500" : "bg-emerald-500/10 text-emerald-500"
                                                )}>
                                                    {isOutgoing ? <ArrowUpRight className="w-6 h-6" /> : <ArrowDownLeft className="w-6 h-6" />}
                                                </div>

                                                <div className="flex-1 min-w-0">
                                                    <div className="flex items-center gap-2 mb-1">
                                                        <span className="font-bold text-sm">
                                                            {isOutgoing ? "Sent to" : "Received from"}
                                                        </span>
                                                        <span className="font-mono text-xs text-muted-foreground truncate max-w-[120px]">
                                                            {isOutgoing ? tx.receiver : tx.sender}
                                                        </span>
                                                    </div>
                                                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                                                        <span className="font-mono">{new Date(tx.blockTime * 1000).toLocaleTimeString()}</span>
                                                        <span className="w-1 h-1 rounded-full bg-border" />
                                                        <span className="font-mono truncate max-w-[100px] opacity-50 hover:opacity-100 transition-opacity">ID: {tx.id.substring(0, 8)}...</span>
                                                    </div>
                                                </div>

                                                <div className="text-right">
                                                    <div className={cn("text-base font-bold font-mono", isOutgoing ? "text-foreground" : "text-emerald-500")}>
                                                        {isOutgoing ? "-" : "+"}{formatNumber(tx.amount)}
                                                    </div>
                                                    <div className="text-[10px] font-bold text-muted-foreground bg-secondary/50 px-2 py-0.5 rounded-lg inline-block mt-1">
                                                        confirmed
                                                    </div>
                                                </div>
                                            </div>
                                        );
                                    })
                                ) : (
                                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground gap-4 opacity-50 py-20">
                                        <div className="w-16 h-16 rounded-full bg-secondary/50 flex items-center justify-center">
                                            <Activity className="w-8 h-8" />
                                        </div>
                                        <p>No transactions found</p>
                                    </div>
                                )}
                            </div>

                            {/* Pagination Footer */}
                            {allTransactions.length > ITEMS_PER_PAGE && (
                                <div className="p-4 border-t border-border/50 bg-secondary/10 flex items-center justify-between shrink-0">
                                    <div className="text-xs text-muted-foreground">
                                        Page <span className="font-bold text-foreground">{currentPage}</span> of {totalPages}
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() => handlePageChange(currentPage - 1)}
                                            disabled={currentPage === 1}
                                            className="h-8 rounded-xl px-3 text-xs"
                                        >
                                            Previous
                                        </Button>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() => handlePageChange(currentPage + 1)}
                                            disabled={currentPage === totalPages}
                                            className="h-8 rounded-xl px-3 text-xs"
                                        >
                                            Next
                                        </Button>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </PageTransition>
    );
}
