import React from 'react';
import {
    List,
    ListItem,
    ListItemText,
    LinearProgress,
    Typography,
    Box,
    Divider,
    Chip,
    IconButton
} from '@mui/material';
import {
    Pause as PauseIcon,
    PlayArrow as PlayIcon,
    Delete as DeleteIcon,
    CloudUpload as UploadIcon,
    CheckCircle as CheckIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { UploadTask } from '../../../services/upload';

interface UploadListProps {
    items: UploadTask[];
}

const UploadList: React.FC<UploadListProps> = ({ items }) => {
    const getStatusChip = (status: string, progress: number, total: number) => {
        switch (status) {
            case 'uploading':
                return <Chip label={`上传中 ${Math.floor((progress / total * 100))}%`} color="primary" size="small" />;
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
                    暂无上传任务
                </Typography>
            ) : (
                <List dense>
                    {items.map((item, index) => (
                        <React.Fragment key={item.id}>
                            <ListItem
                                secondaryAction={
                                    <>
                                        {item.status === 'uploading' ? (
                                            <IconButton edge="end" onClick={() => invoke('pause_upload', { id: item.id })}>
                                                <PauseIcon />
                                            </IconButton>
                                        ) : item.status === 'paused' ? (
                                            <IconButton edge="end" onClick={() => invoke('resume_upload', { id: item.id })}>
                                                <PlayIcon />
                                            </IconButton>
                                        ) : null}
                                        <IconButton edge="end" onClick={() => invoke('cancel_upload', { id: item.id })}>
                                            <DeleteIcon />
                                        </IconButton>
                                    </>
                                }
                            >
                                <Box sx={{ mr: 2 }}>
                                    <UploadIcon color="action" />
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
                                                目标路径: {item.fullPath}
                                            </Typography>
                                            <Typography variant="caption" component="div">
                                                开始时间: {item.startTime}
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

export default UploadList;