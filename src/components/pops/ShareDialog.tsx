import React, { useEffect, useState } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    Checkbox,
    TextField,
    InputAdornment,
    IconButton,
    Typography,
    Box,
    CircularProgress,
    Stack
} from '@mui/material';
import {
    Close as CloseIcon,
    ContentCopy as CopyIcon,
    Link as LinkIcon,
    Check as CheckIcon,
    Visibility,
    Alarm,
    BrowserUpdated
} from '@mui/icons-material';
import { shareFile } from '../../services/fileService';
import { getShareInfo, updateShare } from '../../services/share';

interface ShareDialogProps {
    open: boolean;
    onClose: () => void;
    filePath: string;
    shareId: string;
}

const ShareDialog: React.FC<ShareDialogProps> = ({
    open,
    onClose,
    filePath,
    shareId,
}) => {
    const [options, setOptions] = useState({
        isPrivate: false,
        expire: 0,
        downloads: 0,
    });
    const [shareLink, setShareLink] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState('');
    const [copied, setCopied] = useState(false);
    const [shareUri, setShareUri] = useState('');

    const handleOptionChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        console.log(event.target.name, event.target.checked);

        switch (event.target.name) {
            case 'expire':
                setOptions({
                    ...options,
                    [event.target.name]: event.target.checked ? 86400 : 0,
                });
                break
            case 'downloads':
                setOptions({
                    ...options,
                    [event.target.name]: event.target.checked ? 1 : 0,
                });
                break
            default:
                setOptions({
                    ...options,
                    [event.target.name]: event.target.checked
                });
        }
    };


    useEffect(() => {
        const loadShareUri = async () => {
            try {
                const info = await getShareInfo(shareId, true)
                if (info?.source_uri) {
                    setShareUri(`${info.source_uri}/${info.name}`);
                }
            }
            catch (err) {
                setError('获取分享信息失败')
            }
        }
        if (shareId) {
            loadShareUri();
        }

    }, [])

    const handleShare = async () => {
        setIsLoading(true);
        setError('');
        try {
            var link;
            if (shareId === '') {
                link = await shareFile(options.downloads, options.expire, options.isPrivate, filePath);
            } else {

                link = await updateShare(shareId, options.downloads, options.expire, shareUri);
            }

            setShareLink(link);
            setCopied(false);

        } catch (err) {
            setError(err instanceof Error ? err.message : '分享失败');
        } finally {
            setIsLoading(false);
        }
    };

    const handleCopy = () => {
        if (shareLink) {
            navigator.clipboard.writeText(shareLink);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const resetState = () => {
        setOptions({
            isPrivate: false,
            expire: 0,
            downloads: 0,
        });
        setShareLink('');
        setError('');
        setCopied(false);
    };

    const handleClose = () => {
        resetState();
        onClose();
    };

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
                    <Typography variant="h6">{shareId === '' ? '编辑分享链接' : '分享文件'}</Typography>
                    <IconButton edge="end" onClick={handleClose}>
                        <CloseIcon />
                    </IconButton>
                </Box>
            </DialogTitle>

            <DialogContent dividers>
                {!shareLink ? (
                    <>
                        <Box sx={{ display: 'flex', flexDirection: 'column', ml: 1 }}>
                            <Box sx={{
                                display: 'flex',
                                justifyContent: 'space-between',
                            }}>
                                <Stack direction='row' spacing={2}>
                                    <Visibility htmlColor='gray' />
                                    <Typography>隐藏分享</Typography>
                                </Stack>

                                <Checkbox
                                    disabled={shareId != ''}
                                    checked={options.isPrivate}
                                    onChange={handleOptionChange}
                                    name="isPrivate"
                                    color="primary"
                                />
                            </Box>

                            <Box sx={{
                                display: 'flex',
                                justifyContent: 'space-between',
                            }}>
                                <Stack direction='row' spacing={2}>
                                    <Alarm htmlColor='gray' />
                                    <Typography>超时自动过期</Typography>
                                </Stack>

                                <Checkbox
                                    checked={options.expire > 0}
                                    onChange={handleOptionChange}
                                    name="expire"
                                    color="primary"
                                />
                            </Box>
                            <Box sx={{
                                display: 'flex',
                                justifyContent: 'space-between',
                            }}>
                                <Stack direction='row' spacing={2}>
                                    <BrowserUpdated htmlColor='gray' />
                                    <Typography>下载后自动过期</Typography>
                                </Stack>

                                <Checkbox
                                    checked={options.downloads > 0}
                                    onChange={handleOptionChange}
                                    name="downloads"
                                    color="primary"
                                />
                            </Box>
                        </Box>

                        {error && (
                            <Typography color="error" sx={{ mt: 2 }}>
                                {error}
                            </Typography>
                        )}
                    </>
                ) : (
                    <>
                        <Typography variant="body1" gutterBottom>
                            分享链接已{shareId === '' ? '更新' : '创建'}
                        </Typography>

                        <TextField
                            fullWidth
                            variant="outlined"
                            value={shareLink}
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
                                        caretColor: 'transparent'
                                    }
                                }
                            }}
                            onClick={(e) => (e.target as HTMLInputElement).select()}
                            sx={{ mt: 1 }}
                        />

                        <Box sx={{ mt: 1, display: 'flex', alignItems: 'center' }}>
                            <Typography variant="caption" color="text.secondary">
                                链接已自动全选，可直接复制
                            </Typography>
                        </Box>
                    </>
                )}
            </DialogContent>

            <DialogActions>
                {!shareLink ? (
                    <>
                        <Button onClick={handleClose}>取消</Button>
                        {shareId === '' && (
                            <Button
                                onClick={handleShare}
                                color="primary"
                                variant="contained"
                                disabled={isLoading}
                                startIcon={isLoading ? <CircularProgress size={20} /> : null}
                            >
                                {isLoading ? '创建中...' : '创建分享链接'}
                            </Button>
                        )}
                        {shareId !== '' && (
                            <Button
                                onClick={handleShare}
                                color="primary"
                                variant="contained"
                                disabled={isLoading || error !== ''}
                                startIcon={isLoading ? <CircularProgress size={20} /> : null}
                            >
                                {isLoading ? '更新中...' : '更新分享链接'}
                            </Button>
                        )}
                    </>
                ) : (
                    <>
                        <Button onClick={handleClose}>关闭</Button>
                        <Button
                            onClick={handleCopy}
                            color="primary"
                            variant="contained"
                            startIcon={copied ? <CheckIcon /> : <CopyIcon />}
                        >
                            {copied ? '已复制' : '复制链接'}
                        </Button>
                    </>
                )}
            </DialogActions>
        </Dialog>
    );
};

export default ShareDialog;