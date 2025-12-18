import React, { useEffect, useState } from "react";
import {
  Box,
  Button,
  FormControl,
  IconButton,
  InputAdornment,
  MenuItem,
  Paper,
  Select,
  TextField,
  Typography,
} from "@mui/material";
import { ArrowBack, Visibility, VisibilityOff } from "@mui/icons-material";
import {
  userLogin,
  NetworkMode,
  getNetworkSetting,
  setNetworkSettings,
} from "../../services/userService";
import { useAuth } from "../contexts/AuthContext";
import { CodeError } from "../../services/proto";

// 类型定义

const LoginPage: React.FC = () => {
  const [error, setError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);
  const [addr, setAddr] = useState("");
  const [addr6, setAddr6] = useState("");
  const [irohEndpoint, setIrohEndpoint] = useState("");
  const [networkMode, setNetworkMode] = useState<NetworkMode>(NetworkMode.Auto);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const [step, setStep] = useState(0);

  useEffect(() => {
    const username = localStorage.getItem("user") || "";
    const pass = localStorage.getItem("pwd") || "";

    const getNetworkSettings = async () => {
      try {
        const settings = await getNetworkSetting();
        setAddr(settings.addr);
        setAddr6(settings.addr6);
        setIrohEndpoint(settings.iroh_endpoint);
        setNetworkMode(settings.mode);
        setStep(1);
      } catch (error) {
        console.error("Failed to fetch network settings:", error);
      }
    };
    getNetworkSettings();
    setUsername(username);
    setPassword(pass);
  }, []);
  const { login } = useAuth();

  const handleClickShowPassword = () => {
    setShowPassword(!showPassword);
  };

  const handleLogin = async () => {
    try {
      const ack = await userLogin(username, password);
      localStorage.setItem("user", username);
      localStorage.setItem("pwd", password);
      login(ack);
    } catch (err) {
      console.log(err);

      let { msg } = err as CodeError;
      setError(msg);
    }
  };

  const handleNetworkSetting = async () => {
    try {
      await setNetworkSettings(addr, addr6, irohEndpoint, networkMode);
      setStep(1);
    } catch {
      console.error("Failed to set network settings");
    }
  };

  return (
    <Box
      sx={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        minHeight: "100vh",
        bgcolor: "#f5f5f5",
      }}
    >
      <Paper
        elevation={3}
        sx={{
          p: 4,
          width: "100%",
          maxWidth: 500,
        }}
      >
        <Typography variant="h4" align="center" gutterBottom>
          登录
        </Typography>

        {step === 0 && (
          <Box>
            <TextField
              label="服务器地址"
              variant="outlined"
              fullWidth
              margin="normal"
              value={addr}
              onChange={(e) => setAddr(e.target.value)}
            />
            <TextField
              label="IPv6地址"
              variant="outlined"
              fullWidth
              margin="normal"
              value={addr6}
              onChange={(e) => setAddr6(e.target.value)}
            />
            <TextField
              label="Iroh端点地址"
              variant="outlined"
              fullWidth
              margin="normal"
              placeholder="iroh://..."
              value={irohEndpoint}
              onChange={(e) => setIrohEndpoint(e.target.value)}
              helperText="用于Iroh P2P网络连接的端点地址"
            />
            <FormControl size="small" fullWidth>
              <Select
                labelId="select-network-mode-label"
                id="select-network-mode"
                value={networkMode}
                onChange={(e) => setNetworkMode(e.target.value)}
              >
                <MenuItem value={NetworkMode.Auto}>自动</MenuItem>
                <MenuItem value={NetworkMode.Normal}>主域名</MenuItem>
                <MenuItem value={NetworkMode.IPv6}>仅IPV6域名</MenuItem>
                <MenuItem value={NetworkMode.P2P}>仅P2P</MenuItem>
              </Select>
            </FormControl>

            <Button
              variant="contained"
              onClick={handleNetworkSetting}
              fullWidth
              sx={{
                marginTop: 2,
              }}
            >下一步</Button>
          </Box>
        )}

        {step === 1 && (
          <Box>
            <IconButton onClick={() => setStep(0)}>
              <ArrowBack />
            </IconButton>
            <TextField
              label="用户名"
              variant="outlined"
              fullWidth
              margin="normal"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />

            <TextField
              label="密码"
              variant="outlined"
              fullWidth
              margin="normal"
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              slotProps={{
                input: {
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton onClick={handleClickShowPassword}>
                        {showPassword ? <VisibilityOff /> : <Visibility />}
                      </IconButton>
                    </InputAdornment>
                  ),
                },
              }}
            />

            {error && <Typography color="red">{error}</Typography>}

            <Button
              variant="contained"
              onClick={handleLogin}
              disabled={!password}
              fullWidth
              sx={{
                marginTop: 2,
              }}
            >
              登录
            </Button>
          </Box>
        )}
      </Paper>
    </Box>
  );
};

export default LoginPage;
