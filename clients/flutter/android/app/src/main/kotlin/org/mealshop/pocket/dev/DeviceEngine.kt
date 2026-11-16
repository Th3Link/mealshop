package org.mealshop.pocket.dev

import android.content.Context
import android.os.Handler
import android.os.Looper
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.json.JSONObject
import java.io.BufferedInputStream
import java.io.BufferedWriter
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.OutputStreamWriter
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

/** Only this serialized channel owns the installed core's private stdin/stdout. */
internal object DeviceEngine {
    val worker = Executors.newSingleThreadExecutor()
    private val watchdog = Executors.newSingleThreadScheduledExecutor()
    private val main = Handler(Looper.getMainLooper())
    private const val limit = 16 * 1024 * 1024
    private var process: Process? = null
    private var input: BufferedWriter? = null
    private var output: BufferedInputStream? = null
    private var sequence = 0L

    private class Failure(val code: String) : Exception()

    fun root(context: Context): File = File(context.noBackupFilesDir.canonicalFile, "mealshop-app-only")

    fun handle(context: Context, call: MethodCall, result: MethodChannel.Result) {
        if (call.method == "notices") {
            worker.execute {
                try {
                    val text = context.assets.open("mealshop-core-notices.txt").bufferedReader().use { it.readText() }
                    main.post { result.success(text) }
                } catch (_: Exception) {
                    main.post { result.error("device_unavailable", "Native notices unavailable", null) }
                }
            }
            return
        }
        if (call.method != "command") {
            result.notImplemented()
            return
        }
        val value = call.argument<String>("value")
        if (value == null || value.toByteArray(Charsets.UTF_8).size > limit) {
            result.error("device_request_invalid", "Invalid on-device request", null)
            return
        }
        worker.execute {
            try {
                val answer = command(context.applicationContext, JSONObject(value))
                main.post { result.success(answer.toString()) }
            } catch (error: Failure) {
                main.post { result.error(error.code, "On-device operation failed", null) }
            } catch (_: Exception) {
                close()
                main.post { result.error("device_engine_failed", "On-device operation failed", null) }
            }
        }
    }

    private fun command(context: Context, command: JSONObject): JSONObject {
        val shutdown = command.optString("operation") == "shutdown"
        if (shutdown && process == null) return JSONObject().put("stopped", true)
        start(context)
        val current = process ?: throw Failure("device_engine_failed")
        val timeout = if (command.optString("operation") == "request") 45L else 300L
        val alarm = watchdog.schedule({ current.destroy() }, timeout, TimeUnit.SECONDS)
        try {
            if (sequence == Long.MAX_VALUE) throw Failure("device_engine_failed")
            sequence += 1
            val frame = JSONObject().put("id", sequence).put("command", command).toString()
            input?.apply { write(frame); newLine(); flush() } ?: throw Failure("device_engine_failed")
            val answer = JSONObject(line())
            if (answer.getLong("id") != sequence) throw Failure("device_protocol_failed")
            if (!answer.getBoolean("ok")) {
                val code = answer.optString("error").takeIf { it.matches(Regex("[a-z0-9_]{1,80}")) }
                    ?: "device_engine_failed"
                throw Failure(code)
            }
            val value = answer.getJSONObject("value")
            if (command.optString("operation") == "commit_restore" && value.optBoolean("restored")) {
                val cleared = try {
                    DeviceReminders.worker.submit<Boolean> { DeviceReminders(context).clear(); true }.get(30,TimeUnit.SECONDS)
                } catch (_: Exception) { false }
                value.put("reminders_cleared",cleared)
            }
            if (shutdown) close()
            return value
        } finally { alarm.cancel(false) }
    }

    fun start(context: Context) {
        if (process != null) {
            try { process?.exitValue(); close() } catch (_: IllegalThreadStateException) { return }
        }
        val binary = File(context.applicationInfo.nativeLibraryDir, "libmealshop_core.so")
        if (!binary.isFile || !binary.canExecute()) throw Failure("device_unavailable")
        val child = ProcessBuilder(binary.absolutePath, "--device-stdio", root(context).absolutePath)
            .directory(context.noBackupFilesDir)
            .redirectError(File("/dev/null"))
            .start()
        process = child
        input = BufferedWriter(OutputStreamWriter(child.outputStream, Charsets.UTF_8))
        output = BufferedInputStream(child.inputStream)
        val alarm = watchdog.schedule({ child.destroy() }, 30, TimeUnit.SECONDS)
        try {
            val ready = JSONObject(line())
            if (ready.optInt("protocol") != 1 || ready.optString("kind") != "device") {
                throw Failure("device_protocol_failed")
            }
            if (!ready.has("storage")) {
                throw Failure(ready.optString("error").takeIf { it.matches(Regex("[a-z0-9_]{1,80}")) }
                    ?: "device_engine_failed")
            }
        } catch (error: Exception) {
            close()
            throw error
        } finally { alarm.cancel(false) }
    }

    private fun line(): String {
        val stream = output ?: throw Failure("device_engine_failed")
        val bytes = ByteArrayOutputStream()
        while (true) {
            val next = stream.read()
            if (next == -1) throw Failure("device_engine_failed")
            if (next == 10) return bytes.toString(Charsets.UTF_8.name())
            if (bytes.size() >= limit) throw Failure("device_response_too_large")
            bytes.write(next)
        }
    }

    private fun close() {
        try { input?.close() } catch (_: Exception) { /* The owned process may already have exited. */ }
        // Closing stdin lets Rust drop its stores before the next dataset owner starts.
        val child = process
        if (child != null) {
            for (attempt in 0 until 100) {
                try { child.exitValue(); break } catch (_: IllegalThreadStateException) { Thread.sleep(20) }
            }
        }
        try { output?.close() } catch (_: Exception) { /* No dataset is altered on transport failure. */ }
        process?.destroy()
        process = null
        input = null
        output = null
    }
}
