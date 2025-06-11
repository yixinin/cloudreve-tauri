// App.tsx 路由主配置
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider } from './components/contexts/AuthContext';
import LoginPage from './components/pages/LoginPage';
import FileListPage from './components/pages/FileListPage';
import Layout from './components/Layout';
import VideoPlayerPage from './components/pages/VideoPlayerPage';
import TransferManager from './components/pages/TransferManager';
import ShareListPage from './components/pages/ShareListPage';
import { NotificationProvider } from './components/contexts/NotificationProvider';

export function App() {
  return (
    <NotificationProvider>
      <BrowserRouter>
        <AuthProvider>
          <Routes>
            <Route index path='/login' element={<LoginPage />}></Route>
            <Route path="/player/video" element={<VideoPlayerPage />} />
            <Route path="/" element={<Layout />}>
              <Route index element={<FileListPage />} />
              <Route path="files" element={<FileListPage />} />
              <Route path='shares' element={<ShareListPage />} />
              <Route path='transfers' element={<TransferManager />} />
            </Route>
          </Routes>
        </AuthProvider>
      </BrowserRouter>
    </NotificationProvider>
  );
}