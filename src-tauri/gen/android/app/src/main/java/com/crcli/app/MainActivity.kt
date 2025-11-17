package com.crcli.app

import android.view.KeyEvent
import android.webkit.WebView
import android.content.Intent
import com.crcli.app.service.AlbumSyncService
import android.content.pm.PackageManager
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
  private lateinit var wv: WebView

  override fun onWebViewCreate(webView: WebView) {
    wv = webView
  }

  private val keyEventMap = mapOf(
    KeyEvent.KEYCODE_BACK to "back", 
  )

  private val REQUEST_MEDIA_PERMISSIONS = 1001

  override fun onResume() {
    super.onResume()
    // 检查并请求媒体权限
    if (checkMediaPermissions()) {
      startSyncService()
    } else {
      requestMediaPermissions()
    }
  }

  private fun checkMediaPermissions(): Boolean {
    return if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
      ContextCompat.checkSelfPermission(this, android.Manifest.permission.READ_MEDIA_IMAGES) == PackageManager.PERMISSION_GRANTED &&
      ContextCompat.checkSelfPermission(this, android.Manifest.permission.READ_MEDIA_VIDEO) == PackageManager.PERMISSION_GRANTED
    } else {
      ContextCompat.checkSelfPermission(this, android.Manifest.permission.READ_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
    }
  }

  private fun requestMediaPermissions() {
    val permissions = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
      arrayOf(android.Manifest.permission.READ_MEDIA_IMAGES, android.Manifest.permission.READ_MEDIA_VIDEO)
    } else {
      arrayOf(android.Manifest.permission.READ_EXTERNAL_STORAGE)
    }
    ActivityCompat.requestPermissions(this, permissions, REQUEST_MEDIA_PERMISSIONS)
  }

  override fun onRequestPermissionsResult(
    requestCode: Int,
    permissions: Array<String>,
    grantResults: IntArray
  ) {
    super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    if (requestCode == REQUEST_MEDIA_PERMISSIONS) {
      if (grantResults.all { it == PackageManager.PERMISSION_GRANTED }) {
        startSyncService()
      } else {
        // 权限被拒绝，显示提示
        showPermissionDeniedDialog()
      }
    }
  }

  private fun startSyncService() {
    val serviceIntent = Intent(this, AlbumSyncService::class.java)
    if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
      startForegroundService(serviceIntent)
    } else {
      startService(serviceIntent)
    }
  }

  private fun showPermissionDeniedDialog() {
    android.app.AlertDialog.Builder(this)
      .setTitle("权限请求")
      .setMessage("相册同步功能需要访问您的照片和视频。请在设置中启用媒体访问权限，否则无法使用同步功能。")
      .setPositiveButton("去设置") { _, _ ->
        val intent = Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
        intent.data = android.net.Uri.fromParts("package", packageName, null)
        startActivity(intent)
      }
      .setNegativeButton("取消", null)
      .show()
  }

  override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
    // 处理返回键自定义逻辑
    if (keyCode == KeyEvent.KEYCODE_BACK) {
      // 保留现有的自定义返回键处理逻辑
      return true
    }
    
    // 执行其他按键的JS回调处理
    val jsCallbackName = keyEventMap[keyCode]
    wv.evaluateJavascript(jsCallbackName?.let { "window.\$tauri.internal.invokeKeyCallback('$it')" } ?: "", null)
    
    // 未处理的按键交给系统默认处理
    return jsCallbackName != null || super.onKeyDown(keyCode, event)
  }
}