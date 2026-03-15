package com.crcli.app.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.database.ContentObserver
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.provider.MediaStore
import androidx.core.app.NotificationCompat
import com.crcli.app.MainActivity

class AlbumSyncService : android.app.Service() {
    private val CHANNEL_ID = "album_sync_channel"
    private val NOTIFICATION_ID = 1001
    private lateinit var mediaObserver: MediaContentObserver
    private var isInitialSyncComplete = false

    // 添加数据库辅助类用于跟踪同步文件
    private inner class SyncDatabaseHelper(context: Context) : SQLiteOpenHelper(context, "sync_db", null, 1) {
        override fun onCreate(db: SQLiteDatabase) {
            db.execSQL("CREATE TABLE synced_files (path TEXT PRIMARY KEY, cloud_id TEXT)")
        }

        override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
            db.execSQL("DROP TABLE IF EXISTS synced_files")
            onCreate(db)
        }
    }

    private lateinit var dbHelper: SyncDatabaseHelper
    override fun onCreate() {
        super.onCreate()
        dbHelper = SyncDatabaseHelper(this)
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification())
        mediaObserver = MediaContentObserver(Handler(Looper.getMainLooper()))
        registerMediaObserver()
        checkInitialSyncStatus()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "相册同步服务",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("相册同步中")
            .setContentText("后台同步照片和视频")
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentIntent(pendingIntent)
            .build()
    }

    private fun registerMediaObserver() {
        contentResolver.registerContentObserver(
            MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
            true,
            mediaObserver
        )
        contentResolver.registerContentObserver(
            MediaStore.Video.Media.EXTERNAL_CONTENT_URI,
            true,
            mediaObserver
        )
    }

    private fun checkInitialSyncStatus() {
        val sharedPrefs = getSharedPreferences("sync_prefs", Context.MODE_PRIVATE)
        isInitialSyncComplete = sharedPrefs.getBoolean("initial_sync_complete", false)

        if (!isInitialSyncComplete) {
            Thread { performInitialSync() }.start()
        }
    }

    private fun performInitialSync() {
        // 首次同步逻辑：获取所有媒体文件并上传
        val mediaFiles = queryAllMediaFiles()
        syncMediaFiles(mediaFiles)

        // 标记首次同步完成
        getSharedPreferences("sync_prefs", Context.MODE_PRIVATE)
            .edit()
            .putBoolean("initial_sync_complete", true)
            .apply()
        isInitialSyncComplete = true
    }

    private fun queryAllMediaFiles(): List<String> {
        val mediaList = mutableListOf<String>()
        // 查询图片
        queryMedia(mediaList, MediaStore.Images.Media.EXTERNAL_CONTENT_URI)
        // 查询视频
        queryMedia(mediaList, MediaStore.Video.Media.EXTERNAL_CONTENT_URI)
        return mediaList
    }

    private fun queryMedia(mediaList: MutableList<String>, uri: Uri) {
        val projection = arrayOf(MediaStore.MediaColumns.DATA)
        val cursor = contentResolver.query(uri, projection, null, null, null)
        cursor?.use {
            val dataColumn = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DATA)
            while (it.moveToNext()) {
                val path = it.getString(dataColumn)
                mediaList.add(path)
            }
        }
    }

    private fun syncMediaFiles(filePaths: List<String>) {
        // 实现文件上传逻辑
        filePaths.forEach { path ->
            // 调用Cloudreve上传API
            val cloudId = uploadToCloudreve(path)
            if (cloudId != null) {
                // 记录已同步文件
                val db = dbHelper.writableDatabase
                val values = ContentValues().apply {
                    put("path", path)
                    put("cloud_id", cloudId)
                }
                db.insertWithOnConflict("synced_files", null, values, SQLiteDatabase.CONFLICT_REPLACE)
            }
        }
    }

    private fun uploadToCloudreve(filePath: String): String? {
        // 实现Cloudreve上传API调用
        // 这里需要替换为实际的上传实现
        try {
            // 调用项目现有的文件上传服务
            val uploadService = com.crcli.app.services.FileUploadService()
            return uploadService.uploadFile(filePath, "/相册同步/")
        } catch (e: Exception) {
            e.printStackTrace()
            return null
        }
    }

    private fun deleteSyncedFiles(filePaths: List<String>) {
        val db = dbHelper.writableDatabase
        filePaths.forEach { path ->
            // 查询云端ID
            val cursor = db.query("synced_files", arrayOf("cloud_id"), "path = ?", arrayOf(path), null, null, null)
            if (cursor.moveToFirst()) {
                val cloudId = cursor.getString(0)
                // 调用Cloudreve删除API
                deleteFromCloudreve(cloudId)
                // 从本地数据库删除记录
                db.delete("synced_files", "path = ?", arrayOf(path))
            }
            cursor.close()
        }
    }

    private fun deleteFromCloudreve(cloudId: String) {
        // 实现Cloudreve删除API调用
        try {
            val fileService = com.crcli.app.services.FileService()
            fileService.deleteFile(cloudId)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun detectDeletedMediaFiles(uri: Uri): List<String> {
        val currentFiles = queryCurrentMediaFiles(uri)
        val db = dbHelper.readableDatabase
        val cursor = db.query("synced_files", arrayOf("path"), null, null, null, null, null)
        val deletedFiles = mutableListOf<String>()

        while (cursor.moveToNext()) {
            val path = cursor.getString(0)
            if (!currentFiles.contains(path)) {
                deletedFiles.add(path)
            }
        }
        cursor.close()
        return deletedFiles
    }

    private fun queryCurrentMediaFiles(uri: Uri): List<String> {
        val mediaList = mutableListOf<String>()
        val projection = arrayOf(MediaStore.MediaColumns.DATA)
        val cursor = contentResolver.query(uri, projection, null, null, null)
        cursor?.use {
            val dataColumn = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DATA)
            while (it.moveToNext()) {
                val path = it.getString(dataColumn)
                mediaList.add(path)
            }
        }
        return mediaList
    }

    inner class MediaContentObserver(handler: Handler) : ContentObserver(handler) {
        override fun onChange(selfChange: Boolean, uri: Uri?) {
            super.onChange(selfChange, uri)
            if (isInitialSyncComplete) {
                uri?.let { handleMediaChange(it) }
            }
        }

        private fun handleMediaChange(uri: Uri) {
            // 处理媒体文件变化
            if (uri.toString().contains("images")) {
                handleImageChanges()
            } else if (uri.toString().contains("video")) {
                handleVideoChanges()
            }
        }

        private fun handleImageChanges() {
            // 检测新增或删除的图片并同步
            val newImages = detectNewMediaFiles(MediaStore.Images.Media.EXTERNAL_CONTENT_URI)
            val deletedImages = detectDeletedMediaFiles(MediaStore.Images.Media.EXTERNAL_CONTENT_URI)
            syncMediaFiles(newImages)
            deleteSyncedFiles(deletedImages)
        }

        private fun handleVideoChanges() {
            // 检测新增或删除的视频并同步
            val newVideos = detectNewMediaFiles(MediaStore.Video.Media.EXTERNAL_CONTENT_URI)
            val deletedVideos = detectDeletedMediaFiles(MediaStore.Video.Media.EXTERNAL_CONTENT_URI)
            syncMediaFiles(newVideos)
            deleteSyncedFiles(deletedVideos)
        }

        private fun detectNewMediaFiles(uri: Uri): List<String> {
            val newFiles = mutableListOf<String>()
            val lastSyncTime = getLastSyncTime()
            val projection = arrayOf(MediaStore.MediaColumns.DATA, MediaStore.MediaColumns.DATE_MODIFIED)
            val selection = "${MediaStore.MediaColumns.DATE_MODIFIED} > ?"
            val selectionArgs = arrayOf(lastSyncTime.toString())

            contentResolver.query(uri, projection, selection, selectionArgs, null)?.use {
                val dataColumn = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DATA)
                while (it.moveToNext()) {
                    val path = it.getString(dataColumn)
                    newFiles.add(path)
                }
            }

            updateLastSyncTime()
            return newFiles
        }

        private fun detectDeletedMediaFiles(uri: Uri): List<String> {
            // 实现检测删除文件的逻辑
            // 需要对比上次同步的文件列表与当前文件列表
            return emptyList()
        }

        private fun getLastSyncTime(): Long {
            return getSharedPreferences("sync_prefs", Context.MODE_PRIVATE)
                .getLong("last_sync_time", 0)
        }

        private fun updateLastSyncTime() {
            getSharedPreferences("sync_prefs", Context.MODE_PRIVATE)
                .edit()
                .putLong("last_sync_time", System.currentTimeMillis() / 1000)
                .apply()
        }
    }

    override fun onBind(intent: Intent): android.os.IBinder? {
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        contentResolver.unregisterContentObserver(mediaObserver)
    }
}