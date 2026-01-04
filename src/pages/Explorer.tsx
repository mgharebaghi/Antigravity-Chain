import { useState, useEffect } from 'react';
import { Badge } from "../components/ui/badge";
import { useApp, Transaction } from "../context/AppContext";
// import { formatNumber } from "../utils/format";
import { invoke } from "@tauri-apps/api/core";
import {
    Search,
    User,

    ChevronLeft,
    ChevronRight,
    Layers,
    Hash,
    Clock,
    Box
} from "lucide-react";
import { Button } from "../components/ui/button";
// import { cn } from "../lib/utils";
import BlockDetailsModal from '../components/modals/BlockDetailsModal';
import TxDetailsModal from '../components/modals/TxDetailsModal';

const PAGE_SIZE = 10;

export default function Explorer() {
    const { totalBlocks } = useApp();
    const [blocks, setBlocks] = useState<any[]>([]);
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedBlock, setSelectedBlock] = useState<any>(null);
    const [viewingTx, setViewingTx] = useState<Transaction | null>(null);
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
            const idx = parseInt(term);
            if (!isNaN(idx) && term.length < 10) {
                const block = await invoke<any>("get_block", { index: idx });
                if (block) {
                    setBlocks([block]);
                    setIsLoading(false);
                    return;
                }
            }

            if (term.length === 64) {
                const block = await invoke<any>("get_block_by_hash", { hash: term });
                if (block) {
                    setBlocks([block]);
                    setIsLoading(false);
                    return;
                }
            }

            // Transaction ID search
            if (term.length === 36 || term === 'genesis' || term === 'reward') {
                const result = await invoke<any>("get_transaction", { id: term });
                if (result) {
                    // result is a tuple? or simpler
                    // Assuming result is [tx, block] based on previous context, but let's check
                    // If backend returns (Transaction, Block), result[0] is tx, result[1] is block
                    if (Array.isArray(result) && result.length === 2) {
                        setBlocks([result[1]]);
                        setSelectedBlock(result[1]);
                        setViewingTx(result[0]); // Auto-open tx modal
                    } else {
                        // Fallback just in case
                        console.warn("Unexpected transaction search result format", result);
                    }
                    setIsLoading(false);
                    return;
                }
            }

            setBlocks([]);
        } catch (e) {
            console.error("Search error:", e);
        } finally {
            setIsLoading(false);
        }
    };

    const totalPages = Math.ceil(totalBlocks / PAGE_SIZE) || 1;

    return (
        <div className="flex flex-col gap-8 h-full">

            {/* Hero Search Section */}
            <div className="flex flex-col items-center justify-center py-10 text-center relative shrink-0">
                <div className="absolute inset-0 bg-primary/5 rounded-3xl blur-3xl -z-10" />
                <h1 className="text-4xl font-extrabold tracking-tight mb-4">Blockchain Explorer</h1>
                <p className="text-muted-foreground max-w-lg mb-8">
                    Search through blocks, transactions, and addresses to verify the integrity of the ledger.
                </p>

                <div className="relative w-full max-w-2xl group">
                    <div className="absolute inset-0 bg-primary/20 rounded-full blur-md opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
                    <div className="relative flex items-center bg-background/80 backdrop-blur-xl border border-primary/20 rounded-full shadow-lg p-1.5 focus-within:ring-4 focus-within:ring-primary/10 transition-all">
                        <div className="pl-4 pr-3 text-muted-foreground">
                            <Search className="w-5 h-5" />
                        </div>
                        <input
                            type="text"
                            placeholder="Search by Index, Hash, or Transaction ID..."
                            className="flex-1 bg-transparent border-none text-sm h-10 focus:outline-none placeholder:text-muted-foreground/50"
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && searchBlock()}
                        />
                        <Button onClick={searchBlock} size="default" className="rounded-full px-6 shadow-md">
                            Search
                        </Button>
                    </div>
                </div>
            </div>

            {/* Blocks Table Card */}
            <div className="flex-1 min-h-0 glass-card rounded-3xl border border-white/20 dark:border-white/5 flex flex-col overflow-hidden shadow-xl">
                <div className="p-6 border-b border-border/50 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-secondary/20">
                    <h3 className="font-bold text-lg flex items-center gap-2">
                        <Box className="w-5 h-5 text-primary" /> Latest Blocks
                    </h3>

                    <div className="flex items-center gap-2 bg-background/50 p-1 rounded-xl border border-border/50">
                        <Button
                            variant="ghost" size="icon" className="h-8 w-8 rounded-lg"
                            disabled={page <= 1 || isLoading}
                            onClick={() => setPage(p => p - 1)}
                        >
                            <ChevronLeft className="h-4 w-4" />
                        </Button>
                        <span className="text-xs font-bold tabular-nums min-w-[80px] text-center text-muted-foreground uppercase tracking-wider">
                            Page {page} of {totalPages}
                        </span>
                        <Button
                            variant="ghost" size="icon" className="h-8 w-8 rounded-lg"
                            disabled={page >= totalPages || isLoading}
                            onClick={() => setPage(p => p + 1)}
                        >
                            <ChevronRight className="h-4 w-4" />
                        </Button>
                    </div>
                </div>

                <div className="flex-1 overflow-x-auto overflow-y-auto min-h-[400px]">
                    <table className="w-full text-left border-collapse min-w-[800px]">
                        <thead>
                            <tr className="border-b border-border/50 bg-secondary/10">
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-28">Height</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground">Block Hash</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-20 text-center">Shard</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-24 text-center">VDF</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-32">Proposer</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-24 text-right">Txs</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-widest text-muted-foreground w-32 text-right">Age</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-border/30">
                            {isLoading ? (
                                <tr>
                                    <td colSpan={5} className="py-24 text-center">
                                        <div className="flex flex-col items-center gap-3">
                                            <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
                                            <p className="text-sm text-muted-foreground font-medium">Fetching blockchain data...</p>
                                        </div>
                                    </td>
                                </tr>
                            ) : blocks.length > 0 ? (
                                blocks.map((block: any) => (
                                    <tr
                                        key={block.hash}
                                        className="hover:bg-primary/5 transition-colors cursor-pointer group"
                                        onClick={() => setSelectedBlock(block)}
                                    >
                                        <td className="p-5">
                                            <span className="font-mono font-bold text-primary text-sm">#{block.index}</span>
                                        </td>
                                        <td className="p-5">
                                            <div className="flex items-center gap-2 max-w-sm xl:max-w-md">
                                                <Hash className="w-3.5 h-3.5 text-muted-foreground/50" />
                                                <span className="font-mono text-sm text-muted-foreground group-hover:text-foreground transition-colors truncate">
                                                    {block.hash}
                                                </span>
                                            </div>
                                        </td>
                                        <td className="p-5 text-center">
                                            <Badge variant="outline" className="bg-primary/5 border-primary/20 text-primary font-mono text-[10px]">
                                                #{block.shard_id ?? 0}
                                            </Badge>
                                        </td>
                                        <td className="p-5 text-center">
                                            {block.vdf_proof ? (
                                                <Badge variant="secondary" className="font-mono text-[9px] opacity-70">
                                                    PROOF
                                                </Badge>
                                            ) : (
                                                <span className="text-[9px] text-muted-foreground">-</span>
                                            )}
                                        </td>
                                        <td className="p-5">
                                            <div className="flex items-center gap-2">
                                                <div className="w-5 h-5 rounded-full bg-gradient-to-tr from-blue-500/20 to-purple-500/20 md:flex hidden items-center justify-center">
                                                    <User className="w-3 h-3 text-primary" />
                                                </div>
                                                <span className="font-mono text-xs text-muted-foreground truncate max-w-[120px]">{block.author}</span>
                                            </div>
                                        </td>
                                        <td className="p-5 text-right">
                                            <Badge variant="secondary" className="font-mono text-xs">
                                                {block.transactions.length}
                                            </Badge>
                                        </td>
                                        <td className="p-5 text-right">
                                            <div className="flex items-center justify-end gap-1.5 text-xs text-muted-foreground font-medium">
                                                <Clock className="w-3 h-3 opacity-50" />
                                                {new Date(block.timestamp * 1000).toLocaleTimeString()}
                                            </div>
                                        </td>
                                    </tr>
                                ))
                            ) : (
                                <tr>
                                    <td colSpan={5} className="py-24 text-center text-muted-foreground">
                                        <Layers className="w-12 h-12 mx-auto mb-3 opacity-20" />
                                        <p>No blocks found matching your search</p>
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            <BlockDetailsModal
                block={selectedBlock}
                onClose={() => setSelectedBlock(null)}
                onTxClick={(tx) => {
                    // Start transition to tx details
                    // You might want to close block details or keep it open?
                    // User requested "transaction details displayed", usually implies opening another modal or switching
                    // Similar to Etherscan, opening a modal on top or switching views is fine.
                    // Let's stack the modals or close the block one?
                    // Stacking modals in Glassmorphism can be messy with backdrops.
                    // Let's close block modal for now or just overlay TxModal on top. 
                    // Let's try overlaying with higher z-index in TxModal.
                    setViewingTx(tx);
                }}
            />

            <TxDetailsModal
                tx={viewingTx}
                onClose={() => setViewingTx(null)}
            />
        </div>
    );
}
