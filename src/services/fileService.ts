import { invoke } from '@tauri-apps/api/core';
import { FileDetails, FileTag } from './fileModel';

export interface FileItem {
    type: number;
    id: string;
    tags: FileTag[];
    name: string;
    created_at: string;
    updated_at: string;
    size: number;
    metadata: Record<string, string>;
    path: string;
    capability: string;
    owned: boolean;
    shared: boolean,
    primary_entity: string;
    thumbnailUrl: string;
}

interface DirectoryItem {
    type: number;
    id: string;
    name: string;
    created_at: string;
    updated_at: string;
    size: number;
    metadata: Record<string, unknown> | null;
    path: string;
    capability: string;
    owned: boolean;
    primary_entity: string;
}

export interface FileSystemResponse {
    files: FileItem[];
    tags: string[];
    file_tags: FileTag[][];
    parent: DirectoryItem;
    pagination: {
        page: number;
        page_size: number;
        is_cursor: boolean;
    };
    props: {
        capability: string;
        max_page_size: number;
        order_by_options: string[] | null;
        order_direction_options: string[] | null;
    };
    context_hint: string;
    mixed_type: boolean;
    storage_policy: {
        id: string;
        name: string;
        type: string;
        max_size: number;
    };
}



export const getFileList = async (path: string, page: number, page_size: number, order_by: string, order: string, category: string = ''): Promise<FileSystemResponse> => {
    try {
        const response = await invoke<FileSystemResponse>('get_files', { path, category, page, pageSize: page_size, orderBy: order_by, order });
        return response;
    } catch (error) {
        console.error('Failed to fetch file list:', error);
        throw error;
    }
};

export const getFileListByCategory = async (path: string, category: string, page: number, page_size: number, order_by: string, order: string): Promise<FileSystemResponse> => {
    try {
        const response = await invoke<FileSystemResponse>('get_files', { path, category, page, pageSize: page_size, orderBy: order_by, order });
        return response;
    } catch (error) {
        console.error('Failed to fetch file list:', error);
        throw error;
    }
};

//  name: String,
//     path: String,
//     page: u64,
//     page_size: u64,
//     order_by: String,
//     order: String,
export const searchFilesByName = async (name: string, path: string, page: number, pageSize: number, orderBy: keyof FileItem, order: string): Promise<FileSystemResponse> => {
    try {
        const response = await invoke<FileSystemResponse>('search_files', { name, path, page, pageSize, orderBy, order });
        return response
    } catch (error) {
        console.error('Failed to fetch file list:', error);
        throw error;
    }
};

export interface StorageInfo {
    total: number,
    used: number,
}

export const getStorageInfo = async (): Promise<StorageInfo> => {
    try {
        const response = await invoke<StorageInfo>('get_storage_info', {});
        return response;
    } catch (error) {
        console.error('Failed to fetch storage info:', error);
        throw error;
    }
};

export const getURL = async (uri: string) => {
    try {
        const response = await invoke<string>('get_url', { uri });
        return response;
    } catch (error) {
        console.error('Failed to fetch file url:', error);
        throw error;
    }
}


export const getThumbURL = async (uri: string) => {
    try {
        const imageUrl = await invoke<string>('get_thumb_url', { uri });
        return imageUrl;
    } catch (error) {
        console.error('Failed to get file thumb url:', error);
        throw error;
    }
}

interface ApplicationOwner {
    type: string
}

interface DeleteFileAck {
    owner: ApplicationOwner,
    path: string,
    token: string,
    type: number,
}
export const deleteFile = async (uri: string, unlink: boolean, softDelete: boolean) => {
    try {
        const response = await invoke<DeleteFileAck>('delete_file', {
            unlink,
            softDelete,
            uri
        });
        return response;
    } catch (error) {
        console.error('Failed to delete file:', error);
        throw error;
    }
}

export const renameFile = async (newName: string, uri: string) => {
    try {
        const response = await invoke<FileItem>('rename_file', { newName, uri });
        return response;
    } catch (error) {
        console.error('Failed to rename file:', error);
        throw error;
    }
}

export const createFolder = async (uri: string) => {
    try {
        const response = await invoke<FileItem>('create_folder', { uri });
        return response;
    } catch (error) {
        console.error('Failed to create folder:', uri, error);
        throw error;
    }
}
export const getFileInfo = async (uri: string) => {
    try {
        const response = await invoke<FileDetails>('get_file_info', { uri });
        return response;
    } catch (error) {
        console.error('Failed to get file details:', uri, error);
        throw error;
    }
}
export const shareFile = async (downloads: number, expire: number, isPrivate: boolean, uri: string) => {
    try {
        const response = await invoke<string>('share_file', { downloads, expire, isPrivate, uri });
        return response;
    } catch (error) {
        console.error('Failed to share file:', uri, error);
        throw error;
    }
}
export const getFileSource = async (uri: string) => {
    try {
        const response = await invoke<string>('get_file_source', { uri });
        return response;
    } catch (error) {
        console.error('Failed to get file source:', uri, error);
        throw error;
    }
}

interface PreDownload {
    id: number,
    url: string;
    file_path: string;
}

export const preDownloadFile = async (uri: string, filename: string) => {
    try {
        const response = await invoke<PreDownload>('pre_download', { uri, filename });
        return response;
    } catch (error) {
        console.error('Failed to download file:', uri, error);
        throw error;
    }
}

interface PreUpload {
    id: number,
    file_path: string,
    filename: string,
    uri: string,
    upload_url: string,
}

export const preUploadFile = async (
    targetPath: string) => {
    try {
        const policyId = localStorage.getItem('policyId');
        if (policyId) {

            const response = await invoke<PreUpload>('pre_upload', { policyId, targetPath });
            return response;
        } else {
            console.error('no policy id');
        }

    } catch (error) {
        console.error('Failed to upload file:', error);
        throw error;
    }
}



export const unlockFile = async (tokens: string[]) => {
    try {
        const response = await invoke<boolean>('unlock_file', { tokens });
        return response;
    } catch (error) {
        console.error('Failed to unlock file:', error);
        throw error;
    }
}
export const restoreFile = async (uri: string) => {
    try {
        const response = await invoke<boolean>('restore_file', { uri });
        return response;
    } catch (error) {
        console.error('Failed to restore file:', error);
        throw error;
    }
}
export const moveFile = async (copy: boolean, dst: string, uri: string) => {
    try {
        const response = await invoke<boolean>('move_file', { copy, dst, uri });
        return response;
    } catch (error) {
        console.error('Failed to move/copy file:', copy, error);
        throw error;
    }
}