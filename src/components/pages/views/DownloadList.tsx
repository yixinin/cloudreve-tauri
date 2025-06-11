import React from 'react';
import {
    List,
    ListItem,
    ListItemText,
    IconButton,
    LinearProgress,
    Typography,
    Box,
    Divider,
    Chip
} from '@mui/material';
import {
    Pause as PauseIcon,
    PlayArrow as PlayIcon,
    Delete as DeleteIcon,
    Folder as FolderIcon,
    CheckCircle as CheckIcon
} from '@mui/icons-material';
import { DownloadTask } from '../../../services/upload';

interface DownloadListProps {
    items: DownloadTask[];
    onRefresh: () => void;
}

const DownloadList: React.FC<DownloadListProps> = ({ items }) => {
    const handlePause = (_: number) => {
        // 调用Rust后端暂停下载
        // invoke('pause_download', { id });
    };

    const handleResume = (_: number) => {
        // 调用Rust后端继续下载
        // invoke('resume_download', { id });
    };

    const handleCancel = (_: number) => {
        // 调用Rust后端取消下载
        // invoke('cancel_download', { id }).then(onRefresh);
    };

    const getStatusChip = (status: string, progress: number, total: number) => {
        switch (status) {
            case 'downloading':
                return <Chip label={`下载中 ${Math.floor(progress / total * 100)}%`} color="primary" size="small" />;
            case 'paused':
                return <Chip label="已暂停" color="warning" size="small" />;
            case 'completed':
                return <Chip label="已完成" color="success" size="small" icon={<CheckIcon />} />;
            case 'failed':
                return <Chip label="失败" color="error" size="small" />;
            default:
                return <Chip label="等待中" size="small" />;
        }
    };

    return (
        <>
            {items.length === 0 ? (
                <Typography variant="body2" color="text.secondary" sx={{ py: 4, textAlign: 'center' }}>
                    暂无下载任务
                </Typography>
            ) : (
                <List dense>
                    {items.map((item, index) => (
                        <React.Fragment key={item.id}>
                            <ListItem
                                secondaryAction={
                                    <>
                                        {item.status === 'downloading' ? (
                                            <IconButton edge="end" onClick={() => handlePause(item.id)}>
                                                <PauseIcon />
                                            </IconButton>
                                        ) : item.status === 'paused' || item.status === 'pending' ? (
                                            <IconButton edge="end" onClick={() => handleResume(item.id)}>
                                                <PlayIcon />
                                            </IconButton>
                                        ) : null}
                                        <IconButton edge="end" onClick={() => handleCancel(item.id)}>
                                            <DeleteIcon />
                                        </IconButton>
                                    </>
                                }
                            >
                                <Box sx={{ mr: 2 }}>
                                    <FolderIcon color="action" />
                                </Box>
                                <ListItemText
                                    primary={
                                        <Box display="flex" alignItems="center">
                                            <Typography variant="body1" sx={{ flexGrow: 1 }}>
                                                {item.filename}
                                            </Typography>
                                            {getStatusChip(item.status, item.progress, item.size)}
                                        </Box>
                                    }
                                    secondary={
                                        <>
                                            <Typography variant="caption" component="div">
                                                {item.url}
                                            </Typography>
                                            <Typography variant="caption" component="div">
                                                保存路径: {item.fullPath}
                                            </Typography>
                                            <LinearProgress
                                                variant="determinate"
                                                value={(item.progress / item.size) * 100}
                                                sx={{ mt: 1, height: 6 }}
                                            />
                                        </>
                                    }
                                />
                            </ListItem>
                            {index < items.length - 1 && <Divider variant="inset" component="li" />}
                        </React.Fragment>
                    ))}
                </List>
            )}
        </>
    );
};

export default DownloadList;