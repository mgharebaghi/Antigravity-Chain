import { Activity, Zap, Layers, Pickaxe, RefreshCw, Shield, Info, Gauge } from 'lucide-react';
import { motion } from 'framer-motion';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { cn, formatPatience } from '../lib/utils';
import PageTransition from './PageTransition';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatNumber } from '../utils/format';

export default function Dashboard() {
    // Consume global state
    const { wallet, nodeStatus, patience, peers, startNode, stopNode, latestBlock, minedBlocks, vdfStatus } = useApp();

    const handleStartNodeClick = async () => {
        try {
            await startNode();
        } catch (e) {
            // Already handled in context but could show toast here
            console.error(e);
        }
    };

    const handleRetry = async () => {
        // first stop, then start
        await stopNode();
        setTimeout(() => startNode(), 2000);
    };

    const isRunning = nodeStatus.startsWith('Active') || nodeStatus.includes('Syncing');

    // In-progress states that are not "Active" but the service is STARTING/WORKING
    const isStarting = nodeStatus.includes('Connecting') || nodeStatus.includes('Searching') || nodeStatus.includes('Creating');

    // Explicit error or connection stuck
    const isError = nodeStatus === "Relay Unreachable" || nodeStatus === "Connection Lost";
    const isActive = nodeStatus === "Active";

    return (
        <PageTransition>
            <div className="flex flex-col min-h-full gap-3 sm:gap-6 pb-6">

                {/* Status Section: Responsive grid */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 shrink-0">
                    <Card className={cn(
                        "bg-card/50 backdrop-blur-sm border-primary/20 transition-all hover:bg-card/80 shadow-lg min-h-0 flex flex-col",
                        isError && "border-red-500/30 bg-red-500/5",
                        isActive && "border-emerald-500/30 bg-emerald-500/5 shadow-emerald-500/5"
                    )}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-1.5 sm:pb-2 pt-3 sm:pt-6">
                            <CardTitle className="text-[10px] sm:text-sm font-medium uppercase tracking-wider text-muted-foreground/60">Node Status</CardTitle>
                            <Activity className={cn("h-3.5 w-3.5 sm:h-4 sm:w-4",
                                isActive ? "text-emerald-500 drop-shadow-[0_0_8px_rgba(16,185,129,0.5)]" :
                                    isRunning ? "text-green-500" :
                                        isError ? "text-red-500" : "text-muted-foreground"
                            )} />
                        </CardHeader>
                        <CardContent className="pb-3 sm:pb-6">
                            <div className={cn("font-bold leading-tight truncate",
                                isError ? "text-red-500 text-base sm:text-xl" :
                                    isActive ? "text-emerald-500 text-lg sm:text-2xl" : "text-lg sm:text-2xl"
                            )}>
                                {isError ? "Connection Failed" : nodeStatus}
                            </div>
                            <p className={cn("text-[9px] sm:text-xs mt-0.5 sm:mt-1 truncate",
                                isError ? "text-red-400 font-medium" :
                                    isActive ? "text-emerald-400 font-medium" : "text-muted-foreground"
                            )}>
                                {isRunning || isStarting ? (
                                    <>
                                        {peers > 0 ? `Connected to ${peers} peers` :
                                            isActive ? "Synchronized & Secure" : "Searching for network..."}
                                    </>
                                ) : isError ? "Unable to reach relay network" : "Service is offline"}
                            </p>
                        </CardContent>
                    </Card>

                    <Card className="bg-card/50 backdrop-blur-sm border-primary/20 transition-all hover:bg-card/80 shadow-lg min-h-0 flex flex-col">
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-1.5 sm:pb-2 pt-3 sm:pt-6">
                            <CardTitle className="text-[10px] sm:text-sm font-medium uppercase tracking-wider text-muted-foreground/60">Block Height</CardTitle>
                            <Layers className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                        </CardHeader>
                        <CardContent className="pb-3 sm:pb-6">
                            <div className="text-lg sm:text-2xl font-bold font-mono leading-none">
                                {latestBlock ? `#${formatNumber(latestBlock.index, false)}` : "-"}
                            </div>
                            <p className="text-[9px] sm:text-xs text-muted-foreground truncate mt-1">
                                {latestBlock ? `Hash: ${latestBlock.hash.substring(0, 16)}...` : "Waiting for sync..."}
                            </p>
                        </CardContent>
                    </Card>

                    <Card className="bg-card/50 backdrop-blur-sm border-primary/20 transition-all hover:bg-card/80 shadow-lg sm:grid-cols-1 sm:col-span-2 lg:col-span-1 min-h-0 flex flex-col">
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-1.5 sm:pb-2 pt-3 sm:pt-6">
                            <CardTitle className="text-[10px] sm:text-sm font-medium uppercase tracking-wider text-muted-foreground/60">Mined Blocks</CardTitle>
                            <Pickaxe className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground" />
                        </CardHeader>
                        <CardContent className="pb-3 sm:pb-6">
                            <div className="text-lg sm:text-2xl font-bold text-primary leading-none">{formatNumber(minedBlocks, false)}</div>
                            <p className="text-[9px] sm:text-xs text-muted-foreground truncate mt-1">
                                {minedBlocks > 0
                                    ? "Blocks confirmed by this node"
                                    : "No blocks mined yet"}
                            </p>
                        </CardContent>
                    </Card>

                    <Card className={cn(
                        "bg-card/50 backdrop-blur-sm border-primary/20 transition-all hover:bg-card/80 shadow-lg lg:col-span-3 border-l-4 min-h-0",
                        vdfStatus?.is_active ? "border-l-emerald-500" : "border-l-muted"
                    )}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-1.5 sm:pb-2 pt-3 sm:pt-4 px-4 sm:px-6">
                            <div className="flex items-center gap-2">
                                <CardTitle className="text-[10px] sm:text-xs font-bold uppercase tracking-[0.2em] text-muted-foreground/60">Sybil Resistance Engine (VDF)</CardTitle>
                                <div className="group relative hidden xs:block">
                                    <Info className="h-3 w-3 text-muted-foreground/40 cursor-help transition-colors group-hover:text-primary" />
                                    <div className="absolute left-0 bottom-full mb-2 w-64 p-3 bg-card border border-primary/20 rounded-xl shadow-2xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all text-[10px] text-muted-foreground leading-relaxed z-50">
                                        <p className="font-bold text-primary mb-1">What is Sybil Resistance?</p>
                                        VDF prevents attackers from creating fake identities to manipulate the network. It requires a sequential "Proof of Patience" that cannot be parallelized, making the network fair for all nodes.
                                    </div>
                                </div>
                            </div>
                            <Shield className={cn("h-3.5 w-3.5 sm:h-4 sm:w-4 transition-colors",
                                vdfStatus?.is_active ? "text-emerald-500" : "text-muted-foreground"
                            )} />
                        </CardHeader>
                        <CardContent className="pb-3 sm:pb-4 px-4 sm:px-6">
                            <div className="flex items-center justify-between gap-4">
                                <div className="min-w-0">
                                    <div className={cn("text-sm sm:text-lg font-black tracking-tight truncate flex items-center gap-2",
                                        vdfStatus?.is_active ? "text-emerald-500" : "text-foreground"
                                    )}>
                                        {vdfStatus?.is_active ? (
                                            <>
                                                <Gauge className="w-3.5 h-3.5 sm:w-4 h-4" />
                                                {formatNumber(vdfStatus.iterations_per_second, false)} <span className="text-[9px] sm:text-[10px] text-muted-foreground font-medium">H/s</span>
                                            </>
                                        ) : "Passive Protection"}
                                    </div>
                                    <p className="text-[8px] sm:text-[10px] font-medium text-muted-foreground uppercase tracking-widest mt-0.5">
                                        {vdfStatus?.is_active ? "Protocol secure via sequential hashing" : "Engine awaiting node initialization"}
                                    </p>
                                </div>
                                <div className="flex flex-col items-end shrink-0">
                                    <span className="text-[7px] sm:text-[8px] font-black text-muted-foreground/50 uppercase tracking-tighter">Current Difficulty</span>
                                    <span className="text-[10px] sm:text-xs font-mono font-extrabold text-primary/80">
                                        {vdfStatus ? `${formatNumber(vdfStatus.difficulty, false)} SHA-256` : "200k SHA-256"}
                                    </span>
                                </div>
                            </div>

                            {/* Live Progress Bar Simulation */}
                            {vdfStatus?.is_active && (
                                <div className="mt-2 sm:mt-3 w-full h-1 bg-muted/20 rounded-full overflow-hidden">
                                    <motion.div
                                        animate={{ x: ['-100%', '100%'] }}
                                        transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
                                        className="w-1/3 h-full bg-primary/40 rounded-full"
                                    ></motion.div>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                {/* Visualizer Section: Flex Grow, Adaptive sizing */}
                <Card className={cn("flex-1 min-h-0 relative overflow-hidden flex items-center justify-center border-primary/10 shadow-2xl transition-colors duration-1000",
                    isError && "border-red-500/20 bg-red-950/20",
                    isActive && "border-emerald-500/20 bg-emerald-950/10"
                )}>
                    <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent pointer-events-none" />

                    {isError && (
                        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-red-500/5 via-transparent to-transparent pointer-events-none" />
                    )}

                    {isActive && (
                        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-emerald-500/10 via-transparent to-transparent pointer-events-none" />
                    )}

                    <div className="relative z-10 text-center flex flex-col items-center justify-center gap-4 sm:gap-8 h-full w-full p-4 sm:p-10 min-h-0">
                        {/* Visualizer: Adaptive size */}
                        <div className="relative w-36 h-36 sm:w-64 sm:h-64 lg:w-72 lg:h-72 flex items-center justify-center shrink-0">
                            {!isError ? (
                                <>
                                    <motion.div
                                        animate={isRunning || isStarting ? { rotate: 360 } : {}}
                                        transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
                                        className={cn("absolute w-full h-full rounded-[40%] border",
                                            isActive ? "border-emerald-500/30" : "border-primary/20"
                                        )}
                                    />
                                    <motion.div
                                        animate={isRunning || isStarting ? { rotate: -360 } : {}}
                                        transition={{ duration: 15, repeat: Infinity, ease: "linear" }}
                                        className={cn("absolute w-[85%] h-[85%] rounded-[35%] border",
                                            isActive ? "border-emerald-500/50" : "border-primary/40"
                                        )}
                                    />
                                    <motion.div
                                        animate={isRunning || isStarting ? { scale: [1, 1.05, 1] } : {}}
                                        transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
                                        className={cn("absolute w-[70%] h-[70%] rounded-full border blur-[1px]",
                                            isActive ? "bg-emerald-500/10 border-emerald-500/60" : "bg-primary/5 border-primary/60"
                                        )}
                                    />
                                    {isActive && (
                                        <motion.div
                                            animate={{ scale: [1, 1.2, 1], opacity: [0.1, 0.3, 0.1] }}
                                            transition={{ duration: 4, repeat: Infinity }}
                                            className="absolute w-full h-full rounded-full bg-emerald-500/10 blur-3xl -z-10"
                                        />
                                    )}
                                </>
                            ) : (
                                <div className="relative flex items-center justify-center">
                                    <motion.div
                                        animate={{ scale: [1, 1.1, 1] }}
                                        transition={{ duration: 3, repeat: Infinity }}
                                        className="absolute w-full h-full rounded-full bg-red-500/10 blur-3xl"
                                    />
                                    <div className="w-32 h-32 sm:w-40 sm:h-40 rounded-full border-2 border-dashed border-red-500/30 flex items-center justify-center relative">
                                        <Activity className="w-16 h-16 sm:w-20 sm:h-20 text-red-500 drop-shadow-[0_0_15px_rgba(239,68,68,0.5)]" />
                                        <div className="absolute inset-0 rounded-full border-2 border-red-500/5 animate-ping" />
                                    </div>
                                </div>
                            )}

                            {!isError && (
                                <div className="relative z-20 flex flex-col items-center">
                                    <span className={cn("text-[10px] sm:text-xs font-bold uppercase tracking-[0.2em] mb-1",
                                        isActive ? "text-emerald-400" : "text-muted-foreground"
                                    )}>
                                        {isActive ? "Connected" : "Patience"}
                                    </span>
                                    <div className={cn("text-2xl sm:text-5xl lg:text-6xl font-black font-mono tracking-tighter drop-shadow-2xl transition-colors duration-1000",
                                        isActive ? "text-emerald-500" : "text-foreground"
                                    )}>
                                        {formatPatience(patience)}
                                    </div>
                                    {vdfStatus?.is_active && (
                                        <div className="absolute -bottom-10 left-1/2 -translate-x-1/2 w-max text-[8px] sm:text-[10px] font-black uppercase tracking-widest text-primary animate-pulse bg-primary/10 px-3 py-1 rounded-full border border-primary/20">
                                            Calculating VDF Clock Pulse...
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>

                        {/* Content Wrapper */}
                        <div className="flex flex-col items-center gap-3 sm:gap-4 max-w-lg">
                            <div className="space-y-1 sm:space-y-2">
                                <h3 className={cn("text-lg sm:text-3xl lg:text-4xl font-black tracking-tight transition-colors duration-1000 leading-none",
                                    isError ? "text-red-500 uppercase" :
                                        isActive ? "text-emerald-500" : ""
                                )}>
                                    {isError ? "Network Unreachable" : wallet ? "Active Accumulator" : "Account Setup"}
                                </h3>
                                <p className="text-muted-foreground mx-auto text-[10px] sm:text-sm leading-relaxed px-4">
                                    {isError
                                        ? "No active relays found. Please check your connection."
                                        : isActive
                                            ? "Your node is successfully contributing to the network."
                                            : wallet
                                                ? "Your node is earning potential rewards."
                                                : "A wallet is required to participate in consensus."}
                                </p>
                            </div>

                            <div className="flex flex-col sm:flex-row gap-3 pt-4 w-full justify-center">
                                {!wallet ? (
                                    <Button asChild size="lg" className="h-10 sm:h-12 px-6 sm:px-8 rounded-xl sm:rounded-2xl shadow-xl shadow-primary/20 text-xs sm:text-sm font-bold animate-pulse">
                                        <Link to="/wallet">Go to Wallet</Link>
                                    </Button>
                                ) : (
                                    nodeStatus === "Stopped" && (
                                        <Button onClick={handleStartNodeClick} size="lg" className="gap-2 px-8 sm:px-10 h-11 sm:h-14 rounded-xl sm:rounded-2xl shadow-2xl shadow-primary/30 text-sm sm:base font-bold hover:scale-105 active:scale-95 transition-all">
                                            <Zap className="w-4 h-4 sm:w-5 sm:h-5 fill-current" /> Initialize Node
                                        </Button>
                                    )
                                )}

                                {isActive && (
                                    <div className="flex items-center gap-2 px-6 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-500 text-xs font-bold animate-pulse">
                                        <div className="w-2 h-2 rounded-full bg-emerald-500" />
                                        LIVE ON NETWORK
                                    </div>
                                )}

                                {isError && (
                                    <Button onClick={handleRetry} size="lg" variant="destructive" className="gap-2 px-8 sm:px-10 h-11 sm:h-14 rounded-xl sm:rounded-2xl shadow-2xl shadow-red-500/30 text-sm sm:base font-bold hover:scale-105 active:scale-95 transition-all border-none bg-red-600 hover:bg-red-700">
                                        <RefreshCw className="w-4 h-4 sm:w-5 sm:h-5" /> Restart Node
                                    </Button>
                                )}
                            </div>
                        </div>
                    </div>
                </Card>
            </div>
        </PageTransition>
    );
}
