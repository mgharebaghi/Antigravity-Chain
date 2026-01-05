// import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
// import { Badge } from "../components/ui/badge.tsx";
import {
    Settings as SettingsIcon,
    AlertTriangle,
    Save,
    Cpu,
    Globe,
    // Shield,
    Database,
    Zap,
    Fingerprint,
    Trash2
} from "lucide-react";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../context/ToastContext";
import WipeDataModal from "../components/modals/WipeDataModal";
import { cn } from "../lib/utils";
// import { useTheme } from '../context/ThemeContext';

interface AppSettings {
    node_name: string;
    relay_addresses: string[];
    allow_relay_free_mode: boolean;
    mining_enabled: boolean;
    max_peers: number;
    node_type: "Full" | "Pruned";
}

export default function Settings() {
    const { success, error } = useToast();
    // const { theme } = useTheme();
    const [settings, setSettings] = useState<AppSettings>({
        node_name: "Centichain-Node-01",
        relay_addresses: ["/ip4/127.0.0.1/tcp/9090"],
        allow_relay_free_mode: true,
        mining_enabled: true,
        max_peers: 50,
        node_type: "Pruned",
    });
    const [loading, setLoading] = useState(true);
    const [isWipeModalOpen, setIsWipeModalOpen] = useState(false);

    useEffect(() => {
        loadSettings();
    }, []);

    const loadSettings = async () => {
        try {
            const data = await invoke<AppSettings>("get_app_settings");
            setSettings(data);
        } catch (err) {
            error("Failed to load settings: " + err);
        } finally {
            setLoading(false);
        }
    };

    const handleSave = async () => {
        try {
            await invoke("save_app_settings", { settings });
            success("Settings saved successfully!");
        } catch (err) {
            error("Failed to save settings: " + err);
        }
    };

    const handleReset = async () => {
        setIsWipeModalOpen(true);
    };

    const confirmReset = async () => {
        setIsWipeModalOpen(false);
        try {
            await invoke("reset_chain_data");
            success("Chain data reset successfully.");
            setTimeout(() => {
                window.location.reload();
            }, 1000);
        } catch (err) {
            error("Failed to reset: " + err);
        }
    };

    if (loading) return (
        <div className="flex h-[300px] flex-col items-center justify-center gap-4 text-muted-foreground">
            <SettingsIcon className="w-10 h-10 animate-spin opacity-20" />
            <p className="text-sm">Loading Configuration...</p>
        </div>
    );

    return (
        <div className="flex flex-col gap-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
                    <p className="text-muted-foreground mt-1 text-sm">Configure your node's performance and behavior.</p>
                </div>
                <Button onClick={handleSave} className="gap-2">
                    <Save className="w-4 h-4" /> Save Changes
                </Button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">

                {/* Node Identity Card */}
                <div className="nebula-card p-6 flex flex-col gap-6">
                    <div className="flex items-center gap-3 border-b border-border pb-4">
                        <div className="p-2 bg-primary/10 text-primary rounded-lg">
                            <Fingerprint className="w-5 h-5" />
                        </div>
                        <h3 className="font-semibold text-lg">Identity</h3>
                    </div>

                    <div className="space-y-4">
                        <div className="space-y-2">
                            <label className="text-xs font-semibold uppercase text-muted-foreground">Node Alias</label>
                            <input
                                type="text"
                                value={settings.node_name}
                                onChange={(e) => setSettings({ ...settings, node_name: e.target.value })}
                                className="w-full px-3 py-2 rounded-md border border-input bg-background/50 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                                placeholder="Enter Node Name"
                            />
                            <p className="text-[10px] text-muted-foreground">Broadcasted to peers during handshake.</p>
                        </div>
                    </div>
                </div>

                {/* Connectivity Card */}
                <div className="nebula-card p-6 flex flex-col gap-6">
                    <div className="flex items-center gap-3 border-b border-border pb-4">
                        <div className="p-2 bg-emerald-500/10 text-emerald-500 rounded-lg">
                            <Globe className="w-5 h-5" />
                        </div>
                        <h3 className="font-semibold text-lg">Network</h3>
                    </div>

                    <div className="space-y-4">
                        <div className="space-y-2">
                            <label className="text-xs font-semibold uppercase text-muted-foreground flex justify-between">
                                Relay Address
                                <span className="text-[10px] text-orange-500 font-bold">(Requires Node Restart)</span>
                            </label>
                            <input
                                type="text"
                                value={settings.relay_addresses[0] || ""}
                                onChange={(e) => setSettings({ ...settings, relay_addresses: [e.target.value] })}
                                className="w-full px-3 py-2 rounded-md border border-input bg-background/50 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-ring"
                                placeholder="/ip4/127.0.0.1/tcp/9090"
                            />
                        </div>
                        <div className="space-y-3">
                            <div className="flex justify-between items-center">
                                <label className="text-xs font-semibold uppercase text-muted-foreground">Max Peers</label>
                                <span className="text-xs font-mono font-bold">{settings.max_peers}</span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="200"
                                value={settings.max_peers}
                                onChange={(e) => setSettings({ ...settings, max_peers: parseInt(e.target.value) })}
                                className="w-full h-1.5 bg-secondary rounded-full appearance-none cursor-pointer accent-primary"
                            />
                        </div>

                        <div
                            className="flex items-center justify-between p-3 rounded-lg border border-border bg-card cursor-pointer hover:bg-muted/50 transition-colors"
                            onClick={() => setSettings({ ...settings, allow_relay_free_mode: !settings.allow_relay_free_mode })}
                        >
                            <div className="space-y-0.5">
                                <div className="text-sm font-medium">Relay-free Mode</div>
                                <div className="text-xs text-muted-foreground">Allow operation without a relay</div>
                            </div>
                            <div className={cn(
                                "h-5 w-9 rounded-full relative transition-colors duration-200",
                                settings.allow_relay_free_mode ? "bg-primary" : "bg-muted-foreground/30"
                            )}>
                                <div className={cn(
                                    "h-4 w-4 bg-background rounded-full absolute top-0.5 transition-all duration-200 shadow-sm",
                                    settings.allow_relay_free_mode ? "left-[18px]" : "left-0.5"
                                )} />
                            </div>
                        </div>
                    </div>
                </div>

                {/* Consensus Card */}
                <div className="nebula-card p-6 flex flex-col gap-6">
                    <div className="flex items-center gap-3 border-b border-border pb-4">
                        <div className="p-2 bg-orange-500/10 text-orange-500 rounded-lg">
                            <Cpu className="w-5 h-5" />
                        </div>
                        <h3 className="font-semibold text-lg">Consensus</h3>
                    </div>

                    <div className="space-y-4">
                        <div
                            className="flex items-center justify-between p-3 rounded-lg border border-border bg-card cursor-pointer hover:bg-muted/50 transition-colors"
                            onClick={() => setSettings({ ...settings, mining_enabled: !settings.mining_enabled })}
                        >
                            <div className="space-y-0.5">
                                <div className="text-sm font-medium">Mining Enabled</div>
                                <div className="text-xs text-muted-foreground">Participate in block production</div>
                            </div>
                            <div className={cn(
                                "h-5 w-9 rounded-full relative transition-colors duration-200",
                                settings.mining_enabled ? "bg-primary" : "bg-muted-foreground/30"
                            )}>
                                <div className={cn(
                                    "h-4 w-4 bg-background rounded-full absolute top-0.5 transition-all duration-200 shadow-sm",
                                    settings.mining_enabled ? "left-[18px]" : "left-0.5"
                                )} />
                            </div>
                        </div>

                        <div className="space-y-2">
                            <label className="text-xs font-semibold uppercase text-muted-foreground">Storage Mode</label>
                            <div className="grid grid-cols-2 gap-2">
                                <NodeOption
                                    active={settings.node_type === "Pruned"}
                                    onClick={() => setSettings({ ...settings, node_type: "Pruned" })}
                                    label="Pruned"
                                    icon={<Zap className="w-3 h-3" />}
                                />
                                <NodeOption
                                    active={settings.node_type === "Full"}
                                    onClick={() => setSettings({ ...settings, node_type: "Full" })}
                                    label="Full"
                                    icon={<Database className="w-3 h-3" />}
                                />
                            </div>
                        </div>
                    </div>
                </div>

                {/* Danger Zone */}
                <div className="md:col-span-2 lg:col-span-3 nebula-card p-6 border-red-500/20 bg-red-500/5">
                    <div className="flex items-center justify-between flex-wrap gap-4">
                        <div className="flex items-center gap-4">
                            <div className="p-2 bg-red-500/10 text-red-500 rounded-lg">
                                <AlertTriangle className="w-6 h-6" />
                            </div>
                            <div>
                                <h3 className="font-bold text-red-600 dark:text-red-400">Danger Zone</h3>
                                <p className="text-sm text-red-600/80 dark:text-red-400/80">Irreversible actions that affect your node's data.</p>
                            </div>
                        </div>
                        <Button variant="destructive" onClick={handleReset} className="gap-2">
                            <Trash2 className="w-4 h-4" /> Wipe Chain Data
                        </Button>
                    </div>
                </div>
            </div>

            <WipeDataModal
                isOpen={isWipeModalOpen}
                onClose={() => setIsWipeModalOpen(false)}
                onConfirm={confirmReset}
            />
        </div>
    );
}

function NodeOption({ active, onClick, label, icon }: { active: boolean, onClick: () => void, label: string, icon: any }) {
    return (
        <button
            onClick={onClick}
            className={cn(
                "flex items-center justify-center gap-2 p-2 rounded-md border text-xs font-medium transition-all",
                active
                    ? "bg-primary text-primary-foreground border-primary"
                    : "bg-background border-input hover:bg-muted text-foreground"
            )}
        >
            {icon}
            {label}
        </button>
    );
}
