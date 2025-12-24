import { motion, AnimatePresence } from "framer-motion";
import { LogOut, UserCircle, ArrowRightLeft } from "lucide-react";
import { Button } from "./ui/button";

interface SwitchWalletModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    currentAddress: string;
}

export default function SwitchWalletModal({ isOpen, onClose, onConfirm, currentAddress }: SwitchWalletModalProps) {
    const truncatedAddress = currentAddress
        ? `${currentAddress.slice(0, 10)}...${currentAddress.slice(-10)}`
        : "Unknown Address";

    return (
        <AnimatePresence>
            {isOpen && (
                <>
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-[100] bg-background/60 backdrop-blur-lg"
                        onClick={onClose}
                    />

                    {/* Modal */}
                    <div className="fixed inset-0 z-[110] flex items-center justify-center p-4 pointer-events-none">
                        <motion.div
                            initial={{ opacity: 0, scale: 0.95, y: 30 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.95, y: 30 }}
                            transition={{ type: "spring", damping: 25, stiffness: 400 }}
                            className="bg-card/90 backdrop-blur-2xl w-full max-w-md rounded-[2.5rem] border border-primary/20 shadow-[0_40px_80px_-15px_rgba(139,92,246,0.2)] overflow-hidden pointer-events-auto relative"
                        >
                            {/* Decorative Top Gradient */}
                            <div className="absolute top-0 inset-x-0 h-1.5 bg-gradient-to-r from-primary/10 via-primary to-primary/10" />

                            <div className="p-10">
                                <div className="flex flex-col items-center text-center gap-8">
                                    {/* Icon Branding */}
                                    <div className="relative group">
                                        <motion.div
                                            whileHover={{ rotate: 180 }}
                                            transition={{ duration: 0.6 }}
                                            className="w-20 h-20 rounded-[2rem] bg-primary/10 flex items-center justify-center text-primary relative z-10 border border-primary/20 shadow-inner group-hover:bg-primary/20 transition-colors"
                                        >
                                            <ArrowRightLeft className="w-10 h-10" />
                                        </motion.div>
                                        <div className="absolute inset-0 rounded-[2rem] bg-primary/20 blur-2xl opacity-50 group-hover:opacity-80 transition-opacity" />
                                    </div>

                                    <div className="space-y-3">
                                        <h2 className="text-3xl font-black tracking-tight text-foreground">
                                            Switch Identity
                                        </h2>
                                        <p className="text-muted-foreground text-sm font-medium px-4">
                                            You are about to log out of your current session. Mining and active network tasks will stop immediately.
                                        </p>
                                    </div>

                                    {/* Snapshot of Current Identity */}
                                    <div className="w-full bg-secondary/30 rounded-3xl p-5 border border-primary/10 flex flex-col items-center gap-3">
                                        <div className="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">
                                            <UserCircle className="w-3 h-3" /> Current Wallet
                                        </div>
                                        <div className="font-mono text-xs font-bold text-primary bg-primary/5 px-4 py-2 rounded-xl border border-primary/10 select-all">
                                            {truncatedAddress}
                                        </div>
                                    </div>

                                    {/* Actions */}
                                    <div className="flex flex-col w-full gap-3">
                                        <Button
                                            onClick={onConfirm}
                                            size="lg"
                                            className="w-full h-14 rounded-2xl font-black text-base gap-3 bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg shadow-primary/20 hover:scale-[1.02] active:scale-[0.98] transition-all"
                                        >
                                            <LogOut className="w-5 h-5" />
                                            Confirm Logout
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            onClick={onClose}
                                            className="h-14 rounded-2xl text-muted-foreground font-bold hover:bg-secondary/50 group"
                                        >
                                            Stay Signed In
                                        </Button>
                                    </div>

                                    <div className="flex items-center gap-2 text-[10px] text-muted-foreground/60 font-semibold uppercase tracking-widest">
                                        <ShieldCheck className="w-3 h-3 text-green-500" /> Secure Logout
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

function ShieldCheck({ className }: { className?: string }) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
            <path d="m9 12 2 2 4-4" />
        </svg>
    );
}
