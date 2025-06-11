import React, { useState, useEffect } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    TextField,
    Button,
    IconButton,
    Box,
    Typography
} from '@mui/material';
import { Edit as EditIcon, Close as CloseIcon } from '@mui/icons-material';


export enum FileOp {
    create,
    rename,
}

interface FileNameDialogProps {
    open: boolean;
    targetName: string;
    fileType: number;
    op: FileOp,
    onClose: () => void;
    onSubmit: (newName: string) => Promise<void> | void;
}

const FileNameDialog: React.FC<FileNameDialogProps> = ({
    open,
    targetName,
    fileType,
    op,
    onClose,
    onSubmit,
}) => {
    const [newName, setNewName] = useState(targetName);
    const [error, setError] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    // 重置表单当打开状态或当前名称变化时
    useEffect(() => {
        setNewName(targetName);
        setError('');
        console.log(open, targetName);
    }, []);

    const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const value = e.target.value;
        setNewName(value);

        // 验证文件名
        if (!value.trim()) {
            setError('名称不能为空');
        } else if (value.includes('/') || value.includes('\\')) {
            setError('名称不能包含 / 或 \\');
        } else if (value.length > 255) {
            setError('名称过长（最大255字符）');
        } else {
            setError('');
        }
    };

    const handleSubmit = async () => {
        if (error || !newName.trim()) return;

        setIsSubmitting(true);
        try {
            await onSubmit(newName);
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : '重命名失败');
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleKeyPress = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !error) {
            handleSubmit();
        }
    };

    return (
        <Dialog
            open={open}
            onClose={onClose}
            maxWidth="sm"
            fullWidth
            aria-labelledby="rename-dialog-title"
            PaperProps={{
                sx: {
                    borderRadius: 2,
                    p: 1
                }
            }}
        >
            <DialogTitle id="rename-dialog-title" sx={{ p: 2, pb: 1 }}>
                <Box display="flex" alignItems="center" justifyContent="space-between">
                    <Typography variant="h6" component="div">
                        <EditIcon fontSize="small" sx={{ mr: 1, verticalAlign: 'middle' }} />
                        {op === FileOp.create ? '新建' : '重命名'}{fileType === 1 ? '文件夹' : '文件'}
                    </Typography>
                    <IconButton edge="end" onClick={onClose} aria-label="close">
                        <CloseIcon />
                    </IconButton>
                </Box>
            </DialogTitle>

            <DialogContent sx={{ p: 2 }}>
                <TextField
                    autoFocus
                    margin="dense"
                    label={`${op === FileOp.create ? '' : '新'}${fileType === 1 ? '文件夹名称' : '文件名'}`}
                    type="text"
                    fullWidth
                    variant="outlined"
                    onKeyDown={handleKeyPress}
                    value={newName}
                    onChange={handleNameChange}
                    error={!!error}
                    sx={{ mt: 1 }}
                />
            </DialogContent>

            <DialogActions sx={{ p: 2, pt: 1 }}>
                <Button onClick={onClose} disabled={isSubmitting}>
                    取消
                </Button>
                <Button
                    onClick={handleSubmit}
                    color="primary"
                    variant="contained"
                    disabled={!!error || !newName.trim() || isSubmitting}
                >
                    {isSubmitting ? '处理中...' : '确认'}
                </Button>
            </DialogActions>
        </Dialog>
    );
};

export default FileNameDialog;