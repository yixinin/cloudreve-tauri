import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Grid,
    Card,
    CardContent,
    Typography,
    Select,
    MenuItem,
    FormControl,
    CircularProgress,
    Box,
    Stack,
    IconButton
} from '@mui/material';
import RefreshIcon from '@mui/icons-material/Refresh';
import { styled } from '@mui/material/styles';
import { deleteShare, GetSharesAck, ShareItem } from '../../services/share';
import { getFileIcon } from '../../utils/file';
import { Visibility } from '@mui/icons-material';
import ShareContextMenu, { position } from '../menus/ShareContextMenu';
import { useNotification } from '../contexts/NotificationProvider';
import ShareDialog from '../pops/ShareDialog';

const StyledCard = styled(Card)(({ }) => ({
    backgroundColor: 'rgb(240, 240, 240)',
    borderRadius: '12px', // 增加圆角
    transition: 'all 0.3s ease',
    overflow: 'hidden', // 确保内容也遵循圆角
    '&:hover': {
        backgroundColor: 'rgb(220, 220, 220)',
        // transform: 'translateY(-4px)',
        // boxShadow: theme.shadows[8],
        cursor: 'pointer'
    },
    // 卡片内容也添加圆角
    '& .MuiCardContent-root': {
        borderRadius: 'inherit',
        padding: '5px', // 减少为12px (默认为16px)
        '&:last-child': {
            paddingBottom: '0px' // 确保底部内边距一致
        }
    },
}));
const ShareListPage: React.FC = () => {
    const [shares, setShares] = useState<GetSharesAck | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [orderDirection, setOrderDirection] = useState<'asc' | 'desc'>('desc');
    const [menuPos, setMenuPos] = useState<position | null>(null);
    const [selectedShare, setSelectedShare] = useState<ShareItem | null>(null);
    const [edit, setEdit] = useState(false);

    const handleRefresh = () => {
        fetchShares()
    }

    const fetchShares = async () => {
        setLoading(true);
        setError(null);
        try {
            const data: GetSharesAck = await invoke('get_shares', {
                pageSize: 200,
                orderDirection
            });
            setShares(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to fetch shares');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchShares();
    }, [orderDirection]);

    const handleOpenShare = (url: string) => {
        window.open(url, '_blank');
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString();
    };

    const onFileRightClick = (event: any, share: ShareItem) => {
        setMenuPos({
            x: event?.clientX,
            y: event?.clientY,
        })
        setSelectedShare(share)
    }
    const { notify } = useNotification();

    const handleDeleteShare = async (id: string) => {
        try {
            await deleteShare(id)
            handleRefresh()
        }
        catch (err) {
            console.log("delete share fail", err);
            notify("删除分享失败")
        }
    }


    const handleShareAction = (action: string) => {
        if (!selectedShare) {
            return
        }
        setMenuPos(null);
        switch (action) {
            case 'open':
                window.open(selectedShare.url, '_blank');
                return
            case 'copy':
                navigator.clipboard.writeText(selectedShare.url);
                notify("已复制到剪切板")
                return
            case 'edit':
                setEdit(true);
                return
            case "delete":
                handleDeleteShare(selectedShare.id)
                return
        }

    }


    return (
        <Box sx={{ p: 3, backgroundColor: 'white', height: "100%" }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Box display='flex'>
                    <Typography variant="h4" component="h1">
                        我的分享
                    </Typography>
                    <IconButton onClick={fetchShares}>
                        <RefreshIcon />
                    </IconButton>
                </Box>

                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                    <FormControl size="small" sx={{ minWidth: 120, }}>
                        <Select
                            value={orderDirection}
                            onChange={(e) => setOrderDirection(e.target.value as 'asc' | 'desc')}
                            fullWidth
                            sx={{
                                borderRadius: 3,
                                padding: 0,
                            }}
                        >
                            <MenuItem value="desc">最新</MenuItem>
                            <MenuItem value="asc">最早</MenuItem>
                        </Select>
                    </FormControl>
                </Box>
            </Box>

            {loading && !shares && (
                <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
                    <CircularProgress />
                </Box>
            )}

            {error && (
                <Box sx={{ color: 'error.main', textAlign: 'center', mt: 2 }}>
                    {error}
                </Box>
            )}

            {shares && (
                <>
                    <Typography variant="subtitle1" sx={{ mb: 2 }}>
                        Total: {shares.pagination.total} items
                    </Typography>

                    <Grid container spacing={1}  >
                        {shares.shares.map((share) => (
                            <Grid
                                size={{ xs: 12, sm: 6, md: 4, lg: 3 }}
                                key={share.id}
                                sx={{
                                    width: '100%'
                                }}>
                                <StyledCard
                                    onContextMenu={(event: any) => { onFileRightClick(event, share) }}
                                    onClick={() => handleOpenShare(share.url)}
                                    sx={{
                                        width: '100%'
                                    }}
                                >
                                    <CardContent sx={{
                                        width: '100%'
                                    }}>
                                        <Box sx={{
                                            display: 'flex',
                                            justifyContent: "start",
                                            alignItems: 'center',
                                            padding: '0px 30px 0px 0px'
                                        }}>
                                            <Box sx={{
                                                padding: '0px 5px'
                                            }}>
                                                {getFileIcon(share.source_type, share.name)}
                                            </Box>


                                            <Stack sx={{
                                                width: '100%',
                                                // marginLeft: "10px",
                                                padding: 0.5,
                                                // border: 1
                                            }}>
                                                <Box sx={{
                                                    display: 'flex',
                                                    justifyContent: 'space-between',
                                                    alignItems: 'center',
                                                    width: '100%',
                                                    gap: 1,
                                                    overflow: 'hidden', // 防止整体溢出
                                                    padding: 0,
                                                }}>
                                                    <Typography
                                                        variant="body2"
                                                        noWrap
                                                        fontSize={16}
                                                        sx={{
                                                            textOverflow: 'ellipsis',
                                                            overflow: 'hidden',
                                                            whiteSpace: 'nowrap',
                                                            display: 'inline-block',
                                                        }}
                                                    >
                                                        {share.name}
                                                    </Typography>
                                                    {share.expired && (
                                                        <Typography sx={{
                                                            flex: '0 0 auto',
                                                            backgroundColor: 'rgb(220,220,220)',
                                                            borderRadius: '8px',
                                                            fontSize: '12px',
                                                            padding: '0 8px'
                                                        }}>已失效</Typography>
                                                    )}

                                                </Box>

                                                <Box sx={{
                                                    display: 'flex',
                                                    justifyContent: 'space-between',
                                                    alignItems: 'center',
                                                    width: '100%',
                                                    overflow: 'hidden',
                                                    padding: 0,
                                                }}>
                                                    <Typography
                                                        fontSize={12}
                                                        variant="body2"
                                                        color="text.secondary"
                                                        sx={{
                                                            width: "fit-content",
                                                            textOverflow: 'ellipsis',
                                                            overflow: 'hidden',
                                                            whiteSpace: 'nowrap',
                                                        }}
                                                    >
                                                        Created: {formatDate(share.created_at)}
                                                    </Typography>

                                                    <Box sx={{
                                                        display: 'flex',
                                                        justifyContent: 'center',
                                                        alignItems: 'center',
                                                    }}>
                                                        <Visibility htmlColor='gray' sx={{ scale: '0.7' }} ></Visibility>
                                                        <Typography fontSize={12}>{share.visited}</Typography>
                                                    </Box>
                                                </Box>

                                            </Stack>

                                        </Box>
                                    </CardContent>
                                </StyledCard>
                            </Grid>
                        ))}
                    </Grid>
                </>
            )}
            {(menuPos && selectedShare) && (
                <ShareContextMenu
                    pos={menuPos}
                    share={selectedShare}
                    handleAction={handleShareAction}
                    onClose={() => { setMenuPos(null) }}
                />
            )}
            {((selectedShare && edit) && (
                <ShareDialog
                    open={edit}
                    onClose={() => { setMenuPos(null); setEdit(false) }}
                    filePath={selectedShare.url}
                    shareId={selectedShare.id}
                />
            ))}

        </Box >
    );
};

export default ShareListPage;