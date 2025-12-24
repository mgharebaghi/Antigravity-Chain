import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { cn } from "../lib/utils";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";
import PageTransition from "../components/PageTransition";
import { Wallet as WalletIcon, Send, RefreshCw, Copy, Plus, Eye, EyeOff, Key, Info, ShieldCheck, History as HistoryIcon, Pickaxe, Activity, X, ExternalLink, Clock, ArrowDownLeft, Database, ChevronLeft, ChevronRight } from "lucide-react";
import { useApp, WalletExport, Block, Transaction } from "../context/AppContext";
import { useNavigate } from "react-router-dom";
import { useToast } from "../context/ToastContext";
import React, { useState } from "react";
import { formatNumber, ONE_AGT } from "../utils/format";
import SwitchWalletModal from "../components/SwitchWalletModal";

export default function Wallet() {
    const { wallet, createWallet: createWalletFn, importWallet: importWalletFn, loading, recentBlocks, logout } = useApp();
    const navigate = useNavigate();
    const { success, error, info } = useToast();

    const [createdWallet, setCreatedWallet] = useState<WalletExport | null>(null);
    const [importKey, setImportKey] = useState("");
    const [isImporting, setIsImporting] = useState(false);
    const [showPrivateKey, setShowPrivateKey] = useState(false);
    const [refreshing, setRefreshing] = useState(false);
    const [selectedActivity, setSelectedActivity] = useState<any>(null);
    const [isSwitchModalOpen, setIsSwitchModalOpen] = useState(false);
    const [activityPage, setActivityPage] = useState(1);
    const ACTIVITY_PAGE_SIZE = 6;

    const { refreshWallet, refreshBlockHeight } = useApp();

    const handleRefresh = async () => {
        setRefreshing(true);
        try {
            await Promise.all([refreshWallet(), refreshBlockHeight()]);
            success("Wallet balance refreshed!");
        } catch (err) {
            error("Refresh failed: " + err);
        } finally {
            setRefreshing(false);
        }
    };

    const handleCreateWallet = async () => {
        try {
            const result = await createWalletFn();
            setCreatedWallet(result);
            success("Wallet created! Please save your keys.");
        } catch (err) {
            error("Failed to create wallet: " + err);
        }
    };

    const handleImportSubmit = async () => {
        if (!importKey) return;
        try {
            await importWalletFn(importKey);
            success("Wallet imported successfully!");
            setIsImporting(false);
        } catch (err) {
            error("Import failed: " + err);
        }
    };

    // Extract rewards for this wallet from recent blocks
    const allRewards = recentBlocks.flatMap((b: Block) => {
        // Find explicit systemic rewards
        const explicitRewards = b.transactions
            .filter((tx: Transaction) => tx.receiver === wallet?.address && tx.sender === "SYSTEM")
            .map((tx: Transaction) => ({ ...tx, blockIndex: b.index }));

        // If no explicit reward tx but authored by us, show as a synthetic reward
        if (explicitRewards.length === 0 && b.author === wallet?.address) {
            return [{
                id: `reward-${b.index}`,
                sender: "SYSTEM",
                receiver: b.author,
                amount: b.index === 0 ? 5000000 * ONE_AGT : 40 * ONE_AGT,
                timestamp: b.timestamp,
                blockIndex: b.index,
                signature: "reward"
            }];
        }
        return explicitRewards;
    });

    const totalActivityPages = Math.ceil(allRewards.length / ACTIVITY_PAGE_SIZE);
    const myRewards = allRewards.slice((activityPage - 1) * ACTIVITY_PAGE_SIZE, activityPage * ACTIVITY_PAGE_SIZE);


    return (
        <PageTransition>
            <div className="flex flex-col min-h-full gap-3 md:gap-4 pb-6">
                <div className="flex items-center gap-2 shrink-0">
                    <WalletIcon className="w-6 h-6 text-primary" />
                    <h1 className="text-2xl font-bold tracking-tight">Wallet</h1>
                    {wallet && (
                        <Button
                            variant="outline"
                            size="sm"
                            className="ml-auto gap-2 text-muted-foreground hover:text-destructive hover:border-destructive transition-colors"
                            onClick={() => setIsSwitchModalOpen(true)}
                        >
                            <X className="w-4 h-4" />
                            Switch Wallet
                        </Button>
                    )}
                </div>

                {!wallet || createdWallet ? (
                    <Card className="flex-1 flex flex-col items-center justify-center border-dashed border-2 relative overflow-hidden bg-card/10 backdrop-blur-sm shadow-inner min-h-[500px] py-12">
                        <div className="absolute top-0 right-0 p-4">
                            <Button variant="ghost" size="sm" onClick={() => setIsImporting(!isImporting)} className="gap-2">
                                {isImporting ? "Go Back" : "I already have a wallet"}
                            </Button>
                        </div>

                        {createdWallet ? (
                            <div className="p-8 max-w-md w-full space-y-6">
                                <div className="text-center space-y-2">
                                    <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto text-green-500">
                                        <ShieldCheck className="w-8 h-8" />
                                    </div>
                                    <h3 className="text-2xl font-bold">Secure Your Wallet</h3>
                                    <p className="text-sm text-muted-foreground">Save the private key below. You will need it to recover your wallet.</p>
                                </div>

                                <div className="space-y-4">
                                    <div className="space-y-2">
                                        <label className="text-xs font-semibold uppercase text-muted-foreground flex items-center gap-2">
                                            <Key className="w-3 h-3" /> Private Key (HEX)
                                        </label>
                                        <div className="relative group">
                                            <input
                                                type={showPrivateKey ? "text" : "password"}
                                                readOnly
                                                value={createdWallet.private_key}
                                                className="w-full p-4 bg-secondary/30 rounded-xl border border-primary/10 text-xs font-mono pr-24 focus:outline-none"
                                            />
                                            <div className="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                                                <Button size="icon" variant="ghost" className="w-8 h-8 rounded-lg" onClick={() => setShowPrivateKey(!showPrivateKey)}>
                                                    {showPrivateKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                                                </Button>
                                                <Button size="icon" variant="ghost" className="w-8 h-8 rounded-lg" onClick={() => {
                                                    navigator.clipboard.writeText(createdWallet.private_key);
                                                    success("Private key copied!");
                                                }}>
                                                    <Copy className="w-4 h-4" />
                                                </Button>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="space-y-3">
                                        <label className="text-xs font-semibold uppercase text-muted-foreground flex items-center gap-2">
                                            <ShieldCheck className="w-3 h-3" /> Recovery Phrase (Mnemonic)
                                        </label>
                                        <div className="p-4 bg-primary/5 border border-primary/10 rounded-2xl relative group">
                                            <div className="grid grid-cols-3 gap-2">
                                                {createdWallet.mnemonic.split(' ').map((word, i) => (
                                                    <div key={i} className="flex items-center gap-2 bg-background/50 px-2 py-1.5 rounded-lg border border-primary/5">
                                                        <span className="text-[10px] text-muted-foreground font-mono">{i + 1}.</span>
                                                        <span className="text-xs font-bold font-mono text-foreground">{word}</span>
                                                    </div>
                                                ))}
                                            </div>
                                            <Button
                                                size="icon"
                                                variant="secondary"
                                                className="absolute -top-2 -right-2 w-8 h-8 rounded-full shadow-lg border border-primary/20 opacity-0 group-hover:opacity-100 transition-opacity"
                                                onClick={() => {
                                                    navigator.clipboard.writeText(createdWallet.mnemonic);
                                                    success("Recovery phrase copied!");
                                                }}
                                            >
                                                <Copy className="w-3.5 h-3.5" />
                                            </Button>
                                        </div>
                                        <p className="text-[10px] text-muted-foreground text-center italic">Tip: Write these 12 words down in order on a piece of paper.</p>
                                    </div>

                                    <div className="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-2xl flex gap-3 text-xs text-yellow-600/90 leading-relaxed font-medium">
                                        <Info className="w-5 h-5 shrink-0 text-yellow-500" />
                                        <p>CRITICAL: If you lose these keys, your funds cannot be recovered. Antigravity never stores your private keys or phrases.</p>
                                    </div>

                                    <Button className="w-full h-14 text-base font-bold rounded-2xl shadow-xl shadow-primary/20 hover:scale-[1.02] transition-all" onClick={() => setCreatedWallet(null)}>
                                        I've Secured My Keys, Proceed
                                    </Button>
                                </div>
                            </div>
                        ) : isImporting ? (
                            <div className="p-8 max-w-md w-full space-y-6">
                                <div className="text-center space-y-2">
                                    <h3 className="text-2xl font-bold">Restore Identity</h3>
                                    <p className="text-sm text-muted-foreground">Enter your 12-word recovery phrase or HEX private key.</p>
                                </div>
                                <div className="space-y-4">
                                    <div className="relative group">
                                        <textarea
                                            placeholder="Example: word1 word2 word3..."
                                            value={importKey}
                                            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setImportKey(e.target.value)}
                                            className="w-full h-32 p-4 bg-secondary/30 rounded-xl border border-primary/10 text-xs font-mono focus:outline-none resize-none"
                                        />
                                        <div className="absolute right-2 bottom-2 text-[10px] text-muted-foreground/50 font-mono">
                                            {importKey.split(/\s+/).filter(Boolean).length} WORDS
                                        </div>
                                    </div>
                                    <Button className="w-full h-12 text-base font-bold rounded-xl" onClick={handleImportSubmit} disabled={!importKey || loading}>
                                        {loading ? "Decrypting..." : "Restore Account"}
                                    </Button>
                                    <p className="text-[10px] text-muted-foreground text-center">
                                        <ShieldCheck className="w-3 h-3 inline mr-1" /> All decryption happens locally.
                                    </p>
                                </div>
                            </div>
                        ) : (
                            <div className="flex flex-col items-center gap-4 p-6">
                                <div className="w-16 h-16 rounded-full bg-secondary flex items-center justify-center">
                                    <WalletIcon className="w-8 h-8 text-muted-foreground" />
                                </div>
                                <div className="space-y-2 text-center">
                                    <h3 className="text-xl font-semibold">Ready to Join?</h3>
                                    <p className="text-muted-foreground max-w-sm mx-auto text-sm">
                                        Generate a new identity to participate in consensus and earn patient rewards.
                                    </p>
                                </div>
                                <div className="flex flex-col gap-3 mt-4 w-full max-w-[240px]">
                                    <Button onClick={handleCreateWallet} size="lg" className="h-12 text-base gap-2 shadow-lg shadow-primary/20" disabled={loading}>
                                        <Plus className="w-5 h-5" /> Generate Identity
                                    </Button>
                                    <Button onClick={() => setIsImporting(true)} variant="outline" size="lg" className="h-12 text-base gap-2" disabled={loading}>
                                        <Key className="w-5 h-5" /> Import HEX Key
                                    </Button>
                                </div>
                            </div>
                        )}
                    </Card>
                ) : (
                    // Responsive Grid Layout
                    <div className="flex flex-col lg:grid lg:grid-cols-2 gap-4 sm:gap-6 flex-1 min-h-0">

                        {/* Balance Card: Full width on mobile, spans 2 on desktop */}
                        <Card className="lg:col-span-2 bg-gradient-to-br from-primary/20 via-primary/5 to-secondary/20 border-primary/20 relative overflow-hidden shrink-0">
                            <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent" />
                            <CardContent className="p-4 sm:p-6 lg:p-8 flex flex-col md:flex-row items-center md:justify-between gap-4 md:gap-8 h-full relative z-10">
                                <div className="flex flex-col items-center md:items-start text-center md:text-left min-w-0 flex-1">
                                    <div className="text-[10px] sm:text-xs text-muted-foreground mb-2 uppercase tracking-[0.2em] font-black">Current Balance</div>
                                    <div className="flex items-baseline gap-2 sm:gap-3 flex-wrap justify-center md:justify-start">
                                        <span className={cn(
                                            "font-black tracking-tighter text-foreground drop-shadow-sm leading-none transition-all duration-300",
                                            (formatNumber(wallet.balance)?.length || 0) > 15 ? "text-2xl sm:text-4xl lg:text-5xl" :
                                                (formatNumber(wallet.balance)?.length || 0) > 12 ? "text-3xl sm:text-5xl lg:text-6xl" :
                                                    "text-4xl sm:text-6xl lg:text-7xl"
                                        )}>
                                            {formatNumber(wallet.balance) || "0"}
                                        </span>
                                        <span className="text-lg sm:text-2xl font-black text-primary/80 shrink-0">AGT</span>
                                    </div>
                                    <div className="mt-3 flex items-center gap-2 text-[10px] sm:text-xs font-medium text-green-500 bg-green-500/10 px-3 py-1 rounded-full border border-green-500/20">
                                        <div className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                                        Network Synchronized
                                    </div>
                                </div>
                                <div className="flex flex-col sm:flex-row md:flex-col gap-3 w-full sm:w-auto shrink-0">
                                    <Button className="gap-2 h-12 md:h-14 px-8 rounded-2xl shadow-xl shadow-primary/20 font-bold text-base hover:scale-105 transition-all w-full md:w-44" size="lg" onClick={() => navigate('/transactions')}>
                                        <Send className="w-5 h-5" /> Transfer
                                    </Button>
                                    <Button
                                        className="gap-2 h-14 px-8 rounded-2xl font-bold text-base hover:bg-secondary/80 transition-all w-full md:w-44 border-2"
                                        variant="outline"
                                        size="lg"
                                        onClick={handleRefresh}
                                        disabled={refreshing}
                                    >
                                        <RefreshCw className={`w-5 h-5 ${refreshing ? 'animate-spin' : ''}`} /> {refreshing ? "Refreshing..." : "Receive"}
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Recent Activity: Mobile stacks under Balance */}
                        <Card className="lg:col-span-1 border-primary/10 flex flex-col min-h-0 bg-card/30 backdrop-blur-md">
                            <CardHeader className="pb-4 border-b border-primary/5 shrink-0 flex flex-row items-center justify-between">
                                <div className="space-y-1">
                                    <CardTitle className="text-base sm:text-lg font-bold">Activity Log</CardTitle>
                                    <p className="text-[10px] sm:text-xs text-muted-foreground">Recent block rewards & transfers</p>
                                </div>
                                <Badge variant="outline" className="rounded-lg h-7 px-3 bg-primary/5 border-primary/20 text-primary font-bold text-[10px]">REWARDS</Badge>
                            </CardHeader>
                            <CardContent className="flex-1 min-h-0 overflow-y-auto p-0 bg-gradient-to-b from-secondary/5 to-transparent flex flex-col">
                                {myRewards.length === 0 ? (
                                    <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground text-xs p-10 text-center gap-4">
                                        <div className="w-12 h-12 rounded-2xl bg-muted/50 flex items-center justify-center opacity-50">
                                            <HistoryIcon className="w-6 h-6" />
                                        </div>
                                        <p className="italic max-w-[180px]">Your node hasn't received any rewards in the recent blocks.</p>
                                    </div>
                                ) : (
                                    <>
                                        <div className="divide-y divide-primary/5 flex-1">
                                            {myRewards.map((reward) => (
                                                <div
                                                    key={reward.id}
                                                    className="p-3 sm:p-4 flex items-center justify-between hover:bg-primary/5 transition-all group cursor-pointer"
                                                    onClick={() => setSelectedActivity(reward)}
                                                >
                                                    <div className="flex items-center gap-3">
                                                        <div className="w-9 h-9 rounded-xl bg-primary/10 flex items-center justify-center text-primary border border-primary/10 group-hover:scale-110 transition-transform">
                                                            <Pickaxe className="w-4 h-4" />
                                                        </div>
                                                        <div>
                                                            <div className="font-bold text-xs">Block Reward</div>
                                                            <div className="text-[9px] text-muted-foreground flex items-center gap-1.5 mt-0.5">
                                                                <div className="w-1 h-1 rounded-full bg-primary/40" />
                                                                Block #{formatNumber(reward.blockIndex, false)}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div className="text-right">
                                                        <div className="font-mono font-black text-green-500 text-xs">+{formatNumber(reward.amount)} AGT</div>
                                                        <div className="text-[9px] uppercase tracking-widest text-muted-foreground font-semibold">Success</div>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>

                                        {/* Activity Pagination */}
                                        {totalActivityPages > 1 && (
                                            <div className="p-2 border-t border-primary/5 bg-muted/10 shrink-0 flex items-center justify-between">
                                                <div className="flex items-center gap-1 ml-1">
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        className="h-7 w-7 rounded-lg hover:bg-primary/10 disabled:opacity-30"
                                                        onClick={() => setActivityPage(p => Math.max(1, p - 1))}
                                                        disabled={activityPage === 1}
                                                    >
                                                        <ChevronLeft className="w-3.5 h-3.5" />
                                                    </Button>

                                                    <div className="flex items-center gap-1 mx-1">
                                                        {Array.from({ length: totalActivityPages }, (_, i) => i + 1).map((p) => (
                                                            <Button
                                                                key={p}
                                                                variant="ghost"
                                                                className={cn(
                                                                    "h-7 w-7 p-0 rounded-lg text-[10px] font-black transition-all",
                                                                    activityPage === p
                                                                        ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20"
                                                                        : "text-muted-foreground hover:bg-primary/10 hover:text-primary"
                                                                )}
                                                                onClick={() => setActivityPage(p)}
                                                            >
                                                                {p}
                                                            </Button>
                                                        ))}
                                                    </div>

                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        className="h-7 w-7 rounded-lg hover:bg-primary/10 disabled:opacity-30"
                                                        onClick={() => setActivityPage(p => Math.min(totalActivityPages, p + 1))}
                                                        disabled={activityPage === totalActivityPages}
                                                    >
                                                        <ChevronRight className="w-3.5 h-3.5" />
                                                    </Button>
                                                </div>
                                                <div className="mr-3 text-[9px] font-black uppercase text-muted-foreground/60 tracking-widest hidden sm:block">
                                                    Activity Index: <span className="text-primary/70">{allRewards.length}</span>
                                                </div>
                                            </div>
                                        )}
                                    </>
                                )}
                            </CardContent>
                        </Card>

                        {/* Details Stack: Row 2, Col 2 */}
                        <div className="lg:col-span-1 flex flex-col gap-4 sm:gap-6 min-h-0">
                            {/* Address Card */}
                            <Card className="shrink-0 bg-card/30 backdrop-blur-sm border-primary/5 transition-all hover:bg-card/50">
                                <CardHeader className="pb-2 pt-4 px-6 uppercase tracking-widest text-muted-foreground">
                                    <CardTitle className="text-[10px] md:text-xs font-black">My Identity Address</CardTitle>
                                </CardHeader>
                                <CardContent className="px-6 pb-4 md:pb-5">
                                    <div
                                        onClick={() => {
                                            navigator.clipboard.writeText(wallet.address);
                                            success("Address copied!");
                                        }}
                                        className="p-4 rounded-xl bg-secondary/30 font-mono text-xs break-all border border-primary/10 flex items-center justify-between gap-4 group cursor-pointer hover:border-primary/30 transition-all"
                                    >
                                        <span className="truncate text-foreground/80 group-hover:text-foreground">{wallet.address}</span>
                                        <div className="w-8 h-8 rounded-lg bg-background flex items-center justify-center shadow-sm group-hover:bg-primary group-hover:text-primary-foreground transition-all">
                                            <Copy className="w-4 h-4" />
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>

                            {/* Stats Card */}
                            <Card className="flex-1 border-primary/5 min-h-0 flex flex-col overflow-hidden bg-card/30 backdrop-blur-sm relative group">
                                <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full -mr-16 -mt-16 blur-3xl group-hover:bg-primary/10 transition-all" />
                                <CardHeader className="pb-3 pt-5 px-6 shrink-0 relative z-10">
                                    <CardTitle className="text-sm md:text-base font-bold flex items-center gap-2">
                                        <Activity className="w-4 h-4 text-primary" />
                                        Network Performance
                                    </CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-3 md:space-y-4 px-6 pb-4 md:pb-6 flex-1 flex flex-col justify-center min-h-0 relative z-10">
                                    <div className="flex justify-between items-center group/item">
                                        <div className="flex flex-col">
                                            <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Contribution</span>
                                            <span className="text-xs font-bold text-foreground/80">Network Validation</span>
                                        </div>
                                        <div className="text-right">
                                            <span className="text-base font-black text-foreground">0</span>
                                            <div className="text-[9px] text-muted-foreground uppercase font-black">Tasks</div>
                                        </div>
                                    </div>

                                    <div className="h-px bg-gradient-to-r from-border/0 via-border/50 to-border/0" />

                                    <div className="flex justify-between items-center group/item">
                                        <div className="flex flex-col">
                                            <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Security State</span>
                                            <span className="text-xs font-bold text-foreground/80">Node Validation</span>
                                        </div>
                                        <div className="flex flex-col items-end">
                                            <span className="text-xs font-black text-green-500 flex items-center gap-1.5">
                                                <div className="w-1.5 h-1.5 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse" />
                                                Active
                                            </span>
                                            <div className="text-[9px] text-muted-foreground uppercase font-black tracking-tighter">Verified</div>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>
                        </div>
                    </div>
                )}

                {/* Activity Detail Modal */}
                {selectedActivity && (
                    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6 bg-background/80 backdrop-blur-sm animate-in fade-in duration-200">
                        <div className="w-full max-w-lg bg-card border border-primary/20 rounded-[2rem] shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 relative">
                            <Button
                                variant="ghost"
                                size="icon"
                                className="absolute top-4 right-4 rounded-full h-10 w-10 hover:bg-muted"
                                onClick={() => setSelectedActivity(null)}
                            >
                                <X className="w-5 h-5 text-muted-foreground" />
                            </Button>

                            <div className="p-8">
                                <div className="flex flex-col items-center text-center mb-8">
                                    <div className="w-20 h-20 rounded-3xl bg-primary/10 flex items-center justify-center text-primary mb-4 border border-primary/10 shadow-inner">
                                        <Pickaxe className="w-10 h-10" />
                                    </div>
                                    <h2 className="text-2xl font-black tracking-tight text-foreground">Activity Detail</h2>
                                    <Badge variant="outline" className="mt-2 bg-green-500/10 text-green-500 border-green-500/20 font-bold px-4 py-1 rounded-full uppercase tracking-widest text-[10px]">
                                        Confirmed
                                    </Badge>
                                </div>

                                <div className="space-y-6">
                                    <div className="flex items-center justify-between p-5 bg-secondary/30 rounded-3xl border border-primary/5">
                                        <div className="flex items-center gap-3">
                                            <div className="p-3 bg-primary/20 rounded-2xl text-primary">
                                                <ArrowDownLeft className="w-6 h-6" />
                                            </div>
                                            <div className="flex flex-col">
                                                <span className="text-xs font-bold text-muted-foreground uppercase tracking-widest">Amount Reward</span>
                                                <span className="text-2xl font-black text-foreground">+{formatNumber(selectedActivity.amount)} <span className="text-primary/70">AGT</span></span>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-1 gap-4">
                                        <div className="p-4 rounded-2xl border border-primary/5 bg-background/50 space-y-1">
                                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                                                <Info className="w-3 h-3" /> Event Type
                                            </p>
                                            <p className="text-sm font-bold text-foreground">Block Mining Reward</p>
                                        </div>

                                        <div className="p-4 rounded-2xl border border-primary/5 bg-background/50 space-y-1">
                                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                                                <Clock className="w-3 h-3" /> Timestamp
                                            </p>
                                            <p className="text-sm font-bold text-foreground">{new Date(selectedActivity.timestamp * 1000).toLocaleString()}</p>
                                        </div>

                                        <div className="p-4 rounded-2xl border border-primary/5 bg-background/50 space-y-1">
                                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                                                <Database className="w-3 h-3" /> Block Reference
                                            </p>
                                            <div className="flex items-center justify-between">
                                                <p className="text-sm font-black text-primary">Block #{formatNumber(selectedActivity.blockIndex, false)}</p>
                                                <Button variant="ghost" size="sm" className="h-6 gap-1.5 text-[10px] font-bold text-muted-foreground hover:text-primary" onClick={() => navigate('/network')}>
                                                    <ExternalLink className="w-3 h-3" /> View Block
                                                </Button>
                                            </div>
                                        </div>

                                        <div className="p-4 rounded-2xl border border-primary/5 bg-background/50 space-y-1">
                                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                                                <ShieldCheck className="w-3 h-3" /> Transaction ID
                                            </p>
                                            <p className="text-[11px] font-mono text-muted-foreground break-all bg-muted/30 p-2 rounded-lg border border-primary/5">{selectedActivity.id}</p>
                                        </div>
                                    </div>

                                    <Button className="w-full h-14 rounded-2xl font-bold text-base shadow-xl shadow-primary/20" onClick={() => setSelectedActivity(null)}>
                                        Dismiss
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            <SwitchWalletModal
                isOpen={isSwitchModalOpen}
                onClose={() => setIsSwitchModalOpen(false)}
                onConfirm={async () => {
                    setIsSwitchModalOpen(false);
                    await logout();
                    info("Logged out of wallet.");
                }}
                currentAddress={wallet?.address || ""}
            />
        </PageTransition>
    );
}
