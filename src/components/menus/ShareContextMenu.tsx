import React, { useState } from 'react';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import {
    Delete as DeleteIcon,
    FileOpen as FileOpenIcon,
    DriveFileRenameOutline as RenameIcon,
} from '@mui/icons-material';
import { ShareItem } from '../../services/share';

export interface position {
    x: number;
    y: number;
}
export interface ShareMenuProps {
    pos: position,
    share: ShareItem,
    handleAction: (action: string) => void;
    onClose: () => void;
}

const ShareContextMenu: React.FC<ShareMenuProps> = ({
    pos,
    share,
    handleAction,
    onClose }) => {
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number } | null>({ x: pos.x, y: pos.y });

    const handleClose = () => {
        setContextMenu(null);
        onClose();
    };



    return (
        <Menu
            open={!!contextMenu}
            onClose={handleClose}
            anchorReference="anchorPosition"
            anchorPosition={
                contextMenu ? { top: contextMenu.y, left: contextMenu.x } : undefined
            }
        >
            <MenuItem onClick={() => handleAction('open')}>
                <ListItemIcon>
                    <FileOpenIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText>打开</ListItemText>
            </MenuItem>
            <MenuItem onClick={() => handleAction('copy')}>
                <ListItemIcon>
                    <FileOpenIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText>复制链接到剪切板</ListItemText>
            </MenuItem>

            {!share.expired && (
                <MenuItem onClick={() => handleAction('edit')}>
                    <ListItemIcon>
                        <RenameIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText>编辑</ListItemText>
                </MenuItem>
            )}


            <MenuItem onClick={() => handleAction('delete')}>
                <ListItemIcon>
                    <DeleteIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText>删除</ListItemText>
            </MenuItem>
        </Menu>
    );
};

export default ShareContextMenu;