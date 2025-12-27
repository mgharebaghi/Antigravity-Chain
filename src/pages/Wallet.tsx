import { cn } from "../lib/utils";
import { Button } from "../components/ui/button";
import { Wallet as WalletIcon, Send, RefreshCw, Copy, Plus, Key, ShieldCheck, History as HistoryIcon, Pickaxe, ArrowRight, UserCircle, Download } from "lucide-react";
import { useApp, WalletExport, Block, Transaction } from "../context/AppContext";
import { useNavigate } from "react-router-dom";
import { useToast } from "../context/ToastContext";
import { useState } from "react";
import { formatNumber, ONE_AGT } from "../utils/format";
import SwitchWalletModal from "../components/SwitchWalletModal";
import { Badge } from "../components/ui/badge";

export default function Wallet() {
    const { wallet, createWallet: createWalletFn, importWallet: importWalletFn, loading, recentBlocks, refreshWallet, refreshBlockHeight } = useApp();
    const navigate = useNavigate();
    const { success, error } = useToast();

    const { logout } = useApp();
    const [createdWallet, setCreatedWallet] = useState<WalletExport | null>(null);
    const [importKey, setImportKey] = useState("");
    const [isImporting, setIsImporting] = useState(false);
    const [refreshing, setRefreshing] = useState(false);
    const [isSwitchModalOpen, setIsSwitchModalOpen] = useState(false);
    const [showKey, setShowKey] = useState(false);

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
            success("Wallet created successfully");
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

    const allRewards = recentBlocks.flatMap((b: Block) => {
        const explicitRewards = b.transactions
            .filter((tx: Transaction) => tx.receiver === wallet?.address && tx.sender === "SYSTEM")
            .map((tx: Transaction) => ({ ...tx, blockIndex: b.index }));

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

    const [currentPage, setCurrentPage] = useState(1);
    const ITEMS_PER_PAGE = 5;
    const totalPages = Math.ceil(allRewards.length / ITEMS_PER_PAGE);
    const paginatedRewards = allRewards.slice((currentPage - 1) * ITEMS_PER_PAGE, currentPage * ITEMS_PER_PAGE);

    const handlePageChange = (newPage: number) => {
        if (newPage >= 1 && newPage <= totalPages) {
            setCurrentPage(newPage);
        }
    };

    if (!wallet) {
        return (
            <div className="flex flex-col h-full bg-background relative overflow-hidden">
                {/* Background Blobs */}
                <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-primary/20 rounded-full blur-[120px] pointer-events-none -mr-40 -mt-40" />
                <div className="absolute bottom-0 left-0 w-[400px] h-[400px] bg-purple-500/10 rounded-full blur-[100px] pointer-events-none -ml-20 -mb-20" />

                <div className="flex-1 flex flex-col items-center justify-center relative p-6">
                    <div className="glass-card max-w-lg w-full p-8 md:p-12 rounded-[2.5rem] shadow-2xl border-white/20 dark:border-white/5 relative overflow-hidden">

                        {!isImporting && !createdWallet && (
                            <div className="flex flex-col items-center text-center gap-8">
                                <div className="w-20 h-20 rounded-3xl bg-primary/10 flex items-center justify-center text-primary shadow-inner mb-2">
                                    <WalletIcon className="w-10 h-10" />
                                </div>
                                <div className="space-y-2">
                                    <h1 className="text-3xl font-black tracking-tight">Connect Wallet</h1>
                                    <p className="text-muted-foreground">Initialize your identity to start interacting with the Centichain Chain.</p>
                                </div>

                                <div className="grid grid-cols-1 gap-4 w-full">
                                    <Button onClick={handleCreateWallet} size="lg" className="h-16 rounded-2xl text-base font-bold shadow-lg shadow-primary/25 hover:shadow-primary/40 transition-all hover:scale-[1.02]">
                                        <Plus className="w-5 h-5 mr-3" /> Create New Identity
                                    </Button>
                                    <Button variant="outline" onClick={() => setIsImporting(true)} size="lg" className="h-16 rounded-2xl text-base font-medium border-2 hover:bg-secondary/50">
                                        <Download className="w-5 h-5 mr-3" /> Import Existing Key
                                    </Button>
                                </div>
                            </div>
                        )}

                        {isImporting && (
                            <div className="space-y-6 animate-in slide-in-from-right-10 fade-in duration-300">
                                <div className="text-center space-y-2">
                                    <div className="mx-auto w-12 h-12 bg-secondary rounded-xl flex items-center justify-center text-muted-foreground mb-4">
                                        <Key className="w-6 h-6" />
                                    </div>
                                    <h2 className="text-2xl font-bold">Import Key</h2>
                                    <p className="text-sm text-muted-foreground">Paste your Private Key or Mnemonic phrase below.</p>
                                </div>

                                <textarea
                                    value={importKey}
                                    onChange={(e) => setImportKey(e.target.value)}
                                    placeholder="e.g. 5K3..."
                                    className="w-full h-32 p-4 rounded-xl border border-border bg-secondary/30 focus:bg-background focus:ring-2 focus:ring-primary/20 focus:border-primary/50 transition-all font-mono text-sm resize-none"
                                    autoFocus
                                />

                                <div className="flex gap-4">
                                    <Button variant="ghost" onClick={() => setIsImporting(false)} className="flex-1 rounded-xl h-12">Cancel</Button>
                                    <Button onClick={handleImportSubmit} disabled={!importKey || loading} className="flex-1 rounded-xl h-12 font-bold shadow-lg shadow-primary/20">
                                        {loading ? "Importing..." : "Recover Wallet"}
                                    </Button>
                                </div>
                            </div>
                        )}

                        {createdWallet && (
                            <div className="space-y-6 animate-in zoom-in-95 duration-300">
                                <div className="flex items-center gap-4 p-4 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-600 dark:text-emerald-400">
                                    <div className="p-2 bg-emerald-500/20 rounded-full">
                                        <ShieldCheck className="w-6 h-6" />
                                    </div>
                                    <div>
                                        <h3 className="font-bold text-lg">Identity Created</h3>
                                        <p className="text-xs opacity-80 font-medium">Save these credentials offline immediately.</p>
                                    </div>
                                </div>

                                <div className="space-y-4">
                                    <div className="space-y-2">
                                        <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground ml-1">Private Key</label>
                                        <div className="relative group">
                                            <div className="absolute inset-0 bg-secondary rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                                            <div className="relative p-3 bg-secondary/50 border border-border rounded-xl font-mono text-xs break-all pr-12">
                                                {createdWallet.private_key}
                                                <Button size="icon" variant="ghost" className="absolute right-2 top-1/2 -translate-y-1/2 h-8 w-8 hover:bg-background" onClick={() => {
                                                    navigator.clipboard.writeText(createdWallet.private_key);
                                                    success("Copied Private Key");
                                                }}>
                                                    <Copy className="w-4 h-4" />
                                                </Button>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground ml-1">Mnemonic Phrase</label>
                                        <div className="relative p-3 bg-secondary/50 border border-border rounded-xl font-mono text-xs break-words pr-12 leading-relaxed">
                                            {createdWallet.mnemonic}
                                            <Button size="icon" variant="ghost" className="absolute right-2 top-2 h-8 w-8 hover:bg-background" onClick={() => {
                                                navigator.clipboard.writeText(createdWallet.mnemonic);
                                                success("Copied Phrase");
                                            }}>
                                                <Copy className="w-4 h-4" />
                                            </Button>
                                        </div>
                                    </div>
                                </div>

                                <Button className="w-full h-14 rounded-2xl text-base font-bold mt-4" onClick={() => setCreatedWallet(null)}>
                                    I Have Saved My Keys
                                </Button>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-8 pb-8">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div>
                    <h1 className="text-4xl font-extrabold tracking-tight">Wallet</h1>
                    <p className="text-muted-foreground mt-1 text-lg">Overview of your holdings and history.</p>
                </div>
                <Button variant="secondary" onClick={() => setIsSwitchModalOpen(true)} className="gap-2 rounded-full px-5 hover:bg-secondary/80">
                    <UserCircle className="w-4 h-4" /> Account
                </Button>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">

                {/* Main Asset Card */}
                <div className="lg:col-span-2 relative group">
                    <div className="absolute inset-0 bg-gradient-to-r from-primary to-purple-600 rounded-[2.5rem] opacity-90 blur-xl group-hover:blur-2xl transition-all duration-500" />
                    <div className="relative h-full bg-gradient-to-br from-gray-900 to-black rounded-[2.5rem] p-8 sm:p-10 flex flex-col justify-between text-white shadow-2xl overflow-hidden border border-white/10">
                        {/* Decorative Patterns */}
                        <div className="absolute top-0 right-0 w-96 h-96 bg-white/5 rounded-full blur-3xl -mr-32 -mt-32 pointer-events-none" />

                        <div className="relative z-10 flex flex-col gap-8">
                            <div className="flex items-center justify-between">
                                <Badge variant="outline" className="border-white/20 text-white bg-white/5 backdrop-blur-md px-3 py-1 font-normal tracking-wide">
                                    Centichain Mainnet
                                </Badge>
                                <Pickaxe className="w-6 h-6 text-white/50" />
                            </div>

                            <div>
                                <p className="text-white/60 font-medium mb-2">Total Balance</p>
                                <h2 className="text-5xl sm:text-7xl font-bold tracking-tighter flex items-baseline gap-2">
                                    {formatNumber(wallet.balance)}
                                    <span className="text-2xl sm:text-3xl font-medium text-white/40">AGT</span>
                                </h2>
                            </div>

                            <div className="flex gap-4 mt-4">
                                <Button
                                    className="h-14 px-8 rounded-2xl bg-white text-black hover:bg-white/90 font-bold text-base shadow-lg shadow-white/10 hover:scale-105 transition-transform"
                                    onClick={() => navigate('/transactions')}
                                >
                                    <Send className="w-5 h-5 mr-2" /> Send
                                </Button>
                                <Button
                                    className="h-14 px-8 rounded-2xl bg-white/10 text-white hover:bg-white/20 font-bold text-base border border-white/10 backdrop-blur-md"
                                    onClick={handleRefresh}
                                    disabled={refreshing}
                                >
                                    <RefreshCw className={cn("w-5 h-5 mr-2", refreshing && "animate-spin")} /> Receive
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Details Side Card */}
                <div className="glass-card rounded-[2.5rem] p-8 flex flex-col justify-center gap-6 border border-white/20 dark:border-white/5 shadow-xl bg-gradient-to-b from-white/40 to-white/10 dark:from-white/5 dark:to-transparent">
                    <div>
                        <h3 className="text-lg font-bold mb-4 flex items-center gap-2">
                            <ShieldCheck className="w-5 h-5 text-primary" /> Credentials
                        </h3>

                        <div className="space-y-5">
                            <div className="space-y-2">
                                <label className="text-xs font-bold text-muted-foreground uppercase tracking-wider pl-1">Public Address</label>
                                <div className="p-3 rounded-2xl bg-secondary/30 border border-border/50 flex gap-2 items-center group cursor-pointer hover:bg-secondary/50 transition-colors" onClick={() => {
                                    navigator.clipboard.writeText(wallet.address);
                                    success("Copied Address");
                                }}>
                                    <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-blue-500 to-indigo-500 shrink-0" />
                                    <div className="flex-1 min-w-0">
                                        <p className="text-xs font-mono font-medium truncate text-foreground/80 group-hover:text-primary transition-colors">
                                            {wallet.address}
                                        </p>
                                    </div>
                                    <Copy className="w-4 h-4 text-muted-foreground" />
                                </div>
                            </div>

                            <div className="space-y-2">
                                <div className="flex items-center justify-between pl-1">
                                    <label className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Private Key</label>
                                    <button onClick={() => setShowKey(!showKey)} className="text-xs text-primary hover:underline font-medium">
                                        {showKey ? "Hide" : "Reveal"}
                                    </button>
                                </div>
                                <div className="p-4 rounded-2xl bg-black/5 dark:bg-black/20 border border-border/50 font-mono text-[10px] break-all relative min-h-[3rem] flex items-center">
                                    {showKey ? (
                                        <span>{wallet.private_key}</span>
                                    ) : (
                                        <div className="flex items-center gap-1 text-muted-foreground/50 w-full justify-center">
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                            <div className="w-2 h-2 rounded-full bg-current" />
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Mining Rewards / Transactions */}
            <div className="glass-card rounded-[2.5rem] p-8 border border-white/20 dark:border-white/5 shadow-lg">
                <div className="flex items-center justify-between mb-8">
                    <h3 className="text-xl font-bold flex items-center gap-3">
                        <HistoryIcon className="w-6 h-6 text-primary" /> Recent Activity
                    </h3>
                    <div className="flex items-center gap-3">
                        {allRewards.length > ITEMS_PER_PAGE && (
                            <div className="flex items-center gap-2 mr-2">
                                <Button
                                    variant="outline"
                                    size="icon"
                                    className="h-8 w-8 rounded-lg border-border/50"
                                    onClick={() => handlePageChange(currentPage - 1)}
                                    disabled={currentPage === 1}
                                >
                                    <ArrowRight className="w-4 h-4 rotate-180" />
                                </Button>
                                <span className="text-xs font-bold text-muted-foreground min-w-[3rem] text-center">
                                    {currentPage} / {totalPages}
                                </span>
                                <Button
                                    variant="outline"
                                    size="icon"
                                    className="h-8 w-8 rounded-lg border-border/50"
                                    onClick={() => handlePageChange(currentPage + 1)}
                                    disabled={currentPage === totalPages}
                                >
                                    <ArrowRight className="w-4 h-4" />
                                </Button>
                            </div>
                        )}
                        <Badge variant="secondary" className="px-3 py-1 text-xs">
                            {allRewards.length} Events
                        </Badge>
                    </div>
                </div>

                <div className="space-y-3">
                    {allRewards.length === 0 ? (
                        <div className="py-16 text-center text-muted-foreground bg-secondary/10 rounded-3xl border border-dashed border-border/60">
                            <div className="w-16 h-16 bg-secondary rounded-full flex items-center justify-center mx-auto mb-4 opacity-50">
                                <HistoryIcon className="w-8 h-8" />
                            </div>
                            <p className="font-medium">No activity recorded yet.</p>
                            <p className="text-sm mt-1 opacity-70">Start mining or transactions to generate history.</p>
                        </div>
                    ) : (
                        paginatedRewards.map((tx: any) => (
                            <div
                                key={tx.id}
                                className="group p-5 rounded-3xl bg-secondary/10 hover:bg-secondary/30 border border-transparent hover:border-primary/10 transition-all flex items-center justify-between"
                            >
                                <div className="flex items-center gap-4">
                                    <div className="w-12 h-12 rounded-2xl bg-emerald-500/10 flex items-center justify-center text-emerald-500 group-hover:scale-110 transition-transform">
                                        <Pickaxe className="w-6 h-6" />
                                    </div>
                                    <div>
                                        <p className="font-bold text-foreground">Mining Reward</p>
                                        <div className="flex items-center gap-2 mt-0.5">
                                            <Badge variant="outline" className="text-[9px] h-4 px-1.5 font-normal border-border/50">
                                                BLK #{formatNumber(tx.blockIndex, false)}
                                            </Badge>
                                            <span className="text-xs text-muted-foreground">{new Date(tx.timestamp * 1000).toLocaleDateString()}</span>
                                        </div>
                                    </div>
                                </div>
                                <div className="text-right">
                                    <p className="font-bold text-lg text-emerald-600 dark:text-emerald-400">+{formatNumber(tx.amount)}</p>
                                    <span className="text-[10px] uppercase font-bold text-emerald-500/50 tracking-wider">AGT</span>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>

            <SwitchWalletModal
                isOpen={isSwitchModalOpen}
                onClose={() => setIsSwitchModalOpen(false)}
                onConfirm={async () => {
                    setIsSwitchModalOpen(false);
                    await logout();
                    success("Identity disconnected securey.");
                }}
                currentAddress={wallet?.address || ""}
            />
        </div>
    );
}
