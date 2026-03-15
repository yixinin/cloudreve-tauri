// VideoPlayerPage.tsx
import React, { useState, useRef, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import {
    Box,
    IconButton,
    Slider,
    Typography,
    Paper,
    Stack,
    CircularProgress,
} from "@mui/material";


import {
    PlayArrow as PlayIcon,
    Pause as PauseIcon,
    VolumeUp as VolumeUpIcon,
    VolumeOff as VolumeOffIcon,
    Fullscreen as FullscreenIcon,
    FullscreenExit as FullscreenExitIcon,
    Replay as ReplayIcon,
    ArrowBack as BackIcon
} from '@mui/icons-material';
import { getURL } from '../../services/fileService';
import { onBackKeyDown } from 'tauri-plugin-app-events-api';
import { useNotification } from '../contexts/NotificationProvider';


const VideoPlayerPage: React.FC = () => {
    const navigate = useNavigate();
    const videoRef = useRef<HTMLVideoElement>(null);

    const [searchParams] = useSearchParams();
    const uri = searchParams.get('url');

    const [fileUrl, setFileUrl] = useState(uri);

    // 状态管理
    const [isPlaying, setIsPlaying] = useState(true);
    const [progress, setProgress] = useState(0);
    const [volume, setVolume] = useState(50);
    const [isMuted, setIsMuted] = useState(true);
    const [duration, setDuration] = useState(0);
    const [currentTime, setCurrentTime] = useState(0);
    const [showControls, setShowControls] = useState(true);
    const [controlsTimeout, setControlsTimeout] = useState<number>();
    const [isLoading, setIsLoading] = useState(true);
    const [isFullscreen] = useState(false);
    const debounceTimerRef = useRef<number | null>(null);

    // 自定义防抖函数
    const debounce = (fn: (time: number) => void, delay: number) => {
        return (time: number) => {
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
            }
            debounceTimerRef.current = setTimeout(() => fn(time), delay);
        };
    };

    // 防抖处理函数
    const debouncedTimeUpdate = debounce((time: number) => {
        setCurrentTime(time);
        if (videoRef.current) {
            setProgress(time / videoRef.current.duration * 100)
        }

    }, 10);

    // 清理定时器
    useEffect(() => {
        return () => {
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
            }
        };
    }, []);

    useEffect(() => {
        const loadVideoUrl = async () => {
            try {
                if (uri) {
                    const videoUrl = await getURL(uri);
                    setFileUrl(videoUrl);
                }
            }
            catch (err) {
                console.log("fetch video url with error: ", err);
            }


        }

        loadVideoUrl()
    }, [uri]);

    onBackKeyDown(() => {
        navigate(-1)
    })



    // 处理播放/暂停
    const togglePlay = () => {
        if (videoRef.current) {
            if (isPlaying) {
                videoRef.current.pause();
            } else {
                videoRef.current.play();
            }
            setIsPlaying(!isPlaying);
        }
    };


    // 处理音量变化
    const handleVolumeChange = (_: Event, newValue: number | number[]) => {
        const newVolume = Array.isArray(newValue) ? newValue[0] : newValue;
        setVolume(newVolume);
        setIsMuted(newVolume === 0);
        console.log("volume update", newVolume);

        if (videoRef.current) {
            const volume = newVolume / 100
            videoRef.current.volume = volume;
            if (newVolume > 0 && isMuted) {
                toggleMute()
            }
            if (!newVolume && !isMuted) {
                toggleMute()
            }
            console.log("volume update", videoRef.current.volume);
        }
        resetControlsTimeout();
    };

    // 静音/取消静音
    const toggleMute = () => {
        if (videoRef.current) {
            videoRef.current.muted = !isMuted;
            setIsMuted(!isMuted);
        }
        console.log("mute: ", videoRef.current?.muted);

        resetControlsTimeout();
    };



    const { notify } = useNotification();

    // 退出全屏
    const exitFullscreen = async () => {
        try {
            if (document.exitFullscreen) {
                await document.exitFullscreen();
            } else if ((document as any).webkitExitFullscreen) {
                await (document as any).webkitExitFullscreen();
            }
        } catch (err) {
            console.error('退出全屏失败:', err);
        }
    };


    const enterFullscreen = async () => {
        if (!videoRef.current) return;
        try {
            if (videoRef.current.requestFullscreen) {
                await videoRef.current.requestFullscreen();
            } else if ((videoRef.current as any).webkitRequestFullscreen) {
                await (videoRef.current as any).webkitRequestFullscreen();
            } else if ((videoRef.current as any).msRequestFullscreen) {
                await (videoRef.current as any).msRequestFullscreen();
            }
        } catch (err) {
            notify(`全屏错误：${err}`)
            console.error('全屏错误:', err);
        }
    }

    const toggleFullscreen = async () => {
        if (!document.fullscreenElement) {
            enterFullscreen()
        } else {
            exitFullscreen()
        }
        resetControlsTimeout();
    };



    // 重播视频
    const replayVideo = () => {
        if (videoRef.current) {
            videoRef.current.currentTime = 0;
            videoRef.current.play();
            setIsPlaying(true);
        }
        resetControlsTimeout();
    };

    // 重置控制条隐藏计时器
    const resetControlsTimeout = () => {
        if (controlsTimeout) clearTimeout(controlsTimeout);
        setShowControls(true);
        setControlsTimeout(setTimeout(() => setShowControls(false), 3000));
    };

    // 处理进度条变化
    const handleProgressChange = (_: Event, newValue: number | number[]) => {
        const newProgress = Array.isArray(newValue) ? newValue[0] : newValue;
        setProgress(newProgress);
        if (videoRef.current) {
            videoRef.current.currentTime = (newProgress / 100) * duration;
        }
        resetControlsTimeout();
    };

    // 初始化视频事件监听
    useEffect(() => {
        const video = videoRef.current;
        if (!video) return;

        const handleLoadedData = () => {
            setDuration(video.duration);
            video.volume = volume / 100;
        };
        const handleTimeUpdate = () => {
            if (videoRef.current) {
                const value = videoRef.current.currentTime;
                debouncedTimeUpdate(value);
            }
        };
        const handleEnded = () => {
            setIsPlaying(false);
        };



        video.addEventListener('loadedmetadata', handleLoadedData);
        video.addEventListener('timeupdate', handleTimeUpdate);
        video.addEventListener('ended', handleEnded);
        video.addEventListener('click', togglePlay);
        video.addEventListener('dblclick', toggleFullscreen);

        // 鼠标移动时显示控制条
        const container = video.parentElement;
        if (container) {
            container.addEventListener('mousemove', resetControlsTimeout);
        }

        return () => {
            video.removeEventListener('loadedmetadata', handleLoadedData);
            video.removeEventListener('timeupdate', handleTimeUpdate);
            video.removeEventListener('ended', handleEnded);
            video.removeEventListener('click', togglePlay);
            video.removeEventListener('dblclick', toggleFullscreen);

            if (container) {
                container.removeEventListener('mousemove', resetControlsTimeout);
            }

            if (controlsTimeout) clearTimeout(controlsTimeout);
        };
    }, [fileUrl, volume]);

    // 格式化时间 (秒 -> MM:SS)
    const formatTime = (timeInSeconds: number) => {
        const minutes = Math.floor(timeInSeconds / 60);
        const seconds = Math.floor(timeInSeconds % 60);
        return `${minutes}:${seconds < 10 ? '0' : ''}${seconds}`;
    };


    return (
        <Box
            sx={{
                position: 'relative',
                width: '100%',
                height: '100%',
                backgroundColor: '#050505',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                overflow: 'hidden',
                minHeight: '100vh',
            }}
        >
            {/* 返回按钮 */}
            <IconButton
                sx={{
                    position: 'absolute',
                    top: 16,
                    left: 16,
                    zIndex: 10,
                    color: '#fff',
                    backgroundColor: 'rgba(0, 0, 0, 0.5)',
                    '&:hover': {
                        backgroundColor: 'rgba(0, 0, 0, 0.7)'
                    }
                }}
                onClick={() => navigate(-1)}
            >
                <BackIcon fontSize="large" />
            </IconButton>
            {isLoading && (
                <Box
                    sx={{
                        position: 'absolute',
                        top: '50%',
                        left: '50%',
                        transform: 'translate(-50%, -50%)',
                    }}
                >
                    <CircularProgress /> {/* 默认旋转动画 */}
                </Box>
            )}
            {/* 视频元素 */}

            <Box sx={{
                display: isLoading ? 'none' : 'block'
            }}>
                <div>
                    <video
                        autoPlay
                        muted
                        webkit-playsinline
                        x5-video-player-type="h5"
                        x5-video-orientation="portrait"
                        ref={videoRef}
                        src={fileUrl || ''}
                        onClick={(e: any) => {
                            e.preventDefault();
                            e.stopPropagation(); // 阻止事件冒泡
                            togglePlay()
                        }}
                        onLoadedData={() => setIsLoading(false)}
                        style={{
                            maxWidth: "100%",     // 不超过父容器宽度
                            maxHeight: "100vh",    // 限制为视口高度的80%（避免滚动条）
                            objectFit: 'contain',
                            cursor: 'pointer',
                            backgroundColor: 'black',
                        }}
                    />
                    <div
                        style={{
                            position: "absolute",
                            top: 0,
                            left: 0,
                            width: "100%",
                            height: "100%",
                            cursor: "not-allowed",
                        }}
                    />
                </div>
            </Box>


            {/* 视频控制条 */}
            {showControls && (
                <Paper
                    sx={{
                        position: 'absolute',
                        bottom: 0,
                        left: 0,
                        right: 0,
                        backgroundColor: 'rgba(0, 0, 0, 0.7)',
                        color: '#fff',
                        p: 1,
                        transition: 'opacity 0.3s ease'
                    }}
                    onClick={(e) => e.stopPropagation()}
                >
                    <Stack spacing={2}>
                        {/* 进度条 */}
                        <Slider
                            value={progress}
                            onChange={handleProgressChange}
                            sx={{
                                color: '#fff',
                                height: 4,
                                '& .MuiSlider-thumb': {
                                    width: 12,
                                    height: 12,
                                    transition: '0.3s cubic-bezier(.47,1.64,.41,.8)',
                                    '&:hover, &.Mui-focusVisible': {
                                        boxShadow: '0 0 0 8px rgba(255, 255, 255, 0.16)'
                                    },
                                    '&.Mui-active': {
                                        width: 20,
                                        height: 20
                                    }
                                },
                                '& .MuiSlider-rail': {
                                    opacity: 0.5
                                }
                            }}
                        />

                        {/* 控制按钮 */}
                        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                            <Box sx={{ display: 'flex', alignItems: 'center' }}>
                                {/* 播放/暂停按钮 */}
                                <IconButton onClick={togglePlay} color="inherit">
                                    {isPlaying ? <PauseIcon fontSize="large" /> : <PlayIcon fontSize="large" />}
                                </IconButton>

                                {/* 音量控制 */}
                                <IconButton onClick={toggleMute} color="inherit">
                                    {isMuted || volume === 0 ? (
                                        <VolumeOffIcon fontSize="medium" />
                                    ) : (
                                        <VolumeUpIcon fontSize="medium" />
                                    )}
                                </IconButton>
                                <Slider
                                    value={isMuted ? 0 : volume}
                                    onChange={handleVolumeChange}
                                    sx={{
                                        width: 100,
                                        marginLeft: 1,
                                        color: '#fff',
                                        '& .MuiSlider-thumb': {
                                            width: 12,
                                            height: 12
                                        }
                                    }}
                                />

                                {/* 时间显示 */}
                                <Typography variant="body2" sx={{ ml: 2, minWidth: 100 }}>
                                    {formatTime(currentTime)} / {formatTime(duration)}
                                </Typography>
                            </Box>

                            <Box>
                                {/* 重播按钮 */}
                                <IconButton onClick={replayVideo} color="inherit">
                                    <ReplayIcon fontSize="medium" />
                                </IconButton>

                                {/* 全屏按钮 */}
                                <IconButton onClick={toggleFullscreen} color="inherit">
                                    {isFullscreen ? <FullscreenExitIcon fontSize="medium" /> : <FullscreenIcon fontSize="medium" />}
                                </IconButton>
                            </Box>
                        </Box>
                    </Stack>
                </Paper>
            )}
        </Box>
    );
};

export default VideoPlayerPage;