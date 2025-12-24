import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';

// Types
export interface WalletInfo {
    address: string;
    alias?: string;
    balance?: number;
}

export interface Transaction {
    id: string;
    sender: string;
    receiver: string;
    amount: number;
    timestamp: number;
    signature: string;
}

export interface Block {
    index: number;
    timestamp: number;
    author: string;
    transactions: Transaction[];
    previous_hash: string;
    hash: string;
    start_time_weight: number;
    vdf_proof?: string;
    signature?: string;
}

export interface VdfStatus {
    iterations_per_second: number;
    difficulty: number;
    is_active: boolean;
}

export interface WalletExport {
    address: string;
    private_key: string;
    mnemonic: string;
}

interface AppContextType {
    wallet: WalletInfo | null;
    nodeStatus: string;
    relayStatus: string;
    patience: number;
    peers: number;
    loading: boolean;
    latestBlock: Block | null;
    recentBlocks: Block[];
    minedBlocks: number;
    totalBlocks: number;
    height: number;
    refreshWallet: () => Promise<void>;
    refreshBlockHeight: () => Promise<void>;
    connectedRelay: string | null;
    vdfStatus: VdfStatus | null;
    startNode: () => Promise<void>;
    stopNode: () => Promise<void>;
    logout: () => Promise<void>;
    exitApp: () => Promise<void>;
    importWallet: (privateKey: string) => Promise<string>;
    createWallet: () => Promise<WalletExport>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
    const [wallet, setWallet] = useState<WalletInfo | null>(null);
    const [nodeStatus, setNodeStatus] = useState<string>("Stopped");
    const [relayStatus, setRelayStatus] = useState<string>("Disconnected");
    const [connectedRelay, setConnectedRelay] = useState<string | null>(null);
    const [patience, setPatience] = useState<number>(0);
    const [peers, setPeers] = useState<number>(0);
    const [loading, setLoading] = useState<boolean>(true);
    const [latestBlock, setLatestBlock] = useState<Block | null>(null);
    const [recentBlocks, setRecentBlocks] = useState<Block[]>([]);
    const [minedBlocks, setMinedBlocks] = useState<number>(0);
    const [totalBlocks, setTotalBlocks] = useState<number>(0);
    const [height, setHeight] = useState<number>(0);
    const [vdfStatus, setVdfStatus] = useState<VdfStatus | null>(null);

    // Initial load
    useEffect(() => {
        refreshWallet();
        refreshBlockHeight();

        // Listeners
        const unlistenNode = listen('node-status', (event: any) => {
            console.log("Node Status Event:", event.payload);
            setNodeStatus(event.payload);
        });

        const unlistenRelay = listen('relay-status', (event: any) => {
            console.log("Relay Status Event:", event.payload);
            setRelayStatus(event.payload);
            if (event.payload.toLowerCase() === 'disconnected') {
                setConnectedRelay(null);
            }
        });

        const unlistenRelayInfo = listen('relay-info', (event: any) => {
            console.log("Relay Info Event:", event.payload);
            setConnectedRelay(event.payload);
        });

        const unlistenPeerCount = listen('peer-count', (event: any) => {
            setPeers(event.payload);
        });

        const unlistenNewBlock = listen('new-block', (event: any) => {
            console.log("New Block Event:", event.payload);
            const block = event.payload as Block;
            setLatestBlock(block);
            setRecentBlocks(prev => [block, ...prev].slice(0, 100));
            setTotalBlocks(prev => {
                // If it's the genesis we might have 0->1, or if it's new it's index + 1
                return Math.max(prev + 1, block.index + 1);
            });
            setHeight(block.index);
            // Refresh wallet balance and height/mining stats on new block!
            refreshWallet();
            refreshBlockHeight();
        });

        const unlistenVdf = listen('vdf-status', (event: any) => {
            setVdfStatus(event.payload as VdfStatus);
        });

        // Cleanup
        return () => {
            unlistenNode.then(f => f());
            unlistenRelay.then(f => f());
            unlistenRelayInfo.then(f => f());
            unlistenPeerCount.then(f => f());
            unlistenNewBlock.then(f => f());
            unlistenVdf.then(f => f());
        };
    }, []);

