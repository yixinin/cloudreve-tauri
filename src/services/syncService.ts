import { invoke } from '@tauri-apps/api/core';

// 同步状态接口定义
export interface SyncStatus {
  enabled: boolean;
  lastSync?: string;
  syncedFiles?: number;
}

// 同步进度接口定义
export interface SyncProgress {
  percentage: number;
  synced: number;
  total: number;
  status: 'idle' | 'syncing' | 'completed' | 'error';
}

// 获取同步状态
export const getSyncStatus = async (): Promise<SyncStatus> => {
  try {
    return await invoke<SyncStatus>('plugin:cloudreve|get_sync_status');
  } catch (error) {
    console.error('Failed to get sync status:', error);
    return { enabled: false };
  }
};

// 切换同步开关
export const toggleSync = async (enabled: boolean): Promise<boolean> => {
  try {
    return await invoke<boolean>('plugin:cloudreve|toggle_sync', { enabled });
  } catch (error) {
    console.error('Failed to toggle sync:', error);
    return false;
  }
};

// 获取同步进度
export const getSyncProgress = async (): Promise<SyncProgress> => {
  try {
    return await invoke<SyncProgress>('plugin:cloudreve|get_sync_progress');
  } catch (error) {
    console.error('Failed to get sync progress:', error);
    return { percentage: 0, synced: 0, total: 0, status: 'idle' };
  }
};

// 触发立即同步
export const triggerSync = async (): Promise<boolean> => {
  try {
    return await invoke<boolean>('plugin:cloudreve|trigger_sync');
  } catch (error) {
    console.error('Failed to trigger sync:', error);
    return false;
  }
};
