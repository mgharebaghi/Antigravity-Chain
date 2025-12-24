import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Send, Coins, RefreshCw } from "lucide-react";
import PageTransition from "../components/PageTransition";
import { useToast } from "../context/ToastContext";
import { useApp } from "../context/AppContext";
import { formatNumber, calculateFee, parseAmount } from "../utils/format";

export default function Transactions() {
    const { success, error } = useToast();
    const { wallet, refreshWallet } = useApp();
    const [receiver, setReceiver] = useState('');
    const [amount, setAmount] = useState('');
    const [loading, setLoading] = useState(false);

    const atomicAmount = parseAmount(amount);
    const fee = calculateFee(atomicAmount);
    const total = atomicAmount + fee;
    const balance = wallet?.balance || 0;
    const isInsufficient = total > balance;

    async function sendTransaction(e: React.FormEvent) {
        e.preventDefault();

        // Basic frontend validation
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
            <div className="flex flex-col min-h-full container mx-auto p-4 sm:p-6 lg:max-w-4xl justify-center pb-10">
                <div className="flex items-center gap-2 sm:gap-3 shrink-0 mb-3 sm:mb-6 px-1 sm:px-2">
                    <div className="p-1.5 sm:p-2 bg-primary/10 rounded-lg">
                        <Send className="w-5 h-5 sm:w-6 sm:h-6 text-primary" />
                    </div>
                    <div>
                        <h1 className="text-xl sm:text-2xl font-bold tracking-tight">Transfer Assets</h1>
                        <p className="text-[10px] sm:text-sm text-muted-foreground">Securely send coins across the mesh</p>
                    </div>
                </div>

                <Card className="flex flex-col border-primary/10 shadow-2xl bg-gradient-to-br from-card/50 to-background/50 backdrop-blur-xl rounded-[1.5rem] sm:rounded-[2.5rem] overflow-hidden">
                    <CardHeader className="border-b border-border/50 pb-3 sm:pb-4 shrink-0 bg-muted/20">
                        <CardTitle className="text-xs sm:text-lg flex items-center gap-2 font-black uppercase tracking-widest text-primary/80">
                            <Coins className="w-4 h-4 sm:w-5 h-5" />
                            Transaction Details
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="p-4 sm:p-10 space-y-4 sm:space-y-8 flex-1 overflow-y-auto custom-scrollbar flex flex-col justify-center">
                        <form onSubmit={sendTransaction} className="space-y-4 sm:space-y-6 flex flex-col justify-center max-w-xl mx-auto w-full py-2 sm:py-4">
                            <div className="space-y-2">
                                <label className="text-[10px] sm:text-sm font-bold uppercase tracking-wider text-muted-foreground/60 ml-1">Receiver Address</label>
                                <div className="relative">
                                    <input
                                        type="text"
                                        value={receiver}
                                        onChange={(e) => setReceiver(e.target.value)}
                                        className="w-full p-3 sm:p-4 bg-muted/30 rounded-xl sm:rounded-2xl border border-input focus:border-primary/50 focus:ring-4 focus:ring-primary/10 outline-none transition-all font-mono text-[9px] sm:text-sm"
                                        placeholder="Enter public key (12D3...)"
                                        required
                                    />
                                </div>
                            </div>

                            <div className="space-y-2">
                                <label className="text-[10px] sm:text-sm font-bold uppercase tracking-wider text-muted-foreground/60 ml-1">Amount</label>
                                <div className="relative">
                                    <input
                                        type="number"
                                        value={amount}
                                        onChange={(e) => setAmount(e.target.value)}
                                        className="w-full p-3 sm:p-4 bg-muted/30 rounded-xl sm:rounded-2xl border border-input focus:border-primary/50 focus:ring-4 focus:ring-primary/10 outline-none transition-all text-lg sm:text-2xl font-black text-primary"
                                        placeholder="0.000000"
                                        required
                                        min="0.000001"
                                        step="0.000001"
                                    />
                                    <div className="absolute right-3 sm:right-4 top-1/2 -translate-y-1/2 text-[10px] sm:text-sm font-black bg-primary/20 text-primary px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-md sm:rounded-lg">AG</div>
                                </div>
                            </div>

                            <div className="space-y-4">
                                <div className="p-4 bg-muted/30 rounded-2xl border border-primary/5 space-y-3">
                                    <div className="flex justify-between text-xs font-bold text-muted-foreground uppercase tracking-wider">
                                        <span>Network Fee (0.01%)</span>
                                        <span className="text-foreground">{formatNumber(fee)} AGT</span>
                                    </div>
                                    <div className="h-px bg-border/20" />
                                    <div className="flex justify-between items-center">
                                        <span className="text-xs font-black text-primary uppercase tracking-widest">Total to Deduct</span>
                                        <span className={`text-xl font-black ${isInsufficient ? 'text-red-500' : 'text-foreground'}`}>
                                            {formatNumber(total)} AGT
                                        </span>
                                    </div>
                                </div>

                                {isInsufficient && (
                                    <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-2xl flex gap-3 text-xs text-red-600 font-bold items-center">
                                        <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse shrink-0" />
                                        Insufficient Balance: You need {formatNumber(total - balance)} more AGT.
                                    </div>
                                )}

                                <p className="text-[10px] text-muted-foreground text-center italic px-4">
                                    Important: Ensure the receiver address is correct. Transactions on the Antigravity Chain are irreversible.
                                </p>
                            </div>

                            <Button
                                type="submit"
                                size="lg"
                                className="w-full h-11 sm:h-16 font-black rounded-xl sm:rounded-[1.5rem] shadow-2xl shadow-primary/20 hover:scale-[1.02] active:scale-95 transition-all text-xs sm:text-lg mt-2 uppercase tracking-widest"
                                disabled={loading || isInsufficient || !amount || parseFloat(amount) <= 0}
                            >
                                {loading ? (
                                    <RefreshCw className="w-4 h-4 sm:w-6 sm:h-6 animate-spin mr-2" />
                                ) : (
                                    <Send className="w-4 h-4 sm:w-6 sm:h-6 mr-2 sm:mr-3" />
                                )}
                                {loading ? "Processing..." : "Authorize Transfer"}
                            </Button>
                            <p className="text-[7px] sm:text-[9px] text-muted-foreground/50 text-center uppercase tracking-tighter px-4 mt-2">
                                Final Confirmation: Antigravity transactions are cryptographic and final.
                            </p>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </PageTransition>
    );
}
