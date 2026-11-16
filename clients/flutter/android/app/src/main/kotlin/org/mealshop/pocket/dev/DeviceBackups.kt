package org.mealshop.pocket.dev

import android.app.Activity
import android.content.Intent
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.util.UUID

/** Android owns document permissions; only encrypted files cross this boundary. */
internal class DeviceBackups(private val activity: MainActivity) {
    private var pending: MethodChannel.Result? = null
    private var exported: String? = null
    private val maximum = 512L * 1024 * 1024

    fun handle(call: MethodCall, result: MethodChannel.Result) {
        if (pending != null) { result.error("device_document_busy", "Document picker is active", null); return }
        val intent: Intent
        val code: Int
        when (call.method) {
            "export" -> {
                val id = call.argument<String>("id")
                if (id == null || !id.matches(Regex("[0-9a-f]{32}"))) {
                    result.error("device_request_invalid", "Invalid backup identifier", null); return
                }
                exported = id
                code = 509
                intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "application/octet-stream"
                    putExtra(Intent.EXTRA_TITLE, "mealshop-backup-${id.take(8)}.age")
                }
            }
            "import" -> {
                code = 510
                intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "*/*"
                }
            }
            else -> { result.notImplemented(); return }
        }
        pending = result
        try { activity.startActivityForResult(intent, code) } catch (_: Exception) {
            pending = null; exported = null
            result.error("device_document_failed", "Document picker unavailable", null)
        }
    }

    fun activityResult(code: Int, outcome: Int, data: Intent?): Boolean {
        if (code != 509 && code != 510) return false
        val result = pending ?: return true
        val id = exported
        pending = null; exported = null
        val uri = data?.data
        if (outcome != Activity.RESULT_OK || uri == null) { result.success(null); return true }
        DeviceEngine.worker.execute {
            var partial: File? = null
            try {
                // Serialized with core requests, including startup transfer cleanup.
                DeviceEngine.start(activity.applicationContext)
                val root = DeviceEngine.root(activity.applicationContext)
                if (code == 509 && id != null) {
                    val source = File(root, "exports/$id.age")
                    if (!source.isFile || source.canonicalFile != source.absoluteFile || source.length() > maximum) {
                        throw IllegalStateException()
                    }
                    FileInputStream(source).use { input ->
                        activity.contentResolver.openOutputStream(uri, "wt")?.use { output ->
                            input.copyTo(output, 65536)
                            output.flush()
                        } ?: throw IllegalStateException()
                    }
                    // Closing the provider stream is part of success, not just opening its picker.
                    activity.runOnUiThread { result.success(true) }
                } else {
                    val imported = UUID.randomUUID().toString().replace("-", "")
                    val destination = File(root, "imports/$imported.age")
                    val temporary = File(root, "imports/$imported.pending")
                    partial = temporary
                    if (!temporary.createNewFile()) throw IllegalStateException()
                    temporary.setReadable(false, false); temporary.setWritable(false, false)
                    temporary.setReadable(true, true); temporary.setWritable(true, true)
                    FileOutputStream(temporary).use { output ->
                        activity.contentResolver.openInputStream(uri)?.use { input ->
                            val bytes = ByteArray(65536)
                            var total = 0L
                            while (true) {
                                val count = input.read(bytes)
                                if (count == -1) break
                                total += count
                                if (total > maximum) throw IllegalStateException()
                                output.write(bytes, 0, count)
                            }
                        } ?: throw IllegalStateException()
                        output.fd.sync()
                    }
                    if (!temporary.renameTo(destination)) throw IllegalStateException()
                    partial = null
                    activity.runOnUiThread { result.success(imported) }
                }
            } catch (_: Exception) {
                activity.runOnUiThread { result.error("device_document_failed", "Could not transfer encrypted backup", null) }
            } finally { partial?.delete() }
        }
        return true
    }
}
