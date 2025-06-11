import React, { useEffect, useState } from 'react';
import {
    Box,
    Button,
    Checkbox,
    FormControlLabel,
    IconButton,
    InputAdornment,
    Paper,
    TextField,
    Typography
} from '@mui/material';
import { Visibility, VisibilityOff } from '@mui/icons-material';
import { LoginData, userLogin } from '../../services/userService';
import { useAuth } from '../contexts/AuthContext';
import { CodeError } from '../../services/proto';

// 类型定义


const LoginPage: React.FC = () => {
    const [error, setError] = useState<string | null>(null);
    const [showPassword, setShowPassword] = useState(false);
    const [loginData, setLoginData] = useState<LoginData>({
        addr: '',
        ipv6: false,
        addr6: '',
        username: '',
        password: '',
    });

    const handleChange = (name: keyof LoginData, value: string | boolean) => {
        setLoginData((prev) => ({ ...prev, [name]: value }));
    };


    useEffect(() => {
        const addr = localStorage.getItem('addr') || '';
        const useIPv6 = (localStorage.getItem('ipv6') || '0') === '1';
        const ipv6Address = localStorage.getItem('addr6') || '';
        const username = localStorage.getItem('user') || '';
        const pass = localStorage.getItem("pwd") || '';
        setLoginData({
            addr: addr,
            ipv6: useIPv6,
            addr6: ipv6Address,
            username: username,
            password: pass,
        })
    }, [])
    const { login } = useAuth();

    const handleClickShowPassword = () => {
        setShowPassword(!showPassword);
    }

    const handleLogin = async () => {
        try {
            console.log(loginData);
            const ack = await userLogin(loginData.addr, loginData.ipv6, loginData.addr6, loginData.username, loginData.password);
            localStorage.setItem("pwd", loginData.password);
            login(ack);
        }
        catch (err) {
            console.log(err);

            let { msg } = err as CodeError;
            setError(msg)
        }
    };

    return (
        <Box
            sx={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                minHeight: '100vh',
                bgcolor: '#f5f5f5'
            }}
        >
            <Paper
                elevation={3}
                sx={{
                    p: 4,
                    width: '100%',
                    maxWidth: 500
                }}
            >
                <Typography variant="h4" align="center" gutterBottom>
                    登录
                </Typography>

                <TextField
                    label="服务器地址"
                    variant="outlined"
                    fullWidth
                    margin="normal"
                    value={loginData.addr}
                    onChange={(e) => handleChange('addr', e.target.value)}
                />
                <FormControlLabel
                    control={
                        <Checkbox
                            checked={loginData.ipv6}
                            onChange={(e) => handleChange('ipv6', e.target.checked)}
                        />
                    }
                    label="优先访问IPv6域名"
                />
                {loginData.ipv6 && (
                    <TextField
                        label="IPv6地址"
                        variant="outlined"
                        fullWidth
                        margin="normal"
                        value={loginData.addr6}
                        onChange={(e) => handleChange('addr6', e.target.value)}

                    />
                )}

                <TextField
                    label="用户名"
                    variant="outlined"
                    fullWidth
                    margin="normal"
                    value={loginData.username}
                    onChange={(e) => handleChange('username', e.target.value)}
                />

                <TextField
                    label="密码"
                    variant="outlined"
                    fullWidth
                    margin="normal"
                    type={showPassword ? 'text' : 'password'}
                    value={loginData.password}
                    onChange={(e) => handleChange('password', e.target.value)}
                    slotProps={{
                        input: {
                            endAdornment: (
                                <InputAdornment position="end">
                                    <IconButton onClick={handleClickShowPassword}>
                                        {showPassword ? <VisibilityOff /> : <Visibility />}
                                    </IconButton>
                                </InputAdornment>
                            )
                        }
                    }}
                />

                {error && (
                    <Typography color='red'>
                        {error}
                    </Typography>
                )}

                <Button
                    variant="contained"
                    onClick={handleLogin}
                    disabled={!loginData.password}
                    fullWidth
                    sx={{
                        marginTop: 2
                    }}
                >
                    登录
                </Button>

            </Paper>
        </Box>
    );
};

export default LoginPage;