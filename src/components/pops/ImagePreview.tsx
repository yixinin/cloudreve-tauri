import React, { useState, useEffect } from 'react';
import {
    Dialog,
    Fade,
    Box,
    CircularProgress,
    styled,
    DialogProps,
    DialogContent,
} from "@mui/material";
import { Close } from '@mui/icons-material';
import { CloseButton } from './dialog';

// 定义组件 Props 类型
interface ImagePreviewProps extends Omit<DialogProps, 'onClose' | 'open'> {
    imageId: string; // 图片ID（用于API获取）
    open: boolean;
    onClose: () => void;
    fetchImage: (id: string) => Promise<string>; // 获取图片URL的异步函数
    transitionDuration?: number; // 自定义动画时长（毫秒）
}

// 自定义样式
const PreviewDialog = styled(Dialog)({
    '& .MuiDialog-paper': {
        maxWidth: '90vw',
        maxHeight: '90vh',
        backgroundColor: 'transparent',
        boxShadow: 'none',
        overflow: 'hidden',
    },
    '& .MuiBackdrop-root': {
        backgroundColor: 'rgba(0, 0, 0, 0.9)',
    },
});



export const ImagePreview: React.FC<ImagePreviewProps> = ({
    imageId,
    open,
    onClose,
    fetchImage,
    transitionDuration = 400,
    ...dialogProps
}) => {
    const [imageUrl, setImageUrl] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // 当打开或imageId变化时获取图片
    useEffect(() => {
        if (!open) return;

        const fetchData = async () => {
            setIsLoading(true);
            setError(null);
            try {
                const url = await fetchImage(imageId);
                setImageUrl(url);
            } catch (err) {
                setError(err instanceof Error ? err.message : '图片加载失败');
            } finally {
                setIsLoading(false);
            }
        };

        fetchData();
    }, [open, imageId, fetchImage]);

    // 关闭时重置状态
    const handleClose = () => {
        setImageUrl(null);
        setError(null);
        onClose();
    };

    return (
        <PreviewDialog
            open={open}
            onClose={handleClose}
            transitionDuration={transitionDuration}

            {...dialogProps}

            sx={{
                justifyContent: 'center',
                height: '100%'
            }}
        >
            {/* 使用最新的Fade API - 直接作为Dialog的过渡组件 */}
            <Fade in={open} timeout={transitionDuration}>
                <Box sx={{
                    position: "relative",
                    display: "flex",
                    justifyContent: "center",
                    alignItems: "center",
                    height: "100vh%", // 确保占满 Dialog 高度 
                    width: '100%',
                }}>

                    <CloseButton onClick={handleClose} aria-label="关闭预览">
                        <Close fontSize="large" />
                    </CloseButton>
                    {isLoading && (
                        <Box
                            display="flex"
                            justifyContent="center"
                            alignItems="center"
                            height={300}
                            width={300}
                        >
                            <CircularProgress color="secondary" />
                        </Box>
                    )}

                    {error && (
                        <Box sx={{ color: 'white', p: 4, textAlign: 'center' }}>
                            {error}
                        </Box>
                    )}

                    {imageUrl && (
                        <Fade in={!isLoading} timeout={transitionDuration} >
                            <DialogContent
                                sx={{
                                    padding: 0, // 去除默认内边距
                                    // display: "flex",
                                    justifyContent: "center",
                                    alignItems: "center",
                                    width: "100%", // 确保宽度占满
                                    height: "100%", // 确保高度占满
                                }}
                            >
                                <Box sx={{
                                    height: '100vh',
                                    width: '100%',
                                    overflow: 'hidden',
                                    // justifyContent: "center",
                                    // alignItems: "center",
                                    display: 'flex'
                                }}

                                >

                                    <img
                                        src={imageUrl}
                                        alt="预览"
                                        style={{
                                            maxWidth: '100%',
                                            maxHeight: "100%",
                                            objectFit: 'contain',
                                            opacity: isLoading ? 0 : 1,
                                            transition: `opacity ${transitionDuration}ms ease-in-out`,
                                        }}
                                    />
                                </Box>

                            </DialogContent>
                        </Fade>
                    )}
                </Box>
            </Fade >
        </PreviewDialog >
    );
};

export default ImagePreview;