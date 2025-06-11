import React, { useState } from 'react';
import { Box, Chip, Popover, Typography, Stack } from '@mui/material';
import { FileTag } from '../../services/fileModel';
import { position } from '../menus/ShareContextMenu';

interface TagPreviewProps {
    tags: FileTag[];
    pos: position,
    maxVisible?: number; // 默认可见的标签数量
}

const TagPreview: React.FC<TagPreviewProps> = ({ tags, pos, maxVisible = 3 }) => {
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
    const visibleTags = tags.slice(1, maxVisible);
    const hiddenTags = tags.slice(maxVisible);

    const handlePopoverOpen = (event: React.MouseEvent<HTMLElement>) => {
        if (hiddenTags.length > 0) {
            setAnchorEl(event.currentTarget);
        }
    };

    const handlePopoverClose = () => {
        setAnchorEl(null);
    };

    const open = Boolean(anchorEl);

    return (
        <Box sx={{
            display: 'flex',
            alignItems: 'center',
            position: 'absolute',
            left: `${pos.x - (tags.length * 30) / 2}px`,
            top: `${pos.y + 20}px`,
            backgroundColor: 'white',
            borderRadius: 2,
            padding: 1
        }}>
            {/* 可见标签 */}
            < Stack direction="row" spacing={1} >
                {
                    visibleTags.map((tag, index) => (
                        <Chip key={index} label={tag.key} size="small" sx={{
                            backgroundColor: tag.color,
                        }} />
                    ))
                }
            </Stack >

            {/* 隐藏标签提示（如果有） */}
            {
                hiddenTags.length > 0 && (
                    <>
                        <Chip
                            label={`+${hiddenTags.length}`}
                            size="small"
                            onMouseEnter={handlePopoverOpen}
                            onMouseLeave={handlePopoverClose}
                            sx={{ ml: 1, cursor: 'pointer' }}
                        />
                        <Popover
                            open={open}
                            anchorEl={anchorEl}
                            anchorOrigin={{
                                vertical: 'bottom',
                                horizontal: 'left',
                            }}
                            transformOrigin={{
                                vertical: 'top',
                                horizontal: 'left',
                            }}
                            onClose={handlePopoverClose}
                            disableRestoreFocus
                            sx={{
                                pointerEvents: 'none', // 避免遮挡鼠标事件
                            }}
                        >
                            <Box sx={{ p: 2, maxWidth: 300 }}>
                                <Typography variant="subtitle2" gutterBottom>
                                    所有标签 ({tags.length}个)
                                </Typography>
                                <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                                    {tags.map((tag, index) => (
                                        <Chip key={index} label={tag.key} size="small" sx={{ mb: 1, backgroundColor: tag.color }} />
                                    ))}
                                </Stack>
                            </Box>
                        </Popover>
                    </>
                )
            }
        </Box>
    );
};

export default TagPreview;