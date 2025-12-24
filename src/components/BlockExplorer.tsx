import { useState } from "react";
import { Layers, Hash, Clock, ChevronLeft, ChevronRight } from "lucide-react";
import { useApp, Block } from "../context/AppContext";
import { formatDistanceToNow } from "date-fns";
import BlockDetailsModal from "./BlockDetailsModal";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "./ui/button";
import { formatNumber } from "../utils/format";

export default function BlockExplorer() {
    const { recentBlocks, totalBlocks } = useApp();
    const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);
    const [currentPage, setCurrentPage] = useState(1);
    const blocksPerPage = 10;

    // Pagination logic
    // We use totalBlocks from AppContext for accurate total pages
    const totalPages = Math.ceil(totalBlocks / blocksPerPage);
    const startIndex = (currentPage - 1) * blocksPerPage;
    const currentBlocks = recentBlocks.slice(startIndex, startIndex + blocksPerPage);

    const handlePageChange = (newPage: number) => {
        if (newPage >= 1 && newPage <= totalPages) {
            setCurrentPage(newPage);
        }
    };

    return (
        <div className="space-y-4 h-full flex flex-col min-h-0 overflow-hidden">
            <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                    <Layers className="w-5 h-5 text-primary" /> Block Explorer
                </h3>
            </div>

            <div className="bg-card/20 backdrop-blur-xl border border-primary/5 rounded-2xl overflow-hidden shadow-2xl relative group flex flex-col min-h-0 flex-1">
                <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-transparent pointer-events-none" />
                <div className="overflow-x-auto relative z-10 custom-scrollbar">
                    <table className="w-full text-left border-collapse min-w-[700px] lg:min-w-0">
                        <thead>
                            <tr className="bg-secondary/20 border-b border-primary/5">
                                <th className="p-5 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60 w-24">Height</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60">Block Hash</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60 w-20">Txs</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60 w-32">Timestamp</th>
                                <th className="p-5 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60 w-32">Proposer</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-primary/5">
                            <AnimatePresence mode="popLayout">
                                {currentBlocks.length === 0 ? (
                                    <motion.tr
                                        initial={{ opacity: 0 }}
                                        animate={{ opacity: 1 }}
                                        exit={{ opacity: 0 }}
                                        key="empty"
                                    >
                                        <td colSpan={5} className="p-16 text-center text-muted-foreground italic">
                                            <div className="flex flex-col items-center gap-3 opacity-50">
                                                <Layers className="w-10 h-10 animate-pulse" />
                                                <p className="text-sm font-medium">Awaiting first block from consensus...</p>
                                            </div>
                                        </td>
                                    </motion.tr>
                                ) : (
                                    currentBlocks.map((block, idx) => (
                                        <motion.tr
                                            initial={{ opacity: 0, x: -10 }}
                                            animate={{ opacity: 1, x: 0 }}
                                            exit={{ opacity: 0, x: 10 }}
                                            transition={{ delay: idx * 0.03, type: "spring", stiffness: 100 }}
                                            key={block.hash}
                                            className="hover:bg-primary/5 transition-all cursor-pointer group/row"
                                            onClick={() => setSelectedBlock(block)}
                                        >
                                            <td className="p-5">
                                                <div className="px-3 py-1 rounded-lg bg-primary/10 text-primary font-black font-mono text-xs w-fit group-hover/row:scale-110 transition-transform">
                                                    #{formatNumber(block.index, false)}
                                                </div>
                                            </td>
                                            <td className="p-5">
                                                <div className="flex items-center gap-3">
                                                    <div className="w-8 h-8 rounded-lg bg-secondary/30 flex items-center justify-center text-muted-foreground group-hover/row:text-primary transition-colors border border-transparent group-hover/row:border-primary/20">
                                                        <Hash className="w-4 h-4" />
                                                    </div>
                                                    <span className="font-mono text-xs text-muted-foreground group-hover/row:text-foreground transition-colors truncate max-w-[140px] xl:max-w-none">
                                                        {block.hash}
                                                    </span>
                                                </div>
                                            </td>
                                            <td className="p-5">
                                                <div className="flex items-center gap-1.5 font-bold text-xs text-foreground/80">
                                                    {formatNumber(block.transactions.length, false)}
                                                    <span className="text-[10px] text-muted-foreground font-normal">TXS</span>
                                                </div>
                                            </td>
                                            <td className="p-5">
                                                <div className="flex items-center gap-2 text-[11px] text-muted-foreground font-medium">
                                                    <Clock className="w-3.5 h-3.5 opacity-60" />
                                                    {formatDistanceToNow(new Date(block.timestamp * 1000))} ago
                                                </div>
                                            </td>
                                            <td className="p-5">
                                                <div className="flex items-center gap-2">
                                                    <div className="w-5 h-5 rounded-full bg-gradient-to-tr from-primary/20 to-secondary/20 border border-primary/10" />
                                                    <span className="text-[11px] font-mono text-muted-foreground group-hover/row:text-primary transition-colors">
                                                        {block.author.substring(0, 8)}...
                                                    </span>
                                                </div>
                                            </td>
                                        </motion.tr>
                                    ))
                                )}
                            </AnimatePresence>
                        </tbody>
                    </table>
                </div>

                {/* Footer Pagination */}
                {recentBlocks.length > 0 && (
                    <div className="p-4 border-t border-primary/5 bg-secondary/10 flex items-center justify-between shrink-0">
                        <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest">
                            Total Blocks: {formatNumber(totalBlocks, false)}
                        </div>
                        <div className="flex items-center gap-2 bg-background/40 p-1 rounded-xl border border-primary/10">
                            <Button
                                variant="ghost"
                                size="icon"
                                disabled={currentPage === 1}
                                onClick={() => handlePageChange(currentPage - 1)}
                                className="w-7 h-7 rounded-lg"
                            >
                                <ChevronLeft className="w-3.5 h-3.5" />
                            </Button>
                            <span className="text-[9px] font-black tracking-[0.2em] uppercase px-3 text-muted-foreground/80">
                                {currentPage} / {totalPages || 1}
                            </span>
                            <Button
                                variant="ghost"
                                size="icon"
                                disabled={currentPage >= totalPages}
                                onClick={() => handlePageChange(currentPage + 1)}
                                className="w-7 h-7 rounded-lg"
                            >
                                <ChevronRight className="w-3.5 h-3.5" />
                            </Button>
                        </div>
                    </div>
                )}
            </div>

            <BlockDetailsModal
                block={selectedBlock}
                onClose={() => setSelectedBlock(null)}
            />
        </div>
    );
}
