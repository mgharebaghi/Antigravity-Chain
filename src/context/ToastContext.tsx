import React, { createContext, useContext, useState, useCallback } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { X, CheckCircle, AlertCircle, Info } from 'lucide-react';
import { cn } from '../lib/utils';

export type ToastType = 'success' | 'error' | 'info';

export interface Toast {
    id: string;
    message: string;
    type: ToastType;
}

interface ToastContextType {
    toasts: Toast[];
    addToast: (message: string, type: ToastType) => void;
    removeToast: (id: string) => void;
    success: (message: string) => void;
    error: (message: string) => void;
    info: (message: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function useToast() {
    const context = useContext(ToastContext);
    if (!context) {
        throw new Error('useToast must be used within a ToastProvider');
    }
    return context;
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([]);

    const removeToast = useCallback((id: string) => {
        setToasts((prev) => prev.filter((toast) => toast.id !== id));
    }, []);

    const addToast = useCallback((message: string, type: ToastType) => {
        const id = Math.random().toString(36).substring(2, 9);
        setToasts((prev) => [...prev, { id, message, type }]);

        // Auto remove after 5 seconds
        setTimeout(() => {
            removeToast(id);
        }, 5000);
    }, [removeToast]);

    const success = (message: string) => addToast(message, 'success');
    const error = (message: string) => addToast(message, 'error');
    const info = (message: string) => addToast(message, 'info');

    return (
        <ToastContext.Provider value={{ toasts, addToast, removeToast, success, error, info }}>
            {children}
            <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none p-4 max-w-sm w-full">
                <AnimatePresence mode="popLayout">
                    {toasts.map((toast) => (
                        <ToastItem key={toast.id} toast={toast} onClose={() => removeToast(toast.id)} />
                    ))}
                </AnimatePresence>
            </div>
        </ToastContext.Provider>
    );
}

function ToastItem({ toast, onClose }: { toast: Toast; onClose: () => void }) {
    const icons = {
        success: <CheckCircle className="w-5 h-5 text-white" />,
        error: <AlertCircle className="w-5 h-5 text-destructive-foreground" />,
        info: <Info className="w-5 h-5 text-blue-500" />
    };

    const styles = {
        success: "bg-green-600/90 border-green-600 text-white shadow-sm", // High contrast: solid green background with white text
        error: "bg-destructive/90 border-destructive text-destructive-foreground shadow-sm", // High contrast: solid red background with white text
        info: "bg-blue-500/10 border-blue-500/20 text-blue-600 dark:text-blue-400"
    };

    return (
        <motion.div
            layout
            initial={{ opacity: 0, y: 50, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9, transition: { duration: 0.2 } }}
            className={cn(
                "pointer-events-auto flex items-start gap-3 p-4 rounded-xl border shadow-lg backdrop-blur-md bg-background/95",
                styles[toast.type]
            )}
        >
            <div className="shrink-0 mt-0.5">{icons[toast.type]}</div>
            <p className="text-sm font-medium leading-tight flex-1">{toast.message}</p>
            <button
                onClick={onClose}
                className="shrink-0 rounded-md p-1 hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
            >
                <X className="w-4 h-4 opacity-50" />
            </button>
        </motion.div>
    );
}
