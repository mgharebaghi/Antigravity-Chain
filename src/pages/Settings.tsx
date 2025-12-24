import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../components/ui/card";
import { Button } from "../components/ui/button";
import PageTransition from "../components/PageTransition";
import { Settings as SettingsIcon, Server, AlertTriangle, Save, Cpu } from "lucide-react";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../context/ToastContext";
import WipeDataModal from "../components/WipeDataModal";

interface AppSettings {
    node_name: string;
    relay_address: string;
    mining_enabled: boolean;
    max_peers: number;
    node_type: "Full" | "Pruned";
}

export default function Settings() {
    const { success, error } = useToast();
    const [settings, setSettings] = useState<AppSettings>({
        node_name: "Antigravity-Node-01",
        relay_address: "/ip4/127.0.0.1/tcp/9090",
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
            // Note: In a real app, some settings might require a restart
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
                window.location.reload(); // Reload to refresh all state
            }, 1000);
        } catch (err) {
            error("Failed to reset: " + err);
        }
    };

    if (loading) return <div className="p-8">Loading settings...</div>;

    return (
        <PageTransition>
            <div className="flex flex-col min-h-full gap-3 sm:gap-6 container mx-auto p-4 sm:p-6 lg:max-w-6xl pb-10">
                <div className="flex items-center justify-between shrink-0 px-1">
                    <div className="flex items-center gap-2 sm:gap-3">
                        <div className="p-1.5 sm:p-2 bg-primary/10 rounded-lg">
                            <SettingsIcon className="w-5 h-5 sm:w-6 h-6 text-primary" />
                        </div>
                        <div>
                            <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-foreground">Cloud Node Config</h1>
                            <p className="text-[10px] sm:text-sm text-muted-foreground font-medium">Fine-tune your mesh participant settings</p>
                        </div>
                    </div>
                    <Button className="gap-2 shadow-lg hover:shadow-primary/20 h-9 sm:h-11 px-3 sm:px-6 rounded-lg sm:rounded-xl text-[10px] sm:text-sm font-bold" onClick={handleSave}>
                        <Save className="w-3.5 h-3.5 sm:w-4 h-4" /> <span className="hidden xs:inline">Save Configuration</span><span className="xs:hidden">Save</span>
                    </Button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-6 pb-2 sm:pb-4 px-1">
                    {/* General Settings */}
                    <Card className="border-primary/10 bg-gradient-to-br from-card to-background">
                        <CardHeader>
                            <CardTitle className="text-lg">General</CardTitle>
                            <CardDescription>Node Identity & Identification</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="space-y-2">
                                <label className="text-sm font-medium">Node Name</label>
                                <input
                                    type="text"
                                    value={settings.node_name}
                                    onChange={(e) => setSettings({ ...settings, node_name: e.target.value })}
                                    className="w-full h-11 px-3 py-2 bg-muted/50 rounded-xl border border-input focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono"
                                    placeholder="e.g. My-Cool-Node"
                                />
                            </div>
                        </CardContent>
                    </Card>

                    {/* Network Settings */}
                    <Card className="border-primary/10 bg-gradient-to-br from-card to-background">
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2 text-lg">
                                <Server className="w-5 h-5 text-primary" /> Network
                            </CardTitle>
                            <CardDescription>Connectivity & P2P configuration</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="space-y-2">
                                <label className="text-sm font-medium">Relay Address (Multiaddress)</label>
                                <input
                                    type="text"
                                    value={settings.relay_address}
                                    onChange={(e) => setSettings({ ...settings, relay_address: e.target.value })}
                                    className="w-full h-11 px-3 py-2 bg-muted/50 rounded-xl border border-input focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono text-xs"
                                    placeholder="/ip4/127.0.0.1/tcp/9090"
                                />
                            </div>
                            <div className="space-y-2">
                                <div className="flex justify-between">
                                    <label className="text-sm font-medium">Max Peers</label>
                                    <span className="text-sm font-bold text-primary">{settings.max_peers}</span>
                                </div>
                                <input
                                    type="range"
                                    min="1"
                                    max="200"
                                    value={settings.max_peers}
                                    onChange={(e) => setSettings({ ...settings, max_peers: parseInt(e.target.value) })}
                                    className="w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary"
                                />
                            </div>
                        </CardContent>
                    </Card>

                    {/* Mining Settings */}
                    <Card className="border-primary/10 bg-gradient-to-br from-card to-background">
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2 text-lg">
                                <Cpu className="w-5 h-5 text-primary" /> Mining & Consensus
                            </CardTitle>
                            <CardDescription>Block production settings</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center justify-between p-4 rounded-xl bg-muted/30 border border-border/50">
                                <div>
                                    <p className="font-semibold text-sm">Enable Block Production</p>
                                    <p className="text-xs text-muted-foreground">Allow this node to mine blocks</p>
                                </div>
                                <div
                                    className={`w-12 h-6 rounded-full p-1 cursor-pointer transition-colors ${settings.mining_enabled ? 'bg-primary' : 'bg-muted'}`}
                                    onClick={() => setSettings({ ...settings, mining_enabled: !settings.mining_enabled })}
                                >
                                    <div className={`w-4 h-4 bg-white rounded-full transition-transform ${settings.mining_enabled ? 'translate-x-6' : ''}`} />
                                </div>
                            </div>

                            <div className="space-y-3">
                                <p className="font-semibold text-sm px-1">Node Type (Scalability)</p>
                                <div className="grid grid-cols-2 gap-3">
                                    <button
                                        onClick={() => setSettings({ ...settings, node_type: "Pruned" })}
                                        className={`flex flex-col items-center justify-center p-4 rounded-xl border-2 transition-all ${settings.node_type === "Pruned" ? 'border-primary bg-primary/5 text-primary shadow-inner' : 'border-border/50 hover:bg-muted/30 text-muted-foreground'}`}
                                    >
                                        <span className="font-bold text-sm">Pruned Node</span>
                                        <span className="text-[10px] opacity-70">Saves Storage (Consumer)</span>
                                    </button>
                                    <button
                                        onClick={() => setSettings({ ...settings, node_type: "Full" })}
                                        className={`flex flex-col items-center justify-center p-4 rounded-xl border-2 transition-all ${settings.node_type === "Full" ? 'border-primary bg-primary/5 text-primary shadow-inner' : 'border-border/50 hover:bg-muted/30 text-muted-foreground'}`}
                                    >
                                        <span className="font-bold text-sm">Full Node</span>
                                        <span className="text-[10px] opacity-70">Stores History (Archive)</span>
                                    </button>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Storage & Danger Zone */}
                    <Card className="border-destructive/30 bg-destructive/5">
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2 text-lg text-destructive">
                                <AlertTriangle className="w-5 h-5" /> Danger Zone
                            </CardTitle>
                            <CardDescription>Critical & Irreversible actions</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center justify-between p-4 rounded-xl bg-background/50 border border-destructive/20 gap-4">
                                <div className="min-w-0 flex-1">
                                    <p className="font-semibold text-sm">Reset Blockchain Data</p>
                                    <p className="text-xs text-muted-foreground">Wipes local blocks and restarts sync</p>
                                </div>
                                <Button variant="destructive" size="sm" onClick={handleReset} className="shrink-0 font-bold">
                                    Wipe Data
                                </Button>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>
            <WipeDataModal
                isOpen={isWipeModalOpen}
                onClose={() => setIsWipeModalOpen(false)}
                onConfirm={confirmReset}
            />
        </PageTransition>
    );
}
