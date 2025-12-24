import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, X } from 'lucide-react';
import { Button } from './ui/button';
import { cn } from '../lib/utils';
import { useEffect } from 'react';

interface ConfirmationModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    description: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'warning' | 'info';
}

export default function ConfirmationModal({
    isOpen,
    onClose,
    onConfirm,
    title,
    description,
    confirmText = "Confirm",
    cancelText = "Cancel",
    variant = 'danger'
}: ConfirmationModalProps) {
    // Prevent scrolling when modal is open
    useEffect(() => {
        if (isOpen) {
            document.body.style.overflow = 'hidden';
        } else {
            document.body.style.overflow = 'unset';
            // Also remove pointer events from body to prevent clicks behind? No, the overlay handles that.
        }
        return () => {
            document.body.style.overflow = 'unset';
        };
    }, [isOpen]);

    return (
        <AnimatePresence>
            {isOpen && (
                <>
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm"
                        onClick={onClose}
                    />

                    {/* Modal */}
                    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
                        <motion.div
                            initial={{ opacity: 0, scale: 0.95, y: 10 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.95, y: 10 }}
                            transition={{ type: "spring", duration: 0.3 }}
                            className="bg-card w-full max-w-md rounded-xl border shadow-xl overflow-hidden pointer-events-auto"
                        >
                            {/* Header Stripe */}
                            <div className={cn("h-1 w-full",
                                variant === 'danger' ? "bg-red-500" :
                                    variant === 'warning' ? "bg-yellow-500" : "bg-blue-500"
                            )} />

                            <div className="p-6">
                                <div className="flex items-start justify-between mb-4">
                                    <div className="flex items-center gap-3 text-red-500">
                                        <div className={cn("p-2 rounded-full bg-red-100 dark:bg-red-900/20",
                                            variant === 'danger' ? "text-red-600 dark:text-red-400" : "text-foreground"
                                        )}>
                                            <AlertTriangle className="w-6 h-6" />
                                        </div>
                                        <h2 className="text-lg font-semibold text-foreground">{title}</h2>
                                    </div>
                                    <button
                                        onClick={onClose}
                                        className="text-muted-foreground hover:text-foreground transition-colors"
                                    >
                                        <X className="w-5 h-5" />
                                    </button>
                                </div>

                                <p className="text-muted-foreground mb-8 text-sm leading-relaxed">
                                    {description}
                                </p>

                                <div className="flex items-center justify-end gap-3">
                                    <Button variant="ghost" onClick={onClose} className="hover:bg-secondary/50">
                                        {cancelText}
                                    </Button>
                                    <Button
                                        variant={variant === 'danger' ? 'destructive' : 'default'}
                                        onClick={onConfirm}
                                        className="gap-2"
                                    >
                                        {confirmText}
                                    </Button>
                                </div>
                            </div>
                        </motion.div>
                    </div>
                </>
            )}
        </AnimatePresence>
    );
}
