package org.mealshop.pocket.dev

import android.content.Intent
import android.app.Activity
import android.os.Handler
import android.os.Looper
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.util.concurrent.Executors

class MainActivity : FlutterActivity() {
    companion object {
        // Activity recreation must not create a second writer to the same atomic file.
        private val worker = Executors.newSingleThreadExecutor()
    }
    private var exportResult: MethodChannel.Result? = null
    private var exportValue: ByteArray? = null
    @Deprecated("Android activity result bridge")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != 508) return
        val result = exportResult ?: return
        val bytes = exportValue
        exportResult = null
        exportValue = null
        if (resultCode != Activity.RESULT_OK || data?.data == null || bytes == null) {
            bytes?.fill(0)
            result.success(false)
            return
        }
        val uri = data.data ?: return
        worker.execute {
            try {
                contentResolver.openOutputStream(uri, "wt")?.use { it.write(bytes) }
                    ?: throw IllegalStateException("No destination")
                runOnUiThread { result.success(true) }
            } catch (_: Exception) {
                runOnUiThread { result.error("export_failed", "Could not write selected document", null) }
            } finally { bytes.fill(0) }
        }
    }
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "org.mealshop/documents")
            .setMethodCallHandler { call, result ->
                if (call.method != "export" || exportResult != null) {
                    result.error("export_failed", "Export unavailable", null)
                } else {
                    val value = call.argument<String>("value")?.toByteArray(Charsets.UTF_8)
                    if (value == null || value.size > 4194304) {
                        result.error("export_failed", "Export exceeds limit", null)
                    } else {
                        exportResult = result
                        exportValue = value
                        try {
                            startActivityForResult(Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                                addCategory(Intent.CATEGORY_OPENABLE)
                                type = "application/json"
                                putExtra(Intent.EXTRA_TITLE, "mealshop-activities.json")
                            }, 508)
                        } catch (_: Exception) {
                            exportResult = null
                            exportValue?.fill(0)
                            exportValue = null
                            result.error("export_failed", "No document destination available", null)
                        }
                    }
                }
            }
        val vault = DeviceVault(applicationContext)
        val main = Handler(Looper.getMainLooper())
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "org.mealshop/device_vault")
            .setMethodCallHandler { call, result ->
                worker.execute {
                    try {
                        val namespace = requireNotNull(call.argument<String>("namespace"))
                        val value = when (call.method) {
                            "read" -> vault.read(namespace)
                            "write" -> { vault.write(namespace, requireNotNull(call.argument<String>("value"))); null }
                            "clear" -> { vault.clear(namespace); null }
                            else -> throw IllegalArgumentException("Unsupported operation")
                        }
                        main.post { result.success(value) }
                    } catch (_: Exception) {
                        main.post { result.error("vault_failed", "Protected storage operation failed", null) }
                    }
                }
            }
    }
}
