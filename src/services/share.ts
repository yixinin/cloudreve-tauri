import { invoke } from "@tauri-apps/api/core";

export interface UserOwner {
    id: string;
    nickname: string;
    email: string;
    createdAt: string;
}

export interface ShareItem {
    id: string;
    name: string;
    expired: boolean;
    source_type: number;
    unlocked: boolean;
    url: string;
    visited: number;
    created_at: string;
    owner: UserOwner;
}

export interface Pagination {
    total: number;
    page: number;
    pageSize: number;
}

export interface GetSharesAck {
    shares: ShareItem[];
    pagination: Pagination;
}


export interface ShareOwner {
    id: string;
    email: string;
    nickname: string;
    created_at: Date | string;
}

export interface ShareInfo {
    id: string;
    name: string;
    visited: number;
    unlocked: boolean;
    source_type: number;
    owner: ShareOwner;
    created_at: Date | string;
    expired: boolean;
    url: string;
    source_uri: string;
}


export const deleteShare = async (id: string) => {
    try {
        const response = await invoke<boolean>('delete_share', { id });
        return response;
    } catch (error) {
        console.error('Failed to delete share:', id, error);
        throw error;
    }
}


export const updateShare = async (id: string, downloads: number, expire: number, uri: string) => {
    try {
        const response = await invoke<string>('update_share', { id, downloads, expire, uri });
        return response;
    } catch (error) {
        console.error('Failed to delete share:', id, error);
        throw error;
    }
}

export const getShareInfo = async (id: string, ownerExtended: boolean) => {
    try {
        const response = await invoke<ShareInfo>('get_share_info', { id, ownerExtended });
        return response;
    } catch (error) {
        console.error('Failed to get share info:', id, error);
        throw error;
    }
}