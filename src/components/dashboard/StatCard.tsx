import { LucideIcon } from 'lucide-react';
import { cn } from '../../lib/utils';

interface StatCardProps {
    title: string;
    value: string | React.ReactNode;
    subValue?: string;
    icon: LucideIcon;
    statusColor?: string;
    footerLabel?: string;
    footerValue?: string;
    className?: string; // Allow custom styling extensions
}

export function StatCard({ title, value, subValue, icon: Icon, statusColor, footerLabel, footerValue, className }: StatCardProps) {
    return (
        <div className={cn(
            "relative overflow-hidden glass-card p-6 rounded-3xl flex flex-col justify-between h-full group transition-all duration-500",
            "hover:shadow-2xl hover:shadow-primary/10 hover:-translate-y-1 border border-white/10",
            className
        )}>
            {/* Background Accent */}
            <div className="absolute -right-4 -top-4 w-24 h-24 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors duration-500" />

            <div className="relative z-10">
                <div className="flex justify-between items-start mb-4">
                    <div className="p-2.5 bg-primary/10 rounded-2xl group-hover:scale-110 group-hover:bg-primary/20 transition-all duration-500">
                        <Icon className={cn("w-5 h-5", statusColor || "text-primary")} />
                    </div>
                </div>

                <div className="space-y-1">
                    <p className="text-xs font-bold text-muted-foreground/60 uppercase tracking-widest">{title}</p>
                    <div className="flex items-baseline gap-2">
                        <h3 className={cn("text-3xl lg:text-4xl font-black tracking-tight", statusColor)}>
                            {value}
                        </h3>
                        {subValue && <span className="text-sm font-medium text-muted-foreground/60">{subValue}</span>}
                    </div>
                </div>
            </div>

            {footerLabel && (
                <div className="relative z-10 mt-6 pt-4 border-t border-white/5 flex items-center justify-between text-xs">
                    <span className="text-muted-foreground/60 font-medium uppercase tracking-tighter">{footerLabel}</span>
                    <span className="font-bold px-2.5 py-1 rounded-full bg-white/5 text-foreground/80 border border-white/5">
                        {footerValue}
                    </span>
                </div>
            )}
        </div>
    );
}
