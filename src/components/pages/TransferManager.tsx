import React, { useState, useEffect } from 'react';
import {
    Box,
    Tabs,
    Tab,
    Typography,
    Paper,
    CircularProgress
} from '@mui/material';
import DownloadList from './views/DownloadList';
import UploadList from './views/UploadList';
import HistoryList from './views/DownloadHistoryList';
import { invoke } from '@tauri-apps/api/core';
import { DownloadTask } from '../../services/upload';
import { useAppBar } from '../contexts/AppBarContext';



const TransferManager = () => {
    const [tabIndex, setTabIndex] = useState(0);
    const [histories, setHistory] = useState<DownloadTask[]>([]);
    const [loading, setLoading] = useState(false);

    const { uploads, downloads } = useAppBar();


    // 获取历史下载记录
    const fetchDownloadHistory = async () => {
        setLoading(true);
        try {
            const historyData: DownloadTask[] = await invoke('get_download_history');
            setHistory(historyData);
        } catch (error) {
            console.error('Failed to fetch download history:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (tabIndex === 2) { // 历史记录tab
            fetchDownloadHistory();
        }
    }, [tabIndex]);

    const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
        setTabIndex(newValue);
    };

    const onRefresh = () => {

    }

    return (
        <Paper elevation={3} sx={{ p: 2, borderRadius: 2, height: '100%' }}>
            <Typography variant="h5" gutterBottom>
                文件传输管理
            </Typography>

            <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
                <Tabs value={tabIndex} onChange={handleTabChange}>
                    <Tab label={`下载列表 (${downloads.length})`} />
                    <Tab label={`上传列表 (${uploads.length})`} />
                    <Tab label="历史下载记录" />
                </Tabs>
            </Box>

            <Box sx={{ pt: 2 }}>
                {loading ? (
                    <Box display="flex" justifyContent="center" py={4}>
                        <CircularProgress />
                    </Box>
                ) : (
                    <>
                        {tabIndex === 0 && <DownloadList items={downloads} onRefresh={onRefresh} />}
                        {tabIndex === 1 && <UploadList items={uploads} />}
                        {tabIndex === 2 && <HistoryList items={histories} />}
                    </>
                )}
            </Box>
        </Paper>
    );
};

export default TransferManager;