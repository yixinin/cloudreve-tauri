import React, { useState } from 'react';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import {
    ContentCopy as CopyIcon,
    Delete as DeleteIcon,
    FileOpen as FileOpenIcon,
    Download as DownloadIcon,
    Share as ShareIcon,
    DriveFileRenameOutline as RenameIcon,
    DriveFileMove as MoveIcon,
    InfoOutline as DetailsIcon,
    Link as LinkIcon,
    Restore as RestoreIcon,
    LocalOffer as TagIcon,
    ArrowRight,
} from '@mui/icons-material';
import { FileItem } from '../../services/fileService';
import { fileCanOpen } from '../../utils/file';
import { Box, Divider } from '@mui/material';


export interface position {
    x: number;
    y: number;
}
export interface FileMenuProps {
    pos: position,
    file: FileItem,
    isTrash: boolean,
    handleAction: (action: string) => void;
    onClose: () => void;
}

const FileContextMenu: React.FC<FileMenuProps> = ({
    pos,
    file,
    isTrash,
    handleAction,
    onClose }) => {
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number } | null>({ x: pos.x, y: pos.y });

    const handleClose = () => {
        setContextMenu(null);
        onClose();
    };


    const handleFileAction = (action: string) => {
        handleAction(action)
    }
    return (
        <Menu
            open={!!contextMenu}
            onClose={handleClose}
            anchorReference="anchorPosition"
            anchorPosition={
                contextMenu ? { top: contextMenu.y, left: contextMenu.x } : undefined
            }
        >
            {(!isTrash && fileCanOpen(file)) && (
                <MenuItem onClick={() => handleFileAction('open')}>
                    <ListItemIcon>
                        <FileOpenIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>打开</ListItemText>
                </MenuItem>
            )}

            {(!isTrash && file.type == 0) && (
                <MenuItem onClick={() => handleFileAction('download')}>
                    <ListItemIcon>
                        <DownloadIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>下载</ListItemText>
                </MenuItem>
            )}
            <Divider />
            {!isTrash && (
                <MenuItem onClick={() => handleFileAction('share')}>
                    <ListItemIcon>
                        <ShareIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>分享</ListItemText>
                </MenuItem>
            )}


            {!isTrash && (
                <MenuItem onClick={() => handleFileAction('source')}>
                    <ListItemIcon>
                        <LinkIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>获取直链</ListItemText>
                </MenuItem>
            )}

            {!isTrash && (
                <MenuItem onClick={() => handleFileAction('rename')}>
                    <ListItemIcon>
                        <RenameIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>重命名</ListItemText>
                </MenuItem>
            )}

            {!isTrash && (
                <MenuItem onClick={() => handleFileAction('copy')}>
                    <ListItemIcon>
                        <CopyIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>复制</ListItemText>
                </MenuItem>
            )}

            {!isTrash && (
                <MenuItem onClick={() => handleFileAction('move')}>
                    <ListItemIcon>
                        <MoveIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>移动</ListItemText>
                </MenuItem>
            )}
            {isTrash && (
                <MenuItem onClick={() => handleFileAction('restore')}>
                    <ListItemIcon>
                        <RestoreIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>还原</ListItemText>
                </MenuItem>
            )}

            <Divider />
            <MenuItem onClick={() => handleFileAction('tags')}>
                <Box width='100%' sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                }}>

                    <Box width='100%' sx={{
                        display: 'flex',
                        justifyContent: 'center',
                        alignItems: 'center'
                    }}>
                        <ListItemIcon>
                            <TagIcon sx={{
                                transform: {
                                    rotate: '90'
                                }
                            }} fontSize="small" />
                        </ListItemIcon>
                        <ListItemText>标签</ListItemText>
                    </Box>
                    <ArrowRight />
                </Box>


            </MenuItem>
            <MenuItem onClick={() => handleFileAction('details')}>
                <ListItemIcon>
                    <DetailsIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText>详细信息</ListItemText>
            </MenuItem>
            <Divider />
            <MenuItem onClick={() => handleFileAction('delete')}>
                <ListItemIcon>
                    <DeleteIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText>删除</ListItemText>
            </MenuItem>
        </Menu>
    );
};

export default FileContextMenu;