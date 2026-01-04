import { ShieldCheck, Zap } from "lucide-react";
import { motion } from "framer-motion";
import { cn } from "../../lib/utils";
import { formatNumber } from "../../utils/format";
import { VdfStatus } from "../../context/AppContext";

interface VdfVisualizerProps {
    vdfStatus: VdfStatus | null;
    isActive: boolean;
}

export function VdfVisualizer({ vdfStatus, isActive }: VdfVisualizerProps) {
    // Normalizing value for gauge (0 to 150k IPS)
    const maxIps = 150000;
    const ips = vdfStatus?.iterations_per_second || 0;
    const percentage = Math.min(ips / maxIps, 1);

    // Gauge Constants for math - Refined for clarity and "air"
    const startAngle = -110;
    const endAngle = 110;
    const totalSweep = endAngle - startAngle;
    const centerX = 150;
    const centerY = 155;
    const radius = 100;  // Balanced size to prevent crowded feel

    // Arc path calculation
    const polarToCartesian = (cx: number, cy: number, r: number, angleInDegrees: number) => {
        const angleInRadians = (angleInDegrees - 90) * Math.PI / 180.0;
        return {
            x: cx + (r * Math.cos(angleInRadians)),
            y: cy + (r * Math.sin(angleInRadians))
        };
    };

    const describeArc = (x: number, y: number, r: number, startA: number, endA: number) => {
        const start = polarToCartesian(x, y, r, startA);
        const end = polarToCartesian(x, y, r, endA);
        const largeArcFlag = endA - startA <= 180 ? "0" : "1";
        return [
            "M", start.x, start.y,
            "A", r, r, 0, largeArcFlag, 1, end.x, end.y
        ].join(" ");
    };

    const arcPath = describeArc(centerX, centerY, radius, startAngle, endAngle);
    const arcLength = (totalSweep / 360) * 2 * Math.PI * radius;

    const tickMarks = Array.from({ length: 23 });
    const labels = [
        { val: 0, label: "0" },
        { val: 0.25, label: "35k" },
        { val: 0.5, label: "75k" },
        { val: 0.75, label: "115k" },
        { val: 1, label: "150k" }
    ];

    return (
        <div className="lg:col-span-2 relative overflow-hidden glass-card p-4 md:p-6 rounded-3xl flex flex-col justify-between h-full group transition-all duration-500 hover:shadow-2xl hover:shadow-primary/10 border border-white/10">

            <div className="absolute -right-4 -top-4 w-40 h-40 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors duration-500" />

            <div className="relative z-10 w-full flex flex-col items-center">
                {/* Header matching StatCard style */}
                <div className="w-full flex justify-between items-start mb-0">
                    <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-2">
                            <div className="p-1.5 bg-primary/10 rounded-xl group-hover:scale-110 group-hover:bg-primary/20 transition-all duration-500">
                                <Zap className="w-3.5 h-3.5 text-primary" />
                            </div>
                            <p className="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest">
                                VDF Performance
                            </p>
                        </div>
                    </div>
                    {/* Status Pill */}
                    <div className={cn(
                        "px-3 py-1 rounded-full text-[9px] font-black tracking-widest uppercase transition-all duration-500 border",
                        isActive
                            ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.1)]"
                            : "bg-muted/50 border-white/5 text-muted-foreground/50"
                    )}>
                        {isActive ? "Engine Running" : "Standby"}
                    </div>
                </div>

                {/* Primary Visual: The Speedometer */}
                <div className="relative w-full aspect-[16/7.5] max-h-[220px] flex items-center justify-center -mt-6">
                    <svg className="w-full h-full overflow-visible" viewBox="0 0 300 180">
                        <defs>
                            <linearGradient id="vdfGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                                <stop offset="0%" stopColor="#3b82f6" />
                                <stop offset="25%" stopColor="#06b6d4" />
                                <stop offset="50%" stopColor="#22c55e" />
                                <stop offset="75%" stopColor="#f97316" />
                                <stop offset="100%" stopColor="#ef4444" />
                            </linearGradient>

                            <filter id="arcGlow" x="-20%" y="-20%" width="140%" height="140%">
                                <feGaussianBlur stdDeviation="5" result="blur" />
                                <feComposite in="SourceGraphic" in2="blur" operator="over" />
                            </filter>
                        </defs>

                        {/* Tick Marks - Positioned with air */}
                        {tickMarks.map((_, i) => {
                            const p = i / (tickMarks.length - 1);
                            const angle = startAngle + (p * totalSweep);
                            const pos1 = polarToCartesian(centerX, centerY, radius + 6, angle);
                            const pos2 = polarToCartesian(centerX, centerY, radius + (i % 5 === 0 ? 12 : 9), angle);

                            return (
                                <line
                                    key={i}
                                    x1={pos1.x} y1={pos1.y} x2={pos2.x} y2={pos2.y}
                                    stroke="currentColor"
                                    strokeWidth={i % 5 === 0 ? 1.5 : 1}
                                    className={cn(
                                        "transition-opacity duration-1000",
                                        p <= percentage ? "text-primary opacity-40" : "text-muted-foreground/10"
                                    )}
                                />
                            );
                        })}

                        {/* Numerical Labels - Positioned for clarity */}
                        {labels.map((l, i) => {
                            const angle = startAngle + (l.val * totalSweep);
                            const pos = polarToCartesian(centerX, centerY, radius - 20, angle);

                            return (
                                <text
                                    key={i}
                                    x={pos.x} y={pos.y}
                                    textAnchor="middle"
                                    alignmentBaseline="middle"
                                    className={cn(
                                        "text-[8px] font-black transition-colors duration-500",
                                        l.val <= percentage ? "fill-foreground/80" : "fill-muted-foreground/20"
                                    )}
                                >
                                    {l.label}
                                </text>
                            );
                        })}

                        {/* Track Background */}
                        <path
                            d={arcPath}
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="8"
                            strokeLinecap="round"
                            className="text-muted-foreground/5 dark:text-white/5"
                        />

                        {/* Active Progress Arc */}
                        <motion.path
                            d={arcPath}
                            fill="none"
                            stroke="url(#vdfGradient)"
                            strokeWidth="10"
                            strokeLinecap="round"
                            strokeDasharray={arcLength}
                            initial={{ strokeDashoffset: arcLength }}
                            animate={{ strokeDashoffset: arcLength * (1 - percentage) }}
                            transition={{ duration: 1.5, ease: "easeOut" }}
                            filter={isActive ? "url(#arcGlow)" : ""}
                        />
                    </svg>

                    {/* Central Value Readout */}
                    <div className="absolute top-[55%] flex flex-col items-center">
                        <motion.span
                            key={ips}
                            initial={{ opacity: 0.8, scale: 0.95, y: 5 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            className={cn(
                                "text-4xl md:text-5xl font-black text-foreground tracking-tighter tabular-nums drop-shadow-2xl transition-all duration-300",
                                percentage > 0.9 && "text-red-500"
                            )}
                        >
                            {formatNumber(ips, false)}
                        </motion.span>
                        <div className="flex items-center gap-1.5 mt-0">
                            <span className="text-[9px] font-black text-muted-foreground/40 uppercase tracking-[0.3em]">
                                iterations / sec
                            </span>
                            {isActive && (
                                <span className="flex h-1.5 w-1.5">
                                    <span className="animate-ping absolute inline-flex h-1.5 w-1.5 rounded-full bg-primary opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-primary"></span>
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {/* Informational Footer Section */}
            <div className="relative z-10 mt-auto p-3 rounded-2xl bg-white/5 border border-white/5 backdrop-blur-md">
                <div className="flex gap-2.5 items-start">
                    <div className="mt-0.5 p-1.5 rounded-lg bg-primary/10">
                        <ShieldCheck className="w-3.5 h-3.5 text-primary" />
                    </div>
                    <div className="flex-1 min-w-0">
                        <div className="flex justify-between items-center mb-0.5">
                            <h4 className="text-[9px] font-black text-foreground uppercase tracking-widest truncate mr-2">Computational Proof of Time</h4>
                            <span className="shrink-0 px-1.5 py-0.5 rounded-md bg-primary/10 text-primary text-[7px] font-black uppercase tracking-tighter">Network Integrity</span>
                        </div>
                        <p className="text-[9px] leading-snug text-muted-foreground/70 font-medium tracking-tight line-clamp-2 hover:line-clamp-none transition-all duration-300">
                            The <span className="text-primary font-bold">VDF Protocol</span> measures sequential processing speed.
                            While stronger hardware produces more iterations per second, the network remains <span className="text-foreground font-bold italic">fair and decentralized</span>.
                            VDFs are non-parallelizable, ensuring that <span className="text-foreground/90">raw power cannot bypass time</span> or centralize control over block validation.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}
