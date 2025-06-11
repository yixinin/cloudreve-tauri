import React from 'react';
import {
    List,
    ListItem,
    ListItemText,
    Typography,
    Box,
    Divider,
    Chip,
    IconButton
} from '@mui/material';
import {
    Folder as FolderIcon,
    CheckCircle as CheckIcon,
    Delete as DeleteIcon
} from '@mui/icons-material';
import { DownloadTask } from '../../../services/upload';
import { invoke } from '@tauri-apps/api/core';

interface HistoryListProps {
    items: DownloadTask[];
}

const HistoryList: React.FC<HistoryListProps> = ({ items }) => {
    const handleDeleteHistory = (id: number) => {
        // 调用Rust后端删除历史记录
        invoke('delete_download_history', { id });
    };

    return (
        <>
            {items.length === 0 ? (
                <Typography variant="body2" color="text.secondary" sx={{ py: 4, textAlign: 'center' }}>
                    暂无历史记录
                </Typography>
            ) : (
                <List dense>
                    {items.map((item, index) => (
                        <React.Fragment key={item.id}>
                            <ListItem
                                secondaryAction={
                                    <IconButton edge="end" onClick={() => handleDeleteHistory(item.id)}>
                                        <DeleteIcon />
                                    </IconButton>
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
                                            <Chip
                                                label="已完成"
                                                color="success"
                                                size="small"
                                                icon={<CheckIcon />}
                                            />
                                        </Box>
                                    }
                                    secondary={
                                        <>
                                            <Typography variant="caption" component="div">
                                                下载时间: {item.startTime}
                                            </Typography>
                                            <Typography variant="caption" component="div">
                                                保存路径: {item.fullPath}
                                            </Typography>
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

export default HistoryList;