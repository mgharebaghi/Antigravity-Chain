import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

export const WelcomeAnimation = () => {
    const [show, setShow] = useState(false);
    const [step, setStep] = useState(0);

    useEffect(() => {
        const hasSeen = localStorage.getItem('hasSeenWelcome');
        if (!hasSeen) {
            setShow(true);
            // Sequence
            setTimeout(() => setStep(1), 1000); // "Initializing"
            setTimeout(() => setStep(2), 3500); // "Connecting"
            setTimeout(() => setStep(3), 6000); // "Ready"
        }
    }, []);

    const handleComplete = () => {
        localStorage.setItem('hasSeenWelcome', 'true');
        setShow(false);
    };

    if (!show) return null;

    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-[#0a0f1c] text-white"
                style={{ backdropFilter: 'blur(20px)' }}
            >
                <motion.div
                    animate={{
                        scale: [1, 1.2, 1],
                        rotate: [0, 360],
                        borderRadius: ["20%", "50%", "20%"]
                    }}
                    transition={{ duration: 3, ease: "easeInOut", repeat: Infinity, repeatDelay: 1 }}
                    className="w-32 h-32 mb-10 border-4 border-cyan-500 shadow-[0_0_50px_rgba(6,182,212,0.5)]"
                />

                <motion.h1
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    className="text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-purple-600 mb-4"
                >
                    Welcome to Antigravity
                </motion.h1>

                <div className="h-8 flex flex-col items-center">
                    <AnimatePresence mode="wait">
                        {step === 0 && (
                            <motion.p key="s0" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="text-gray-400">
                                Initializing Secure Environment...
                            </motion.p>
                        )}
                        {step === 1 && (
                            <motion.p key="s1" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="text-cyan-400">
                                Generating Cryptographic Keys...
                            </motion.p>
                        )}
                        {step === 2 && (
                            <motion.p key="s2" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="text-purple-400">
                                Connecting to Isolated Network...
                            </motion.p>
                        )}
                        {step === 3 && (
                            <motion.button
                                key="s3"
                                initial={{ scale: 0.8, opacity: 0 }}
                                animate={{ scale: 1, opacity: 1 }}
                                whileHover={{ scale: 1.05 }}
                                whileTap={{ scale: 0.95 }}
                                onClick={handleComplete}
                                className="px-8 py-3 bg-gradient-to-r from-cyan-600 to-blue-600 rounded-full font-bold shadow-lg shadow-cyan-500/30"
                            >
                                Enter The Void
                            </motion.button>
                        )}
                    </AnimatePresence>
                </div>

                {step < 3 && (
                    <motion.div
                        className="mt-8 w-64 h-1 bg-gray-800 rounded-full overflow-hidden"
                    >
                        <motion.div
                            className="h-full bg-cyan-500"
                            initial={{ width: "0%" }}
                            animate={{ width: "100%" }}
                            transition={{ duration: 6, ease: "linear" }}
                        />
                    </motion.div>
                )}

            </motion.div>
        </AnimatePresence>
    );
};
