import React, { useState, useEffect } from 'react';
import { Box, Typography, Paper, Switch, LinearProgress, Divider, List, ListItem, ListItemText, Chip } from '@mui/material';
import { getSyncStatus, toggleSync, getSyncProgress } from '../../services/syncService';

const SettingsPage = () => {
  const [syncEnabled, setSyncEnabled] = useState(false);
  const [syncProgress, setSyncProgress] = useState(0);
  const [syncStatus, setSyncStatus] = useState('idle'); // idle, syncing, completed, error
  const [syncedFiles, setSyncedFiles] = useState(0);
  const [totalFiles, setTotalFiles] = useState(0);
  const [lastSyncTime, setLastSyncTime] = useState('从未同步');

  // 初始化时获取同步状态
  useEffect(() => {
    const loadSyncStatus = async () => {
      try {
        const status = await getSyncStatus();
        setSyncEnabled(status.enabled);
        setLastSyncTime(status.lastSync || '从未同步');
        setSyncedFiles(status.syncedFiles || 0);
      } catch (error) {
        console.error('Failed to load sync status:', error);
      }
    };

    loadSyncStatus();
  }, []);

  // 监听同步进度
  useEffect(() => {
    let intervalId: number | null = null;
    if (syncEnabled && syncStatus === 'syncing') {
      intervalId = setInterval(async () => {
        try {
          const progress = await getSyncProgress();
          setSyncProgress(progress.percentage);
          setSyncedFiles(progress.synced);
          setTotalFiles(progress.total);
          setSyncStatus(progress.status);
        } catch (error) {
          console.error('Failed to get sync progress:', error);
        }
      }, 1000);
    }

    return () => {
      if (intervalId !== null) {
        clearInterval(intervalId);
        intervalId = null;
      }
    };
  }, [syncEnabled, syncStatus]);

  // 切换同步开关
  const handleSyncToggle = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const enabled = event.target.checked;
    setSyncEnabled(enabled);

    try {
      await toggleSync(enabled);
      if (enabled) {
        setSyncStatus('syncing');
      } else {
        setSyncStatus('idle');
      }
    } catch (error) {
      console.error('Failed to toggle sync:', error);
      setSyncEnabled(!enabled); // 切换失败时恢复原状态
    }
  };

  // 获取状态标签样式
  const getStatusChip = () => {
    switch (syncStatus) {
      case 'syncing':
        return <Chip label="同步中" color="primary" size="small" />
      case 'completed':
        return <Chip label="已完成" color="success" size="small" />
      case 'error':
        return <Chip label="同步失败" color="error" size="small" />
      default:
        return <Chip label="未同步" color="default" size="small" />
    }
  };

  return (
    <Box sx={{ p: 3, maxWidth: 600, margin: '0 auto' }}>
      <Typography variant="h5" gutterBottom>设置</Typography>
      <Divider sx={{ mb: 3 }} />

      <Paper elevation={1} sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>相册同步</Typography>

        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <ListItemText primary="启用相册自动同步" secondary="同步系统相册中的照片和视频到Cloudreve" />
          <Switch checked={syncEnabled} onChange={handleSyncToggle} name="syncEnabled" />
        </Box>

        <Divider sx={{ my: 2 }} />

        <Box sx={{ mb: 2 }}>
          <Typography variant="subtitle1" gutterBottom>同步状态: {getStatusChip()}</Typography>
          {syncStatus === 'syncing' && (
            <Box sx={{ mb: 1 }}>
              <LinearProgress variant="determinate" value={syncProgress} />
              <Typography variant="body2" sx={{ mt: 1 }}>
                已同步: {syncedFiles}/{totalFiles} 文件 ({syncProgress}%)
              </Typography>
            </Box>
          )}
        </Box>

        <List disablePadding>
          <ListItem>
            <ListItemText primary="上次同步时间" secondary={lastSyncTime} />
          </ListItem>
          <ListItem>
            <ListItemText primary="已同步文件总数" secondary={syncedFiles} />
          </ListItem>
        </List>
      </Paper>
    </Box>
  );
};

export default SettingsPage;