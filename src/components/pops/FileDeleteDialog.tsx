import React, { useState } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogContentText,
    DialogActions,
    Button,
    Checkbox,
    FormControlLabel,
    Typography,
    Box,
    IconButton,
    Collapse,
    Alert,
    CircularProgress,
    Stack
} from '@mui/material';
import {
    Delete as DeleteIcon,
    Close as CloseIcon,
    Warning as WarningIcon,
    ExpandMore as ExpandMoreIcon,
    ExpandLess as ExpandLessIcon,
    InfoOutline as InfoIcon,
} from '@mui/icons-material';

interface FileDeleteDialogProps {
    open: boolean;
    onClose: () => void;
    onDelete: (force: boolean) => Promise<void>;
    fileName: string;
    fileType: number;
}

const FileDeleteDialog: React.FC<FileDeleteDialogProps> = ({
    open,
    onClose,
    onDelete,
    fileName,
    fileType,
}) => {
    const [forceDelete, setForceDelete] = useState(false);
    const [isDeleting, setIsDeleting] = useState(false);
    const [error, setError] = useState('');
    const [showAdvanced, setShowAdvanced] = useState(false);

    const handleDelete = async () => {
        setIsDeleting(true);
        setError('');
        try {
            await onDelete(forceDelete);
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : '删除失败');
        } finally {
            setIsDeleting(false);
        }
    };

    const resetState = () => {
        setForceDelete(false);
        setIsDeleting(false);
        setError('');
        setShowAdvanced(false);
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
            aria-labelledby="delete-dialog-title"
            PaperProps={{
                sx: {
                    borderRadius: 2,
                    border: error ? '1px solid' : undefined,
                    borderColor: error ? 'error.main' : undefined
                }
            }}
        >
            <DialogTitle id="delete-dialog-title" sx={{ p: 2, pb: 1 }}>
                <Box display="flex" alignItems="center" justifyContent="space-between">
                    <Box display="flex" alignItems="center">
                        <WarningIcon color="error" sx={{ mr: 1 }} />
                        <Typography variant="h6" component="span">
                            删除对象
                        </Typography>
                    </Box>
                    <IconButton edge="end" onClick={handleClose} aria-label="close">
                        <CloseIcon />
                    </IconButton>
                </Box>
            </DialogTitle>

            <DialogContent sx={{ p: 2 }}>
                {error && (
                    <Alert severity="error" sx={{ mb: 2 }}>
                        {error}
                    </Alert>
                )}

                {!forceDelete && (
                    <Box>
                        <DialogContentText sx={{ mb: 2 }}>
                            确定要将 <strong>{fileName}</strong> 移至回收站吗？
                        </DialogContentText>
                        <Stack direction='row'>
                            <InfoIcon />
                            <Typography>回收站中的文件会在 7 天 后自动删除。</Typography>
                        </Stack>
                    </Box>

                )}
                {forceDelete && (
                    <DialogContentText sx={{ mb: 2 }}>
                        确定要永久删除 <strong>{fileName}</strong> 吗？
                    </DialogContentText>
                )}


                <Box
                    sx={{
                        display: 'flex',
                        alignItems: 'center',
                        color: 'text.secondary',
                        cursor: 'pointer',
                        mb: 1
                    }}
                    onClick={() => setShowAdvanced(!showAdvanced)}
                >
                    {showAdvanced ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                    <Typography variant="body2" sx={{ ml: 0.5 }}>
                        高级选项
                    </Typography>
                </Box>

                <Collapse in={showAdvanced}>
                    <Box
                        sx={{
                            p: 2,
                            backgroundColor: 'action.hover',
                            borderRadius: 1,
                            border: '1px solid',
                            borderColor: 'divider'
                        }}
                    >
                        <FormControlLabel
                            control={
                                <Checkbox
                                    checked={forceDelete}
                                    onChange={(e) => setForceDelete(e.target.checked)}
                                    color="error"
                                />
                            }
                            label={
                                <Box>
                                    <Typography>强制删除文件</Typography>
                                    <Typography variant="caption" color="text.secondary">
                                        忽略错误并强制删除{fileType === 1 ? '文件夹及其内容' : '文件'}
                                    </Typography>
                                </Box>
                            }
                            sx={{ alignItems: 'flex-start' }}
                        />
                    </Box>
                </Collapse>
            </DialogContent>

            <DialogActions sx={{ p: 2, pt: 1 }}>
                <Button onClick={handleClose} disabled={isDeleting}>
                    取消
                </Button>
                <Button
                    onClick={handleDelete}
                    color="error"
                    variant="contained"
                    disabled={isDeleting}
                    startIcon={isDeleting ? <CircularProgress size={20} /> : <DeleteIcon />}
                    sx={{
                        '&.Mui-disabled': {
                            backgroundColor: 'error.main',
                            opacity: 0.7
                        }
                    }}
                >
                    {isDeleting ? '删除中...' : '确认删除'}
                </Button>
            </DialogActions>
        </Dialog>
    );
};

export default FileDeleteDialog;