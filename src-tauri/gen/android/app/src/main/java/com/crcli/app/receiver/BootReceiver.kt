package com.crcli.app.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import com.crcli.app.service.AlbumSyncService

class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            // 启动相册同步服务
            val serviceIntent = Intent(context, AlbumSyncService::class.java)
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                context.startForegroundService(serviceIntent)
            } else {
                context.startService(serviceIntent)
            }
        }
    }
}