import React, { useState } from 'react';
import {
    ToggleButton,
    Box,
    Typography,
    Stack,
    ButtonGroup,
    Menu
} from '@mui/material';
import {
    ViewList as ListIcon,
    Collections as GalleryIcon,
    GridView as GridIcon,
    HideImage as HideImageIcon,
    Image as ImageIcon,
} from '@mui/icons-material';
import { position } from '../menus/FileContextMenu';

// 定义视图类型
export type ViewMode = 'grid' | 'list' | 'gallery';

interface ViewToggleProps {
    open: boolean,
    value: ViewMode;
    show: boolean,
    pos: position,
    onViewModeChange: (view: ViewMode) => void;
    onShowThumbChange: (show: boolean) => void;
    onClose: () => void;
}

const ViewToggle: React.FC<ViewToggleProps> = ({
    open,
    value,
    show,
    pos,
    onViewModeChange,
    onShowThumbChange,
    onClose,
}) => {
    const [mode, setMode] = useState(value)
    const [showThumb, setShowThumb] = useState(show)

    const handleViewChange = (newView: ViewMode) => {
        setMode(newView)
        onViewModeChange(newView);
        handleClose()
    };

    const handleShowThumbChange = (show: boolean) => {
        setShowThumb(show);
        onShowThumbChange(show);
    };

    const handleClose = () => {
        onClose()
    };



    // 移动端使用菜单，桌面端使用按钮组
    return (
        <Menu
            open={open}
            onClose={handleClose}
            anchorReference="anchorPosition"
            anchorPosition={
                { top: pos.y + 28, left: pos.x }
            }
        >

            <Stack spacing={1} sx={{
                padding: 2,
                backgroundColor: 'transparent'
            }}>
                <Typography>布局</Typography>
                <Box sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                }}>
                    <ButtonGroup sx={{
                        borderRadius: 3,
                        border: 1
                    }}>
                        <ToggleButton
                            value="check"
                            size='small'
                            selected={mode === 'grid'}
                            onClick={() => handleViewChange('grid')}
                            sx={{
                                width: '100%',
                                borderRadius: '10px 0 0 10px',
                            }}
                        >
                            <Stack direction="row" spacing={1} sx={{ scale: '0.9' }}  >
                                <GridIcon />
                                <Typography>网格</Typography>
                            </Stack>

                        </ToggleButton>

                        <ToggleButton
                            value="check"
                            size='small'
                            selected={mode === 'list'}
                            onClick={() => handleViewChange('list')}
                            sx={{
                                width: '100%',
                                borderRadius: 0,
                            }}
                        >
                            <Stack direction="row" spacing={1} sx={{ scale: '0.9' }}  >
                                <ListIcon />
                                <Typography>列表</Typography>
                            </Stack>

                        </ToggleButton>

                        <ToggleButton
                            value="check"
                            size='small'
                            selected={mode === 'gallery'}
                            onClick={() => handleViewChange('gallery')}
                            sx={{
                                width: '100%',
                                borderRadius: '0 10px 10px 0',
                            }}
                        >
                            <Stack direction="row" spacing={1} sx={{ scale: '0.9' }}  >
                                <GalleryIcon />
                                <Typography>画廊</Typography>
                            </Stack>

                        </ToggleButton>
                    </ButtonGroup>

                </Box>
                <Typography>缩略图</Typography>
                <Box sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                }}>
                    <ButtonGroup sx={{
                        borderRadius: 3,
                        border: 0.5,
                        width: '100%',
                        justifyContent: 'space-around'
                    }}>
                        <ToggleButton
                            value="check"
                            size='small'
                            selected={showThumb}
                            onClick={() => handleShowThumbChange(true)}
                            sx={{
                                width: '100%',
                                borderRadius: '10px 0 0 10px',
                            }}
                        >
                            <Stack direction="row" spacing={1} sx={{ scale: '0.9' }}  >
                                <ImageIcon />
                                <Typography>开启</Typography>
                            </Stack>

                        </ToggleButton>

                        <ToggleButton
                            value="check"
                            size='small'
                            selected={!showThumb}
                            onClick={() => handleShowThumbChange(false)}
                            // onChange={() => setShowThumb(!show)}
                            sx={{
                                width: '100%',
                                borderRadius: '0 10px 10px 0',
                            }}
                        >
                            <Stack direction="row" spacing={1} sx={{ scale: '0.9' }}  >
                                <HideImageIcon />
                                <Typography>关闭</Typography>
                            </Stack>

                        </ToggleButton>
                    </ButtonGroup>

                </Box>
            </Stack>

        </Menu >
    );
};

export default ViewToggle;