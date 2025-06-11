import React, { useState, useEffect } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    IconButton,
    Typography,
    CircularProgress,
    List,
    ListItem,
    ListItemIcon,
    ListItemText,
    Collapse,
    Box,
    Stack,
} from '@mui/material';
import {
    Folder as FolderIcon,
    FolderOpen as FolderOpenIcon,
    KeyboardArrowDown as ArrowDownIcon,
    KeyboardArrowRight as ArrowRightIcon,
} from '@mui/icons-material';
import { getFileList, moveFile } from '../../services/fileService';
import { HomePath } from '../contexts/AppBarContext';
import { useNotification } from '../contexts/NotificationProvider';

interface DirectoryItem {
    name: string;
    path: string;
    type: number; // 1 for directory, 0 for file
}

interface TreeState {
    [path: string]: {
        expanded: boolean;
        loaded: boolean;
        loading: boolean;
        children: DirectoryItem[];
    };
}

interface ListDirectorySelectorProps {
    open: boolean;
    filePath: string,
    op: 'move' | 'copy',
    onClose: () => void;
}

const ListDirectorySelector: React.FC<ListDirectorySelectorProps> = ({
    open,
    filePath,
    op,
    onClose,
}) => {
    const root = HomePath
    const [selectedPath, setSelectedPath] = useState(root);
    const [treeState, setTreeState] = useState<TreeState>({
        '': {
            expanded: true,
            loaded: true,
            loading: false,
            children: [{
                name: '我的文件',
                path: root,
                type: 1,
            }],
        }
    });
    const [error, setError] = useState<string | null>(null);

    const fetchDirectoryContents = async (path: string) => {
        console.log("loading", path);

        setTreeState(prev => ({
            ...prev,
            [path]: {
                ...prev[path],
                loading: true
            }
        }));
        setError(null);

        try {
            const { files } = await getFileList(path, 0, 100, 'created_at', 'asc');
            const directories = files.filter(item => item.type === 1);
            console.log("load", path, directories);

            setTreeState(prev => ({
                ...prev,
                [path]: {
                    ...prev[path],
                    children: directories,
                    loaded: true,
                    loading: false
                }
            }));
        } catch (err) {
            setError(`Failed to load directory: ${err}`);
            console.error(err);
            setTreeState(prev => ({
                ...prev,
                [path]: {
                    ...prev[path],
                    loading: false
                }
            }));
        }
    };

    const loadRootDirectory = async () => {
        if (!treeState['']?.loaded) {
            await fetchDirectoryContents(selectedPath);
        }
    };

    useEffect(() => {
        if (open) {
            loadRootDirectory();
        }
    }, [open]);

    const toggleDirectory = (path: string) => {
        setTreeState(prev => ({
            ...prev,
            [path]: {
                ...prev[path],
                expanded: !prev[path]?.expanded
            }
        }));

        // 如果目录未加载且将要展开，则加载内容
        if (!treeState[path]?.loaded && !treeState[path]?.loading && !treeState[path]?.expanded) {
            fetchDirectoryContents(path);
        }
    };

    const { notify } = useNotification();

    const handleSelect = async () => {
        try {
            await moveFile(op === 'copy', selectedPath, filePath);
            switch (op) {
                case 'copy':
                    notify(`已复制到${selectedPath}`)
                    break
                case 'move':
                    notify(`已移动到${selectedPath}`)
                    break
            }
        }
        catch (err) {
            if (op === 'move') {
                notify(`移动文件失败，${err}`)
                setError(`移动文件失败，${err}`)
            } else {
                setError(`复制文件失败，${err}`)
                notify(`复制文件失败，${err}`)
            }
        }
        onClose();
    };

    const renderDirectory = (item: DirectoryItem, level: number = 0) => {
        const state = treeState[item.path] || {
            expanded: false,
            loaded: false,
            loading: false,
            children: []
        };

        return (
            <React.Fragment key={item.path}>
                <ListItem
                    onClick={() => {
                        setSelectedPath(item.path);
                        toggleDirectory(item.path);
                    }}
                    // selected={selectedPath === item.path}

                    sx={{
                        pl: 2 + level * 2,
                        backgroundColor: selectedPath === item.path ? 'rgb(186,213,241)' : 'white',
                        '&:hover': {
                            backgroundColor: selectedPath === item.path ? 'rgb(186,213,241)' : 'action.hover'
                        }
                    }}
                >
                    <ListItemIcon sx={{ minWidth: 32 }}>
                        {state.expanded ? <FolderOpenIcon color="primary" /> : <FolderIcon color="primary" />}
                    </ListItemIcon>
                    <ListItemText primary={item.name} />
                    {state.loading ? (
                        <CircularProgress size={20} />
                    ) : (
                        <IconButton
                            edge="end"
                            size="small"
                            onClick={(e) => {
                                e.stopPropagation();
                                toggleDirectory(item.path);
                            }}
                        >
                            {state.expanded ? <ArrowDownIcon /> : <ArrowRightIcon />}
                        </IconButton>
                    )}
                </ListItem>
                <Collapse in={state.expanded} timeout="auto" unmountOnExit>
                    <List component="div" disablePadding>
                        {state.loading && !state.children?.length && (
                            <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
                                <CircularProgress size={20} />
                            </Box>
                        )}
                        {state.children?.map(child => renderDirectory(child, level + 1))}
                    </List>
                </Collapse>
            </React.Fragment>
        );
    };

    return (
        <Dialog open={open} onClose={onClose} fullWidth maxWidth="md">
            <DialogTitle>
                <Stack direction='row' spacing={2} justifyContent="start" alignItems="center">
                    <Typography variant="h6">{op === 'move' ? '移动' : '复制'}到</Typography>
                    <Typography>
                        我的文件{selectedPath.replace(HomePath, "")}
                    </Typography>
                </Stack>
            </DialogTitle>
            <DialogContent sx={{ minHeight: '400px' }}>

                {error && (
                    <Typography color="error" sx={{ mb: 2 }}>
                        {error}
                    </Typography>
                )}

                <List sx={{ width: '100%', bgcolor: 'background.paper' }}>
                    {treeState['']?.children?.map(item => renderDirectory(item))}
                    {!treeState['']?.loaded && (
                        <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
                            <CircularProgress />
                        </Box>
                    )}
                </List>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>取消</Button>
                <Button
                    onClick={handleSelect}
                    variant="contained"
                    color="primary"
                    disabled={!selectedPath}
                >
                    {op === 'move' ? '移动' : '复制'}
                </Button>
            </DialogActions>
        </Dialog>
    );
};

export default ListDirectorySelector;