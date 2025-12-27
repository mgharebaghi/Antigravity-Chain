import { useEffect, useState, useRef } from 'react';
import ForceGraph2D, { ForceGraphMethods, NodeObject, LinkObject } from 'react-force-graph-2d';
import { listen } from '@tauri-apps/api/event';
import { Card } from './ui/card';

interface PeerNode extends NodeObject {
    id: string;
    val: number; // Size
    color: string;
}

interface PeerLink extends LinkObject {
    source: string;
    target: string;
}

const NetworkMap = () => {
    const fgRef = useRef<ForceGraphMethods | undefined>(undefined);
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState({ width: 800, height: 400 });
    const [graphData, setGraphData] = useState<{ nodes: PeerNode[]; links: PeerLink[] }>({
        nodes: [],
        links: [],
    });

    useEffect(() => {
        // Responsive resize observer
        if (!containerRef.current) return;
        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                setDimensions({ width, height });
            }
        });
        resizeObserver.observe(containerRef.current);
        return () => resizeObserver.disconnect();
    }, []);

    useEffect(() => {
        // Listen for topology updates from Rust
        const unlisten = listen('network-topology-update', (event) => {
            const rawGraph = event.payload as Record<string, string[]>;

            const newNodes: PeerNode[] = [];
            const newLinks: PeerLink[] = [];
            const nodeSet = new Set<string>();

            Object.entries(rawGraph).forEach(([source, targets]) => {
                if (!nodeSet.has(source)) {
                    nodeSet.add(source);
                    newNodes.push({ id: source, val: 5, color: '#10b981' }); // Emerald-500
                }
                targets.forEach((target) => {
                    if (!nodeSet.has(target)) {
                        nodeSet.add(target);
                        newNodes.push({ id: target, val: 3, color: '#3b82f6' }); // Blue-500
                    }
                    newLinks.push({ source, target });
                });
            });

            setGraphData({ nodes: newNodes, links: newLinks });
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    return (
        <Card ref={containerRef} className="w-full h-[400px] glass-card border-primary/20 bg-primary/5 relative overflow-hidden flex items-center justify-center p-0">
            <div className="absolute top-0 right-0 w-64 h-64 bg-primary/10 rounded-full blur-3xl -mr-20 -mt-20 pointer-events-none" />
            <div className="absolute bottom-0 left-0 w-64 h-64 bg-emerald-500/10 rounded-full blur-3xl -ml-20 -mb-20 pointer-events-none" />

            <div className="absolute top-4 left-6 z-10 pointer-events-none">
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    <h2 className="text-sm font-bold text-foreground/80 tracking-wider uppercase">Topology Graph</h2>
                </div>
            </div>

            <ForceGraph2D
                ref={fgRef}
                width={dimensions.width}
                height={dimensions.height}
                graphData={graphData}
                backgroundColor="rgba(0,0,0,0)" // Transparent to let glass card show
                nodeLabel="id"
                nodeColor="color"
                linkColor={() => 'rgba(255,255,255,0.1)'}
                linkWidth={1}
                linkDirectionalParticles={2}
                linkDirectionalParticleSpeed={0.005}
                linkDirectionalParticleWidth={2}
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
            />
        </Card>
    );
};

export default NetworkMap;
