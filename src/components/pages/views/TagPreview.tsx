import React, { useState, useRef } from 'react';
import {
    Chip,
    Menu,
    MenuItem,
    Paper,
    Stack,
    Box,
} from '@mui/material';
import { FileTag } from '../../../services/fileModel';

interface TagPreviewMenuProps {
    tags: FileTag[];
    maxVisible?: number;
}

const TagPreviewMenu: React.FC<TagPreviewMenuProps> = ({
    tags,
    maxVisible = 3,
}) => {
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const visibleTags = tags.slice(0, maxVisible);
    const hiddenTags = tags.slice(maxVisible);

    const handleOpen = (event: React.MouseEvent<HTMLElement>) => {
        if (hiddenTags.length > 0) {
            setAnchorEl(event.currentTarget);
        }
    };

    const handleClose = () => {
        setAnchorEl(null);
    };

    const open = Boolean(anchorEl);

    return (
        <Box ref={containerRef}>
            {/* 可见标签 */}
            <Stack direction="row" spacing={1} >
                {visibleTags.map((tag, index) => (
                    <Chip key={index} label={tag.key} size="small" sx={{ backgroundColor: tag.color }} />
                ))}
                {hiddenTags.length > 0 && (
                    <Chip
                        onMouseEnter={handleOpen}
                        onMouseLeave={handleClose}
                        label={`+${hiddenTags.length}`}
                        size="small"
                        sx={{ cursor: 'pointer', padding: 0 }}
                    />
                )}
            </Stack>
            {tags.length > maxVisible && (
                <Menu
                    open={open}
                    anchorEl={anchorEl}
                    onClose={handleClose}
                    anchorOrigin={{
                        vertical: 'bottom',
                        horizontal: 'left',
                    }}
                    transformOrigin={{
                        vertical: 'top',
                        horizontal: 'left',
                    }}
                    slotProps={{
                        list: {
                            onMouseLeave: handleClose,
                            autoFocus: false,
                        }
                    }}
                    // 悬停交互优化
                    disableAutoFocusItem
                    disableScrollLock
                    sx={{
                        pointerEvents: 'none', // 允许鼠标穿透到菜单内容
                        '& .MuiPaper-root': {
                            pointerEvents: 'none', // 菜单内容可交互
                            maxHeight: 100,
                            overflow: 'hidden',
                        },
                        padding: 0,

                    }}
                >
                    <Paper sx={{
                        p: 0,
                        backgroundColor: 'transparent',
                        boxShadow: 'none',
                    }}>
                        <Stack spacing={1} direction='row'>
                            {tags.slice(maxVisible).map((tag, index) => (
                                <MenuItem key={index} sx={{ p: 0 }}>
                                    <Chip label={tag.key} size="small" sx={{ m: 0.5, backgroundColor: tag.color }} />
                                </MenuItem>
                            ))}
                        </Stack>
                    </Paper>
                </Menu>
            )}
            {/* 预览菜单 */}

        </Box>
    );
};

export default TagPreviewMenu;