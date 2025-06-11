import React, { useState } from "react";
import {
    Menu,
    MenuItem,
    ListItemIcon,
    ListItemText,
    Divider,
    IconButton,
} from "@mui/material";
import {
    UploadFile as UploadFileIcon,
    DriveFolderUpload as UploadFolderIcon,
    CreateNewFolder as CreateFolderIcon,
    Add as AddIcon,
} from "@mui/icons-material";

import { preUploadFile } from "../../services/fileService";
import { uploadFile, UploadTask } from "../../services/upload";
import { useAppBar } from "../contexts/AppBarContext";
import { useNotification } from "../contexts/NotificationProvider";

// 定义组件 Props 类型
interface AddMenuProps {
    currentPath: string;
    handleNewFloder: () => void;
}

const AddMenu: React.FC<AddMenuProps> = ({
    currentPath, handleNewFloder,
}) => {
    const [anchorEl, setAnchorEl] = useState(null);
    const isOpen = Boolean(anchorEl);
    // 打开菜单
    const handleClick = (event: any) => {
        setAnchorEl(event.currentTarget);
    };

    // 关闭菜单
    const handleClose = () => {
        setAnchorEl(null);
    };

    const { uploads, setUploads } = useAppBar();
    const { notify } = useNotification();

    const handleFileSelect = async () => {
        const now = Math.floor(Date.now() / 1000)
        try {
            const resp = await preUploadFile(currentPath);
            if (resp) {
                var task = {
                    id: resp.id,
                    filename: resp.filename,
                    fullPath: resp.file_path,
                    url: resp.uri,
                    status: 'pending',
                    size: 0,
                    startTime: now,
                    progress: 0,
                } as UploadTask;

                const list = uploads;
                list.push(task);
                setUploads(list);
                notify(resp.filename + " 开始上传")
                await uploadFile(resp.id, resp.upload_url, resp.file_path)
            } else {

                console.error('not select file:');
            }
        } catch (error) {
            notify(`上传文件失败：${error}`, 'error')
            console.error('Error selecting file:', error);
        }
    };

    const handleUploadFile = () => {
        handleClose()
        handleFileSelect()
    };
    const handleUploadFolder = () => {
        handleClose()
        handleFileSelect()
    };

    const handleNewFolderClick = () => {
        handleNewFloder()
        handleClose()
    }


    return (
        <>

            <IconButton
                onClick={handleClick}
                sx={{
                    borderRadius: 3,
                }}>
                <AddIcon />
            </IconButton>

            {/* 菜单内容 */}
            <Menu
                anchorEl={anchorEl}
                open={isOpen}
                onClose={handleClose}

                transformOrigin={{
                    vertical: "top",
                    horizontal: "right",
                }}
                anchorOrigin={{
                    vertical: "bottom",
                    horizontal: "right",
                }}
            >
                <MenuItem onClick={handleUploadFile}>
                    <ListItemIcon>
                        <UploadFileIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="上传文件" />
                </MenuItem>

                <MenuItem onClick={handleUploadFolder}>
                    <ListItemIcon>
                        <UploadFolderIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="上传目录" />
                </MenuItem>

                <Divider sx={{ my: 1 }} />
                <MenuItem onClick={handleNewFolderClick}>
                    <ListItemIcon>
                        <CreateFolderIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary="创建文件夹" />
                </MenuItem>
            </Menu>

        </>
    );
};

export default AddMenu;