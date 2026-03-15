import React from 'react';
import { Navigate, Outlet } from 'react-router-dom';
import { useAuth } from './AuthContext';

const ProtectedRoute: React.FC = () => {
  const { isAuthenticated } = useAuth();

  // 如果用户已经认证，则渲染子路由
  if (isAuthenticated) {
    return <Outlet />;
  }

  // 如果用户未认证，则重定向到登录页面
  return <Navigate to="/login" replace />;
};

export default ProtectedRoute;