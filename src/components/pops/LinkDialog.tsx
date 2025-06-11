import React, { useState, useEffect, useRef } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    TextField,
    Button,
    IconButton,
    InputAdornment,
    Box,
    Typography,
    CircularProgress,
    Tooltip,
    Stack,
    Checkbox
} from '@mui/material';
import {
    Close as CloseIcon,
    ContentCopy as CopyIcon,
    Check as CheckIcon,
    Link as LinkIcon,
} from '@mui/icons-material';
import { getFileSource } from '../../services/fileService';

interface LinkDialogProps {
    open: boolean;
    onClose: () => void;
    fileName: string,
    filePath: string;
}

const LinkDialog: React.FC<LinkDialogProps> = ({
    open,
    onClose,
    fileName,
    filePath,
}) => {
    const [fileLink, setFileLink] = useState('');
    const [fixedFileLink, setFixedFileLink] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState('');
    const [copied, setCopied] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);
    const [showFilename, setShowFilename] = useState(false);

    // 获取分享链接
    useEffect(() => {
        if (open) {
            const fetchLink = async () => {
                setIsLoading(true);
                setError('');
                try {
                    const link = await getFileSource(filePath);
                    setFileLink(link);
                    setCopied(false);
                } catch (err) {
                    setError(err instanceof Error ? err.message : '获取链接失败');
                    setFileLink('');
                } finally {
                    setIsLoading(false);
                }
            };

            fetchLink();
        }
    }, [open, filePath]);

    useEffect(() => {
        if (showFilename) {
            setFixedFileLink(`[${fileName}]${fileLink}`)
        } else {
            setFixedFileLink(fileLink)
        }
    }, [fileLink, showFilename])

    // 自动全选文本
    useEffect(() => {
        if (open && fixedFileLink && inputRef.current) {
            inputRef.current.select();
        }
    }, [open, fixedFileLink]);

    const handleCopy = () => {
        if (fixedFileLink) {
            navigator.clipboard.writeText(fixedFileLink);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleClose = () => {
        setFileLink('');
        setError('');
        setCopied(false);
        onClose();
    };

    const handleShowFilenameChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setShowFilename(event.target.checked)
    }

    return (
        <Dialog
            open={open}
            onClose={handleClose}
            maxWidth="sm"
            fullWidth
            PaperProps={{
                sx: {
                    borderRadius: 2
                }
            }}
        >
            <DialogTitle>
                <Box display="flex" alignItems="center" justifyContent="space-between">
                    <Typography variant="h6">
                        获取文件直链
                    </Typography>
                    <IconButton edge="end" onClick={handleClose}>
                        <CloseIcon />
                    </IconButton>
                </Box>
            </DialogTitle>

            <DialogContent dividers>
                {isLoading ? (
                    <Box display="flex" justifyContent="center" py={4}>
                        <CircularProgress />
                    </Box>
                ) : error ? (
                    <Typography color="error">{error}</Typography>
                ) : (
                    <>
                        <TextField
                            inputRef={inputRef}
                            fullWidth
                            value={fixedFileLink}
                            variant="outlined"
                            InputProps={{
                                startAdornment: (
                                    <InputAdornment position="start">
                                        <LinkIcon color="action" />
                                    </InputAdornment>
                                ),
                                readOnly: true,
                                sx: {
                                    '& input': {
                                        cursor: 'text',
                                        caretColor: 'transparent',
                                        py: 1.5
                                    }
                                }
                            }}
                            onClick={() => inputRef.current?.select()}
                            sx={{ mt: 1 }}
                        />
                    </>
                )}
            </DialogContent>

            <DialogActions>
                <Button onClick={handleClose}>关闭</Button>
                <Tooltip title={copied ? '已复制!' : '复制链接'} arrow>
                    <span> {/* 包裹Button解决disabled时Tooltip不显示的问题 */}
                        <Button
                            onClick={handleCopy}
                            color="primary"
                            variant="contained"
                            disabled={!fixedFileLink || isLoading}
                            startIcon={copied ? <CheckIcon /> : <CopyIcon />}
                        >
                            {copied ? '已复制' : '复制链接'}
                        </Button>
                    </span>
                </Tooltip>
            </DialogActions>
            <Stack direction='row' spacing={0} marginLeft={2} sx={{
                alignItems: 'center'
            }}>
                <Checkbox
                    size='small'
                    checked={showFilename}
                    onChange={handleShowFilenameChange}
                    name="isPrivate"
                    color="primary"
                />
                <Typography fontSize='small'>显示文件名</Typography>
            </Stack>
        </Dialog>
    );
};

export default LinkDialog;