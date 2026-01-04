// Common types for the application

export interface Block {
    index: number;
    hash: string;
    prev_hash: string;
    timestamp: number;
    transactions: Transaction[];
    author: string;
    vdf_proof: any; // Ideally more specific
    shard_id: number;
}

export interface Transaction {
    id: string;
    sender: string;
    receiver: string;
    amount: number;
    signature: any;
    timestamp: number;
    sender_bump: number;
}
