package org.mealshop.pocket.dev

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
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
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
