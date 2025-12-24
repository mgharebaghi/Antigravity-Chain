import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Badge } from "../components/ui/badge.tsx";
import PageTransition from "../components/PageTransition";
import { useApp } from "../context/AppContext";
import { formatNumber, calculateFee } from "../utils/format";
import { invoke } from "@tauri-apps/api/core";
import {
    Box,
    ChevronRight,
    Search,
    User,
    ArrowRight,
    FileText,
    Clock,
    Shield,
    Layers,
    X,
    Activity,
    Zap,
    ChevronLeft,
    Cpu,
    Fingerprint,
    Globe,
    Binary,
    Scale,
    Copy,
    Check
} from "lucide-react";
import { Button } from "../components/ui/button";
import { cn } from "../lib/utils";

const PAGE_SIZE = 10;

export default function Explorer() {
    const { height, totalBlocks } = useApp();
    const [blocks, setBlocks] = useState<any[]>([]);
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedBlock, setSelectedBlock] = useState<any>(null);
    const [selectedTx, setSelectedTx] = useState<any>(null);
    const [page, setPage] = useState(1);
    const [isLoading, setIsLoading] = useState(false);

    useEffect(() => {
        fetchBlocks();
    }, [page, totalBlocks]);

    const fetchBlocks = async () => {
        setIsLoading(true);
        try {
            const data = await invoke<any[]>("get_blocks_paginated", { page, limit: PAGE_SIZE });
            setBlocks(data);
        } catch (e) {
            console.error("Failed to fetch blocks", e);
        } finally {
            setIsLoading(false);
        }
    };

    const searchBlock = async () => {
        const term = searchTerm.trim();
        if (!term) {
            setPage(1);
            fetchBlocks();
            return;
        }

        setIsLoading(true);
        try {
            // 1. Try direct index lookup if it's a number
            const idx = parseInt(term);
            if (!isNaN(idx) && term.length < 10) {
                const block = await invoke<any>("get_block", { index: idx });
                if (block) {
                    setBlocks([block]);
                    setIsLoading(false);
                    return;
                }
            }

            // 2. Try Hash lookup (64 chars)
            if (term.length === 64) {
                const block = await invoke<any>("get_block_by_hash", { hash: term });
                if (block) {
                    setBlocks([block]);
                    setIsLoading(false);
                    return;
                }
            }

            // 3. Try Transaction ID lookup (UUID is 36 chars)
            if (term.length === 36 || term === 'genesis' || term === 'reward') {
                const result = await invoke<any>("get_transaction", { id: term });
                if (result) {
                    // result is [transaction, block]
                    setBlocks([result[1]]);
                    setSelectedBlock(result[1]); // Open the block modal
                    setIsLoading(false);
                    return;
                }
            }

            // If nothing found, clear view or show empty
            setBlocks([]);
        } catch (e) {
            console.error("Search error:", e);
        } finally {
            setIsLoading(false);
        }
    };

    const totalPages = Math.ceil(totalBlocks / PAGE_SIZE) || 1;

    return (
        <PageTransition>
            <div className="flex flex-col gap-4 md:gap-6 container mx-auto p-4 sm:p-6 lg:max-w-7xl min-h-full pb-10">
                {/* Header Section */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 md:gap-3 shrink-0">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 md:p-2.5 bg-primary/10 rounded-xl border border-primary/20 shadow-inner">
                            <Box className="w-4 h-4 md:w-5 md:h-5 text-primary" />
                        </div>
                        <div>
                            <h1 className="text-lg md:text-xl font-black tracking-tight text-foreground">Block Explorer</h1>
                            <p className="text-[9px] md:text-[10px] text-muted-foreground flex items-center gap-2 uppercase tracking-widest opacity-60">
                                <Layers className="w-2 md:w-2.5 h-2 md:h-2.5" /> Ledger Visualization
                            </p>
                        </div>
                    </div>
                    <div className="relative w-full md:w-72 flex gap-2">
                        <div className="relative flex-1">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
                            <input
                                type="text"
                                placeholder="Search Ledger..."
                                className="w-full bg-muted/20 border border-primary/10 rounded-lg py-2 pl-9 pr-4 text-xs focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-mono"
                                value={searchTerm}
                                onChange={(e) => setSearchTerm(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && searchBlock()}
                            />
                        </div>
                        <Button variant="outline" size="icon" onClick={searchBlock} className="h-9 w-9 rounded-lg border-primary/10 bg-muted/20">
                            <ArrowRight className="w-3.5 h-3.5" />
                        </Button>
                    </div>
                </div>

                {/* Main Content */}
                <Card className="flex-1 border-primary/10 bg-card/20 backdrop-blur-xl shadow-2xl rounded-3xl md:rounded-[2.5rem] flex flex-col min-h-0">
                    <CardHeader className="py-4 md:py-5 px-4 md:px-8 border-b border-primary/5 shrink-0 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-muted/10">
                        <div className="flex items-center gap-4">
                            <CardTitle className="text-base md:text-lg font-black flex items-center gap-2">
                                <FileText className="w-5 h-5 text-primary" />
                                {searchTerm ? "Search Results" : `Latest Activity (Pg ${page})`}
                            </CardTitle>
                        </div>
                        <div className="flex items-center justify-between w-full sm:w-auto gap-4">
                            <div className="flex items-center bg-background/50 rounded-xl border border-primary/5 p-0.5 md:p-1">
                                <Button
                                    variant="ghost" size="icon" className="h-7 w-7 md:h-8 md:w-8 rounded-lg"
                                    disabled={page <= 1 || isLoading}
                                    onClick={() => setPage(p => p - 1)}
                                >
                                    <ChevronLeft className="h-3.5 w-3.5 md:h-4 md:w-4" />
                                </Button>
                                <span className="text-[9px] md:text-[10px] font-black px-2 md:px-3 tabular-nums opacity-60">
                                    {page} / {totalPages}
                                </span>
                                <Button
                                    variant="ghost" size="icon" className="h-7 w-7 md:h-8 md:w-8 rounded-lg"
                                    disabled={page >= totalPages || isLoading}
                                    onClick={() => setPage(p => p + 1)}
                                >
                                    <ChevronRight className="h-3.5 w-3.5 md:h-4 md:w-4" />
                                </Button>
                            </div>
                            <Badge className="bg-primary/20 text-primary border-primary/20 font-black px-3 md:px-4 py-1 md:py-1.5 rounded-full text-[9px] md:text-[10px] tracking-widest shrink-0">
                                HEIGHT: #{height}
                            </Badge>
                        </div>
                    </CardHeader>

                    <CardContent className="flex-1 min-h-0 overflow-y-auto p-0 scrollbar-thin scrollbar-thumb-primary/20 scrollbar-track-transparent">
                        <div className="w-full">
                            {/* Table Header */}
                            <div className="grid grid-cols-[60px_1fr_40px] md:grid-cols-[80px_1fr_1.5fr_60px_60px] lg:grid-cols-[80px_1fr_1.5fr_100px_80px] gap-2 md:gap-4 px-4 md:px-8 py-3 md:py-4 bg-muted/20 text-[9px] md:text-[10px] font-black uppercase text-muted-foreground tracking-widest border-b border-primary/5">
                                <span>Index</span>
                                <span>Hash</span>
                                <span className="hidden md:block">Author / Miner</span>
                                <span className="hidden lg:block">TXs</span>
                                <span className="text-right">Action</span>
                            </div>

                            {/* Table Body */}
                            <div className="divide-y divide-primary/5">
                                {isLoading ? (
                                    <div className="flex flex-col items-center justify-center py-20 animate-pulse">
                                        <div className="w-12 h-12 rounded-full border-4 border-primary/20 border-t-primary animate-spin mb-4" />
                                        <p className="font-black text-[10px] uppercase tracking-widest text-primary">Fetching Ledger Data...</p>
                                    </div>
                                ) : blocks.length > 0 ? (
                                    blocks.map((block: any) => (
                                        <div
                                            key={block.hash}
                                            className="grid grid-cols-[60px_1fr_40px] md:grid-cols-[80px_1fr_1.5fr_60px_60px] lg:grid-cols-[80px_1fr_1.5fr_100px_80px] gap-2 md:gap-4 px-4 md:px-8 py-2 md:py-2.5 items-center hover:bg-primary/5 transition-all group cursor-pointer border-b border-primary/5 last:border-0"
                                            onClick={() => setSelectedBlock(block)}
                                        >
                                            <div className="font-black text-primary text-[11px] md:text-sm">#{block.index}</div>
                                            <div className="font-mono text-[10px] md:text-xs text-muted-foreground truncate" title={block.hash}>
                                                {block.hash.substring(0, 8)}<span className="hidden md:inline">{block.hash.substring(8, 16)}</span>...
                                            </div>
                                            <div className="hidden md:flex items-center gap-2 min-w-0">
                                                <div className="w-5 h-5 md:w-6 md:h-6 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                                                    <User className="w-3 h-3 text-primary" />
                                                </div>
                                                <span className="text-[10px] md:text-xs font-bold truncate text-foreground/80" title={block.author}>
                                                    {block.author}
                                                </span>
                                            </div>
                                            <div className="hidden lg:flex items-center gap-2">
                                                <Badge variant="secondary" className="bg-secondary/50 text-[10px] font-black">
                                                    {block.transactions.length}
                                                </Badge>
                                            </div>
                                            <div className="flex justify-end pr-1">
                                                <div className="w-7 h-7 md:w-8 md:h-8 rounded-full bg-primary/10 flex items-center justify-center text-primary group-hover:scale-110 transition-transform">
                                                    <ChevronRight className="w-3.5 h-3.5 md:w-4 md:h-4" />
                                                </div>
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <div className="flex flex-col items-center justify-center py-20 text-muted-foreground gap-4">
                                        <Layers className="w-12 h-12 opacity-20" />
                                        <p className="font-bold">No blocks found matching "{searchTerm}"</p>
                                    </div>
                                )}
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Block Detail Modal */}
                {selectedBlock && (
                    <div className="fixed inset-0 z-50 flex items-center justify-center p-2 sm:p-4 md:p-6 backdrop-blur-md bg-background/40 animate-in fade-in duration-300">
                        <div
                            className="bg-card border border-primary/20 shadow-2xl rounded-3xl md:rounded-[3rem] w-full max-w-4xl max-h-[95vh] md:max-h-[90vh] overflow-hidden flex flex-col relative animate-in zoom-in-95 duration-300"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <button
                                onClick={() => setSelectedBlock(null)}
                                className="absolute top-4 right-4 md:top-6 md:right-6 p-2 rounded-full bg-muted/50 text-muted-foreground hover:bg-primary/10 hover:text-primary transition-all z-10"
                            >
                                <X className="w-4 h-4 md:w-5 md:h-5" />
                            </button>

                            <div className="p-5 sm:p-8 md:p-10 overflow-y-auto scrollbar-thin scrollbar-thumb-primary/20">
                                <div className="flex flex-col gap-6 md:gap-10">
                                    {/* Modal Header - Refined & Elegant */}
                                    <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 md:gap-6 border-b border-primary/10 pb-6 md:pb-8">
                                        <div className="space-y-2 md:space-y-3">
                                            <div className="flex items-center gap-3">
                                                <div className="p-2 md:p-2.5 bg-primary/10 rounded-xl border border-primary/20 backdrop-blur-xl">
                                                    <Box className="w-4 h-4 md:w-5 md:h-5 text-primary" />
                                                </div>
                                                <h2 className="text-xl md:text-3xl font-black tracking-tightest">Block <span className="text-primary">#{selectedBlock.index}</span></h2>
                                            </div>
                                            <div className="flex items-center gap-2 group/hash pl-1 overflow-hidden">
                                                <p className="text-[10px] md:text-[11px] text-muted-foreground/60 font-mono tracking-tight truncate">
                                                    {selectedBlock.hash}
                                                </p>
                                                <CopyButton text={selectedBlock.hash} />
                                            </div>
                                        </div>
                                        <div className="flex gap-2.5">
                                            <Badge className="bg-primary/5 text-primary border-primary/20 px-2.5 py-1 font-bold tracking-wider text-[9px] md:text-[10px]">
                                                {new Date(selectedBlock.timestamp * 1000).toLocaleDateString()}
                                            </Badge>
                                            <Badge variant="outline" className="border-primary/20 text-muted-foreground font-bold text-[9px] md:text-[10px]">
                                                v{selectedBlock.version || 1}
                                            </Badge>
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-8 md:gap-10 items-start">
                                        {/* Main Content Area */}
                                        <div className="space-y-8 md:space-y-10">
                                            {/* Primary Ledger Metadata */}
                                            <section className="space-y-5 md:space-y-6">
                                                <div className="flex items-center gap-2 px-1">
                                                    <Shield className="w-3.5 h-3.5 text-primary/60" />
                                                    <h3 className="text-[9px] md:text-xs font-black uppercase tracking-[0.3em] text-muted-foreground/40">Chain Integrity</h3>
                                                </div>
                                                <div className="grid grid-cols-1 gap-4">
                                                    <div className="p-5 md:p-6 rounded-2xl md:rounded-[2rem] bg-muted/10 border border-primary/5 space-y-4 md:space-y-5 hover:bg-muted/15 transition-colors">
                                                        <MetadataRow icon={<User className="w-3.5 h-3.5" />} label="Block Author" value={selectedBlock.author} mono copyable />
                                                        <MetadataRow icon={<ChevronRight className="w-3.5 h-3.5" />} label="Parent Hash" value={selectedBlock.previous_hash} mono copyable />
                                                    </div>
                                                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                                        <div className="p-5 md:p-6 rounded-2xl md:rounded-[2rem] bg-muted/10 border border-primary/5">
                                                            <MetadataRow icon={<Binary className="w-3.5 h-3.5" />} label="Merkle Root" value={selectedBlock.merkle_root || "N/A"} mono copyable />
                                                        </div>
                                                        <div className="p-5 md:p-6 rounded-2xl md:rounded-[2rem] bg-muted/10 border border-primary/5">
                                                            <MetadataRow icon={<Globe className="w-3.5 h-3.5" />} label="State Root" value={selectedBlock.state_root || "N/A"} mono copyable />
                                                        </div>
                                                    </div>
                                                </div>
                                            </section>

                                            {/* Transaction Feed - Elegant & Clean */}
                                            <section className="space-y-6">
                                                <div className="flex items-center justify-between px-1">
                                                    <div className="flex items-center gap-2">
                                                        <Activity className="w-4 h-4 text-primary/60" />
                                                        <h3 className="text-xs font-black uppercase tracking-[0.3em] text-muted-foreground/40">Payload Ledger</h3>
                                                    </div>
                                                    <span className="text-[10px] font-black text-primary/60 bg-primary/5 px-2 py-0.5 rounded-full border border-primary/10">
                                                        {selectedBlock.transactions.length} ITEMS
                                                    </span>
                                                </div>
                                                <div className="space-y-3">
                                                    {selectedBlock.transactions.length > 0 ? (
                                                        selectedBlock.transactions.map((tx: any) => (
                                                            <div
                                                                key={tx.id}
                                                                onClick={(e) => {
                                                                    e.stopPropagation();
                                                                    setSelectedTx(tx);
                                                                }}
                                                                className="p-5 rounded-3xl border border-primary/5 bg-muted/5 flex items-center justify-between group/tx hover:bg-muted/10 transition-all cursor-pointer hover:border-primary/20 hover:translate-x-1"
                                                            >
                                                                <div className="flex items-center gap-5 flex-1 min-w-0">
                                                                    <div className="p-2.5 rounded-2xl bg-primary/5 text-primary group-hover/tx:bg-primary/10 transition-colors shrink-0">
                                                                        <ArrowRight className="w-4 h-4" />
                                                                    </div>
                                                                    <div className="flex flex-col min-w-0 pr-4">
                                                                        <div className="flex items-center gap-2 mb-1">
                                                                            <span className="text-[10px] font-mono font-bold text-foreground/80 truncate max-w-[120px]">{tx.sender}</span>
                                                                            <span className="text-[8px] text-muted-foreground/40 font-black">TO</span>
                                                                            <span className="text-[10px] font-mono font-bold text-foreground/80 truncate max-w-[120px]">{tx.receiver}</span>
                                                                        </div>
                                                                        <span className="text-[9px] font-mono text-muted-foreground/40 truncate">{tx.id}</span>
                                                                    </div>
                                                                </div>
                                                                <div className="text-right shrink-0 flex items-center gap-3">
                                                                    <div className="text-lg font-black text-primary tracking-tighter">
                                                                        {formatNumber(tx.amount)} <span className="text-[10px] opacity-40">AGT</span>
                                                                    </div>
                                                                    <ChevronRight className="w-4 h-4 text-primary/20 group-hover/tx:text-primary/60 transition-colors" />
                                                                </div>
                                                            </div>
                                                        ))
                                                    ) : (
                                                        <div className="py-12 flex flex-col items-center justify-center bg-muted/5 rounded-[2rem] border border-dashed border-primary/10 opacity-40">
                                                            <FileText className="w-8 h-8 mb-3 text-primary/40" />
                                                            <p className="text-[10px] font-black uppercase tracking-widest leading-none">Vacuum Block (No TX)</p>
                                                        </div>
                                                    )}
                                                </div>
                                            </section>
                                        </div>

                                        {/* Sidebar - Precision & Economics */}
                                        <aside className="space-y-6">
                                            {/* Economic Dashboard Card */}
                                            <div className="p-6 md:p-8 rounded-[1.5rem] md:rounded-[2.5rem] bg-primary/5 border border-primary/10 backdrop-blur-xl relative overflow-hidden group/eco shadow-2xl shadow-primary/5">
                                                <div className="absolute top-0 right-0 p-8 opacity-5 group-hover/eco:opacity-10 transition-opacity">
                                                    <Zap className="w-16 md:w-24 h-16 md:h-24 text-primary" />
                                                </div>
                                                <h3 className="text-[10px] md:text-xs font-black uppercase tracking-[0.3em] text-primary mb-6 md:mb-8 flex items-center gap-2">
                                                    <Zap className="w-3.5 h-3.5" /> Block Economics
                                                </h3>
                                                <div className="space-y-6 md:space-y-8">
                                                    <div className="grid grid-cols-2 gap-4 md:gap-6">
                                                        <MetadataRow icon={<Box className="w-3.5 h-3.5" />} label="Emission" value={`${formatNumber(selectedBlock.block_reward || 0)}`} />
                                                        <MetadataRow icon={<Activity className="w-3.5 h-3.5" />} label="Fees" value={`${formatNumber(selectedBlock.total_fees || 0)}`} />
                                                    </div>
                                                    <div className="pt-6 md:pt-8 border-t border-primary/10">
                                                        <p className="text-[8px] md:text-[9px] font-black uppercase text-muted-foreground/40 tracking-[0.2em] mb-2">Total Block Value</p>
                                                        <div className="flex items-baseline gap-2 overflow-hidden">
                                                            <span className="text-2xl sm:text-3xl md:text-4xl font-black text-primary tracking-tightest leading-none">
                                                                {formatNumber(selectedBlock.total_reward || 0)}
                                                            </span>
                                                            <span className="text-xs md:text-sm font-black text-primary/40">AGT</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>

                                            {/* Node Mechanics Card */}
                                            <div className="p-6 md:p-8 rounded-[1.5rem] md:rounded-[2.5rem] bg-muted/20 border border-primary/5 backdrop-blur-md space-y-6 md:space-y-8">
                                                <h4 className="text-[9px] md:text-[10px] font-black uppercase tracking-[0.3em] text-muted-foreground/60 flex items-center gap-2">
                                                    <Cpu className="w-3.5 h-3.5" /> Node Mechanics
                                                </h4>
                                                <div className="grid grid-cols-1 gap-5 md:gap-6">
                                                    <div className="grid grid-cols-2 gap-4 md:gap-6">
                                                        <MetadataRow icon={<Scale className="w-3.5 h-3.5" />} label="Weight" value={`${((selectedBlock.size || 0) / 1024).toFixed(2)} KB`} />
                                                        <MetadataRow icon={<Clock className="w-3.5 h-3.5" />} label="Clock" value={new Date(selectedBlock.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} />
                                                    </div>
                                                    <MetadataRow icon={<Fingerprint className="w-3.5 h-3.5" />} label="Protocol Nonce" value={selectedBlock.nonce?.toString() || "0"} mono copyable />
                                                    <div className="grid grid-cols-2 gap-4 md:gap-6 items-center">
                                                        <MetadataRow icon={<Zap className="w-3.5 h-3.5" />} label="VDF Weight" value={selectedBlock.start_time_weight?.toFixed(2) || "1.00"} />
                                                        <MetadataRow icon={<Cpu className="w-3.5 h-3.5" />} label="Difficulty" value={formatNumber(selectedBlock.vdf_difficulty || 200000)} />
                                                    </div>
                                                </div>
                                            </div>
                                        </aside>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Transaction Detail Modal */}
                {selectedTx && (
                    <div className="fixed inset-0 z-[60] flex items-center justify-center p-2 sm:p-4 md:p-6 backdrop-blur-xl bg-background/60 animate-in fade-in duration-300">
                        <div
                            className="bg-card border border-primary/20 shadow-2xl rounded-3xl md:rounded-[3rem] w-full max-w-2xl max-h-[95vh] overflow-hidden flex flex-col relative animate-in slide-in-from-bottom-8 duration-500"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <button
                                onClick={() => setSelectedTx(null)}
                                className="absolute top-4 right-4 md:top-8 md:right-8 p-2 md:p-3 rounded-2xl bg-muted/20 hover:bg-primary/10 text-muted-foreground hover:text-primary transition-all z-10 group"
                            >
                                <X className="w-4 h-4 md:w-5 md:h-5 group-hover:rotate-90 transition-transform" />
                            </button>

                            <div className="p-6 md:p-10 space-y-8 md:space-y-10 overflow-y-auto">
                                {/* Header */}
                                <div className="flex items-center gap-4">
                                    <div className="p-3 md:p-4 bg-primary/10 rounded-2xl md:rounded-[1.5rem] border border-primary/20 shrink-0">
                                        <ArrowRight className="w-5 h-5 md:w-6 md:h-6 text-primary" />
                                    </div>
                                    <div>
                                        <h2 className="text-xl md:text-2xl font-black tracking-tight text-foreground">Transaction</h2>
                                        <p className="text-[8px] md:text-[10px] text-muted-foreground/60 uppercase tracking-[0.2em] font-black">Cryptographic Transfer Receipt</p>
                                    </div>
                                </div>

                                {/* Flow Visualization */}
                                <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-4 md:gap-6 items-center bg-muted/5 p-6 md:p-8 rounded-3xl md:rounded-[2.5rem] border border-primary/5">
                                    <div className="space-y-2 md:space-y-3 min-w-0">
                                        <span className="text-[8px] md:text-[9px] font-black text-muted-foreground/40 uppercase tracking-widest pl-1">Origin Node</span>
                                        <div className="p-4 md:p-5 rounded-xl md:rounded-2xl bg-background/40 border border-primary/5 flex items-center gap-3 min-w-0">
                                            <div className="p-2 rounded-lg bg-primary/5 shrink-0">
                                                <User className="w-3.5 h-3.5 text-primary/60" />
                                            </div>
                                            <span className="text-xs font-mono font-bold truncate" title={selectedTx.sender}>{selectedTx.sender}</span>
                                        </div>
                                    </div>
                                    <div className="flex md:flex-col items-center justify-center gap-2 py-2 md:py-0">
                                        <div className="flex-1 md:flex-none w-full md:w-px md:h-8 bg-primary/20" />
                                        <ArrowRight className="w-4 h-4 text-primary animate-pulse rotate-90 md:rotate-0" />
                                        <div className="flex-1 md:flex-none w-full md:w-px md:h-8 bg-primary/20" />
                                    </div>
                                    <div className="space-y-2 md:space-y-3 text-left min-w-0">
                                        <span className="text-[8px] md:text-[9px] font-black text-muted-foreground/40 uppercase tracking-widest pl-1">Destination Node</span>
                                        <div className="p-4 md:p-5 rounded-xl md:rounded-2xl bg-background/40 border border-primary/5 flex items-center gap-3 min-w-0">
                                            <div className="p-2 rounded-lg bg-primary/5 shrink-0">
                                                <User className="w-3.5 h-3.5 text-primary/60" />
                                            </div>
                                            <span className="text-xs font-mono font-bold truncate" title={selectedTx.receiver}>{selectedTx.receiver}</span>
                                        </div>
                                    </div>
                                </div>

                                {/* Core Metadata */}
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-8">
                                    <div className="space-y-5 md:space-y-6">
                                        <MetadataRow
                                            icon={<Fingerprint className="w-3.5 h-3.5" />}
                                            label="Transaction Signature"
                                            value={selectedTx.id}
                                            mono
                                            copyable
                                        />
                                        <MetadataRow
                                            icon={<Activity className="w-3.5 h-3.5" />}
                                            label="Network Status"
                                            value="Confirmed on Ledger"
                                        />
                                    </div>
                                    <div className="p-6 md:p-8 rounded-[1.5rem] md:rounded-[2rem] bg-primary/5 border border-primary/10 flex flex-col justify-center">
                                        <span className="text-[8px] md:text-[9px] font-black text-primary/40 uppercase tracking-[0.3em] mb-3">Transfer Payload</span>
                                        <div className="flex items-baseline gap-2 overflow-hidden">
                                            <span className="text-3xl md:text-5xl font-black text-primary tracking-tightest leading-none">
                                                {formatNumber(selectedTx.amount)}
                                            </span>
                                            <span className="text-sm md:text-lg font-black text-primary/40">AGT</span>
                                        </div>
                                        <div className="mt-3 md:mt-4 flex items-center gap-2 text-[9px] md:text-[10px] font-bold text-muted-foreground/60">
                                            <Zap className="w-3 h-3" />
                                            <span>Fee: {selectedTx.sender === 'SYSTEM' ? "0" : formatNumber(calculateFee(selectedTx.amount))} AGT</span>
                                        </div>
                                    </div>
                                </div>

                                {/* Footer Security Note */}
                                <div className="p-6 rounded-2xl bg-muted/5 border border-dashed border-primary/5 flex items-center gap-4">
                                    <Shield className="w-5 h-5 text-primary/40" />
                                    <p className="text-[9px] font-bold text-muted-foreground/40 leading-relaxed uppercase tracking-wider">
                                        This transaction is cryptographically secured and immutable on the Antigravity Chain.
                                        Verification is performed by the decentralized network clock.
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </PageTransition>
    );
}

function MetadataRow({ icon, label, value, mono = false, copyable = false }: { icon: any, label: string, value: string, mono?: boolean, copyable?: boolean }) {
    return (
        <div className="flex flex-col gap-1.5 min-w-0">
            <div className="flex items-center gap-1.5 text-[9px] font-black uppercase text-muted-foreground/60 tracking-tighter">
                {icon}
                <span>{label}</span>
            </div>
            <div className="flex items-center gap-2 group/meta">
                <div className={cn(
                    "text-xs font-bold text-foreground transition-colors",
                    mono ? "font-mono break-all leading-relaxed" : "truncate",
                    copyable && "group-hover/meta:text-primary"
                )} title={value}>
                    {value}
                </div>
                {copyable && <CopyButton text={value} />}
            </div>
        </div>
    );
}

function CopyButton({ text }: { text: string }) {
    const [copied, setCopied] = useState(false);

    const handleCopy = () => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <button
            onClick={handleCopy}
            className="p-1.5 rounded-md hover:bg-primary/10 text-muted-foreground/50 hover:text-primary transition-all shrink-0"
        >
            {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
        </button>
    );
}
