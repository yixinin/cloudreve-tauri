

export interface FileDetails {
    type: number;
    id: string;
    name: string;
    created_at: string;
    updated_at: string;
    size: number;
    metadata: Record<string, never>;
    path: string;
    capability: string;
    owned: boolean;
    primary_entity: string;
    extended_info: ExtendedInfo;
}


export interface FileTag {
    key: string;
    color: string;
}
interface CreatedBy {
    id: string;
    nickname: string;
    created_at: string;
}

interface StoragePolicy {
    id: string;
    name: string;
    type: string;
    max_size: number;
    relay: boolean;
}

interface Entity {
    id: string;
    size: number;
    type: number;
    created_at: string;
    storage_policy: StoragePolicy;
    created_by: CreatedBy;
}

interface ExtendedInfo {
    storage_policy: StoragePolicy;
    storage_used: number;
    entities: Entity[];
}

