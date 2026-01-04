import { motion, AnimatePresence } from "framer-motion";
import { AlertTriangle, Trash2, ShieldAlert } from "lucide-react";
import { useState, useEffect } from "react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";

interface WipeDataModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
}

export default function WipeDataModal({ isOpen, onClose, onConfirm }: WipeDataModalProps) {
    const [confirmText, setConfirmText] = useState("");
    const [isShaking, setIsShaking] = useState(false);

    useEffect(() => {
        if (!isOpen) {
            setConfirmText("");
        }
    }, [isOpen]);

    const handleConfirm = () => {
        if (confirmText.toUpperCase() === "WIPE") {
            onConfirm();
        } else {
            setIsShaking(true);
            setTimeout(() => setIsShaking(false), 500);
        }
    };

    return (
        <AnimatePresence>
            {isOpen && (
                <>
                    {/* Glass Backing Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-[100] bg-background/40 backdrop-blur-md transition-all"
                        onClick={onClose}
                    />

                    {/* Modal Container */}
                    <div className="fixed inset-0 z-[110] flex items-center justify-center p-4 pointer-events-none">
                        <motion.div
                            initial={{ opacity: 0, scale: 0.9, y: 20, rotateX: 15 }}
                            animate={{
                                opacity: 1,
                                scale: 1,
                                y: 0,
                                rotateX: 0,
                                x: isShaking ? [0, -10, 10, -10, 10, 0] : 0
                            }}
                            exit={{ opacity: 0, scale: 0.95, y: 20 }}
                            transition={{ type: "spring", damping: 20, stiffness: 300 }}
                            className="bg-card/80 backdrop-blur-2xl w-full max-w-md rounded-[2rem] border border-destructive/20 shadow-[0_32px_64px_-12px_rgba(220,38,38,0.25)] overflow-hidden pointer-events-auto relative"
                        >
                            {/* Animated Top Glow */}
                            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-1/2 h-1 bg-gradient-to-r from-transparent via-destructive to-transparent opacity-50 shadow-[0_0_20px_2px_rgba(220,38,38,0.5)]" />

                            <div className="p-8">
                                <div className="flex flex-col items-center text-center gap-6">
                                    {/* Icon with Ring Animation */}
                                    <div className="relative">
                                        <motion.div
                                            animate={{ scale: [1, 1.1, 1] }}
                                            transition={{ repeat: Infinity, duration: 2 }}
                                            className="p-4 rounded-full bg-destructive/10 text-destructive relative z-10"
                                        >
                                            <AlertTriangle className="w-10 h-10" />
                                        </motion.div>
                                        <div className="absolute inset-0 rounded-full bg-destructive/20 animate-ping opacity-20 scale-150" />
                                    </div>

                                    <div className="space-y-2">
                                        <h2 className="text-2xl font-black tracking-tight text-foreground flex items-center justify-center gap-2">
                                            Persistent Destruction
                                        </h2>
                                        <p className="text-muted-foreground text-sm font-medium leading-relaxed">
                                            This action will <span className="text-destructive font-bold underline underline-offset-4 decoration-2">permanently delete</span> all local blockchain data, indexes, and cache. Your node will need to resync from scratch.
                                        </p>
                                    </div>

                                    {/* Confirmation Input Area */}
                                    <div className="w-full space-y-4 pt-2">
                                        <div className="relative group">
                                            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground group-focus-within:text-destructive transition-colors">
                                                <ShieldAlert className="w-4 h-4" />
                                            </div>
                                            <input
                                                type="text"
                                                value={confirmText}
                                                onChange={(e) => setConfirmText(e.target.value)}
                                                placeholder='Type "WIPE" to confirm'
                                                className="w-full h-12 pl-10 pr-4 bg-muted/30 hover:bg-muted/50 focus:bg-background rounded-2xl border border-border focus:border-destructive/50 focus:ring-4 focus:ring-destructive/10 outline-none transition-all placeholder:text-muted-foreground/50 font-mono text-center tracking-[0.2em] uppercase"
                                            />
                                        </div>
                                    </div>

                                    {/* Buttons */}
                                    <div className="flex flex-col w-full gap-3">
                                        <motion.div
                                            whileHover={confirmText.toUpperCase() === "WIPE" ? { scale: 1.02 } : {}}
                                            whileTap={confirmText.toUpperCase() === "WIPE" ? { scale: 0.98 } : {}}
                                        >
                                            <Button
                                                onClick={handleConfirm}
                                                disabled={confirmText.toUpperCase() !== "WIPE"}
                                                className={cn(
                                                    "w-full h-12 rounded-2xl font-bold gap-2 text-base transition-all duration-300",
                                                    confirmText.toUpperCase() === "WIPE"
                                                        ? "bg-destructive hover:bg-destructive/90 text-destructive-foreground shadow-[0_10px_20px_-5px_rgba(220,38,38,0.4)]"
                                                        : "bg-muted text-muted-foreground grayscale cursor-not-allowed"
                                                )}
                                            >
                                                <Trash2 className="w-5 h-5" />
                                                Wipe Everything
                                            </Button>
                                        </motion.div>
                                        <Button
                                            variant="ghost"
                                            onClick={onClose}
                                            className="h-12 rounded-2xl hover:bg-secondary text-muted-foreground font-semibold"
                                        >
                                            Cancel & Go Back
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        </motion.div>
                    </div>
                </>
            )}
        </AnimatePresence>
    );
}
