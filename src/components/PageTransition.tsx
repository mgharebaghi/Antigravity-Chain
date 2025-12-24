import { motion } from 'framer-motion';
import { ReactNode } from 'react';

export default function PageTransition({ children }: { children: ReactNode }) {
    return (
        <motion.div
            initial={{ opacity: 0, y: 15, scale: 0.98, filter: 'blur(10px)' }}
            animate={{ opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }}
            exit={{ opacity: 0, y: -15, scale: 0.98, filter: 'blur(10px)' }}
            transition={{ duration: 0.4, ease: [0.19, 1, 0.22, 1] }}
            className="w-full h-full"
        >
            {children}
        </motion.div>
    );
}