    // Timer for patience/peers when node is running
    useEffect(() => {
        let interval: ReturnType<typeof setInterval>;
        const runningStates = ['Active', 'Connected', 'Syncing'];
        if (runningStates.some(s => nodeStatus.includes(s))) {
            interval = setInterval(() => {
                setPatience(prev => prev + 1);
                // Also poll block height here periodically
                refreshBlockHeight();
            }, 1000);
        }
        return () => clearInterval(interval);
    }, [nodeStatus]);

    const refreshWallet = async () => {
        try {
            const w = await invoke<WalletInfo | null>("get_wallet_info");
            setWallet(w);
        } catch (e) {
            console.error("Failed to fetch wallet info", e);
        } finally {
            setLoading(false);
        }
    };

    const refreshBlockHeight = async () => {
        try {
            // Fetch chain stats
            const stats = await invoke<{ total_blocks: number, height: number }>("get_chain_stats");
            setTotalBlocks(stats.total_blocks);
            setHeight(stats.height);

            // Fetch more blocks for explorer (e.g. 100)
            const blocks = await invoke<Block[]>("get_recent_blocks", { limit: 100 });
            if (blocks.length > 0) {
                setLatestBlock(blocks[0]);
                setRecentBlocks(blocks);
            }

            // Also fetch mined blocks (by THIS node)
            const count = await invoke<number>("get_mined_blocks_count");
            console.log("AppContext: Current mined blocks count from backend:", count);
            setMinedBlocks(count);
        } catch (e) {
            console.error("Failed to fetch block info", e);
        }
    };

    const startNode = async () => {
        if (!wallet) throw new Error("Wallet required");

        // If restarting, ensure stopped first locally
        if (nodeStatus === "Running" || nodeStatus === "Connecting") {
            // Optimistic stop
            setNodeStatus("Stopped");
        }

        setNodeStatus("Connecting");
        setRelayStatus("Connecting...");
        setConnectedRelay(null);

        try {
            await invoke("start_node");
            // We rely on events to update status to "Active" or "Syncing"
        } catch (e: any) {
            console.error("Failed to start node", e);
            setNodeStatus("Stopped");
            // Backend might return string error
            throw new Error(e.toString());
        }
    };

    const stopNode = async () => {
        try {
            await invoke("stop_node");
            setNodeStatus("Stopped");
            setRelayStatus("Disconnected");
            setConnectedRelay(null);
            setPeers(0);
        } catch (e) {
            console.error("Failed to stop node", e);
        }
    };

    const logout = async () => {
        try {
            await stopNode();
            await invoke("logout_wallet");
            setWallet(null);
            setMinedBlocks(0);
        } catch (e) {
            console.error("Failed to logout wallet", e);
        }
    };

    // Tauri exit app
    const exitApp = async () => {
        try {
            // clean up first
            await invoke("stop_node");
        } catch (e) {
            console.warn("Stop node failed during exit", e);
        }
        // Exit
        await invoke("exit_app");
    };

    const createWallet = async (): Promise<WalletExport> => {
        setLoading(true);
        try {
            const exportData = await invoke<WalletExport>("create_wallet");
            await refreshWallet();
            return exportData;
        } finally {
            setLoading(false);
        }
    };

    const importWallet = async (privateKey: string): Promise<string> => {
        setLoading(true);
        try {
            const address = await invoke<string>("import_wallet", { privateKeyHex: privateKey });
            await refreshWallet();
            return address;
        } finally {
            setLoading(false);
        }
    };

    return (
        <AppContext.Provider value={{
            wallet,
            nodeStatus,
            relayStatus,
            connectedRelay,
            patience,
            peers,
            loading,
            latestBlock,
            recentBlocks,
            minedBlocks,
            totalBlocks,
            height,
            vdfStatus,
            refreshWallet,
            refreshBlockHeight,
            startNode,
            stopNode,
            logout,
            exitApp, // Expose exit
            createWallet,
            importWallet
        }}>
            {children}
        </AppContext.Provider>

    );
}

export function useApp() {
    const context = useContext(AppContext);
    if (context === undefined) {
        throw new Error('useApp must be used within an AppProvider');
    }
    return context;
}
