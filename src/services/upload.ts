import { invoke } from "@tauri-apps/api/core";

// 定义下载任务类型
export interface DownloadTask {
    id: number;
    filename: string;
    fullPath: string;
    url: string;
    progress: number;
    status: 'pending' | 'completed' | 'downloading' | 'failed' | 'paused';
    size: number;
    startTime?: number;
}

export interface UploadTask {
    id: number;
    filename: string;
    fullPath: string;
    url: string;
    progress: number;
    status: 'pending' | 'completed' | 'uploading' | 'failed' | 'paused';
    size: number;
    startTime?: number;
}


export const uploadFile = async (id: number, url: string, filePath: string) => {
    await invoke<boolean>('upload', { id, url, filePath })
}
export const downloadFile = async (id: number, url: string, filePath: string) => {
    await invoke<number>('download', { id, url, filePath })
}