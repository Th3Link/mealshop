package org.mealshop.pocket.dev

import android.Manifest
import android.app.Activity
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.SystemClock
import android.provider.Settings
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors

/** Only a bounded, encrypted deadline projection; never credentials, recipes or people. */
internal class DeviceReminders(private val context: Context) {
    companion object {
        const val permissionRequest = 509
        private const val namespace = "mealshop.reminders.v1"
        private const val channel = "mealshop.cooking.v1"
        val worker = Executors.newSingleThreadExecutor()
        private var permissionResult: MethodChannel.Result? = null
        private val gateLock = Any()

        fun handle(activity: Activity, call: MethodCall, result: MethodChannel.Result) {
            val bridge = DeviceReminders(activity.applicationContext)
            if (call.method == "requestPrecision") {
                try {
                    if (Build.VERSION.SDK_INT >= 31 && !bridge.alarms.canScheduleExactAlarms()) {
                        activity.startActivity(Intent(Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM, Uri.parse("package:${activity.packageName}")))
                    }
                    result.success(bridge.status())
                } catch (_: Exception) { result.error("reminders_failed", "Precise alarm settings unavailable", null) }
                return
            }
            if (call.method == "requestPermission") {
                if (Build.VERSION.SDK_INT >= 33 && activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
                    if (permissionResult != null) {
                        result.error("reminders_busy", "Permission request already open", null)
                    } else {
                        permissionResult = result
                        activity.requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), permissionRequest)
                    }
                } else { result.success(bridge.status()) }
                return
            }
            // Close the gate synchronously before waiting for I/O during logout/scope changes.
            if (call.method == "clear") bridge.gate(false)
            val epoch = bridge.epoch()
            worker.execute {
                try {
                    val value = when (call.method) {
                        "clock" -> bridge.clock()
                        "status" -> bridge.status()
                        "replace" -> {
                            bridge.replace(requireNotNull(call.argument<List<Map<String, Any>>>("entries")), epoch, call.argument<String>("generation"))
                            bridge.status()
                        }
                        "clear" -> { bridge.clear(); bridge.status() }
                        else -> throw IllegalArgumentException("Unsupported reminder operation")
                    }
                    activity.runOnUiThread { result.success(value) }
                } catch (_: Exception) {
                    activity.runOnUiThread { result.error("reminders_failed", "Device reminders could not be reconciled", null) }
                }
            }
        }

        fun permissionResult(context: Context, request: Int) {
            if (request != permissionRequest) return
            val result = permissionResult
            permissionResult = null
            result?.success(DeviceReminders(context.applicationContext).status())
        }
    }

    private val vault = DeviceVault(context)
    private val manager = context.getSystemService(NotificationManager::class.java)
    private val alarms = context.getSystemService(AlarmManager::class.java)
    private val preferences = context.getSharedPreferences("mealshop.reminder-gate", Context.MODE_PRIVATE)
    private fun enabled() = preferences.getBoolean("enabled", false)
    private fun epoch() = preferences.getLong("epoch", 0)
    private fun gate(value: Boolean, expected: Long? = null): Boolean = synchronized(gateLock) {
        if (expected != null && expected != epoch()) return@synchronized false
        val edit = preferences.edit().putBoolean("enabled", value)
        if (!value) edit.putLong("epoch", Math.addExact(epoch(), 1))
        check(edit.commit())
        true
    }
    private fun now() = System.currentTimeMillis() / 1000
    private fun rows(): List<JSONObject> {
        val raw = vault.read(namespace) ?: return emptyList()
        val root = JSONObject(raw)
        check(root.getInt("version") == 1)
        val entries = root.getJSONArray("entries")
        check(entries.length() <= 200)
        return (0 until entries.length()).map { entries.getJSONObject(it) }
    }
    private fun save(rows: List<JSONObject>) {
        vault.write(namespace, JSONObject().put("version", 1).put("entries", JSONArray(rows)).toString())
    }
    fun clock(): Map<String, Any> = mapOf(
        "wall_ms" to System.currentTimeMillis(), "elapsed_ms" to SystemClock.elapsedRealtime(),
        "boot" to (Settings.Global.getString(context.contentResolver, "boot_count") ?: "unknown"),
        "timezone" to java.util.TimeZone.getDefault().id
    )
    fun status(): Map<String, Any> {
        val granted = (Build.VERSION.SDK_INT < 33 || context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED) &&
            (Build.VERSION.SDK_INT < 24 || manager.areNotificationsEnabled())
        val channelAllowed = Build.VERSION.SDK_INT < 26 || manager.getNotificationChannel(channel)?.importance != NotificationManager.IMPORTANCE_NONE
        val exact = Build.VERSION.SDK_INT < 31 || alarms.canScheduleExactAlarms()
        return mapOf("permission" to if (granted && channelAllowed) "granted" else "denied", "precision" to if (exact) "exact" else "inexact", "enabled" to enabled(), "scope" to (preferences.getString("scope", "") ?: ""),
            "active_notifications" to manager.activeNotifications.size)
    }
    private fun intent(key: String): PendingIntent = PendingIntent.getBroadcast(context, 0,
        Intent(context, ReminderReceiver::class.java).apply { data = Uri.Builder().scheme("mealshop").authority("reminder").appendPath(key).build() },
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    private fun cancel(row: JSONObject) {
        val key = row.getString("key")
        alarms.cancel(intent(key))
        manager.cancel(key, 0)
    }
    private fun generationMatches(generation: String?): Boolean {
        if (generation == null) return true
        if (!generation.matches(Regex("[0-9a-f]{32}"))) return false
        return try {
            val file = java.io.File(DeviceEngine.root(context), "active.json")
            if (file.length() > 4096) false else {
                val value = JSONObject(file.readText())
                value.optInt("schema_version") == 1 && value.optString("id") == generation
            }
        } catch (_: Exception) { false }
    }
    fun replace(input: List<Map<String, Any>>, expected: Long, generation: String?) {
        if (expected != epoch()) return
        if (!generationMatches(generation)) { clear(); return }
        require(input.size <= 200)
        val previous = rows()
        val instant = now()
        val keys = mutableSetOf<String>()
        val next = input.map { raw ->
            val key = raw["key"] as String
            val due = (raw["due"] as Number).toLong()
            val expires = (raw["expires_at"] as Number).toLong()
            val language = raw["language"] as String
            require(key.matches(Regex("[a-zA-Z0-9_.:-]{1,384}")) && keys.add(key))
            require(due in 946684800..4102444800 && expires >= due && expires <= instant + 604805 && expires > instant)
            require(language == "en" || language == "de")
            val attempted = previous.firstOrNull { it.getString("key") == key && it.getLong("due") == due }?.optBoolean("attempted", false) ?: false
            JSONObject().put("key", key).put("due", due).put("expires_at", expires).put("language", language).put("attempted", attempted)
        }
        save(next)
        check(preferences.edit().putString("local_generation", generation).commit())
        if (!gate(true, expected)) return
        check(preferences.edit().putString("scope", next.firstOrNull()?.getString("key")?.substringBefore(":" ) ?: "").commit())
        for (old in previous) if (next.none { it.getString("key") == old.getString("key") && it.getLong("due") == old.getLong("due") }) cancel(old)
        reconcile()
    }
    fun clear() {
        gate(false)
        try { rows().forEach { cancel(it) } } finally {
            manager.cancelAll()
            if (Build.VERSION.SDK_INT >= 34) alarms.cancelAll()
            vault.clear(namespace)
        }
    }
    fun reconcile(key: String? = null) {
        if (!enabled()) return
        if (!generationMatches(preferences.getString("local_generation", null))) { clear(); return }
        val current = rows()
        val instant = now()
        for (row in current) {
            if (row.getLong("expires_at") <= instant) { cancel(row); continue }
            if (row.optBoolean("attempted", false)) continue
            if (status()["permission"] != "granted") { cancel(row); continue }
            val due = row.getLong("due")
            if (due > instant) {
                if (Build.VERSION.SDK_INT < 31 || alarms.canScheduleExactAlarms()) {
                    alarms.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, due * 1000, intent(row.getString("key")))
                } else {
                    alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, due * 1000, intent(row.getString("key")))
                }
            } else if (key == null || key == row.getString("key")) {
                if (instant - due > 86400) {
                    row.put("attempted", true); save(current)
                } else if (status()["permission"] == "granted") {
                    // Save attempt first: a crash must not create a blind duplicate alarm.
                    row.put("attempted", true); save(current)
                    notify(row)
                }
            }
        }
    }
    @Suppress("DEPRECATION")
    private fun notify(row: JSONObject) {
        if (Build.VERSION.SDK_INT >= 26) manager.createNotificationChannel(NotificationChannel(channel, "Mealshop", NotificationManager.IMPORTANCE_DEFAULT))
        val launch = PendingIntent.getActivity(context, 0, Intent(context, MainActivity::class.java), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val builder = if (Build.VERSION.SDK_INT >= 26) Notification.Builder(context, channel) else Notification.Builder(context)
        val german = row.getString("language") == "de"
        val notification = builder.setSmallIcon(R.drawable.mealshop_mark).setContentTitle("Mealshop")
            .setContentText(if (german) "Ein Koch- oder Brauschritt ist fällig. Öffne die App zum Prüfen." else "A cooking or brewing action is due. Open the app to review.")
            .setVisibility(Notification.VISIBILITY_PRIVATE).setAutoCancel(true).setContentIntent(launch)
            .setWhen(row.getLong("due") * 1000).build()
        manager.notify(row.getString("key"), 0, notification)
    }
}

class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val key = intent.data?.takeIf { it.scheme == "mealshop" && it.authority == "reminder" }?.lastPathSegment
        if (key == null && intent.action !in listOf(Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_TIME_CHANGED, Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_MY_PACKAGE_REPLACED, AlarmManager.ACTION_SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED)) return
        val pending = goAsync()
        DeviceReminders.worker.execute {
            try { DeviceReminders(context.applicationContext).reconcile(key) }
            catch (_: Exception) { /* The app reports reconciliation failure on the next foreground check. */ }
            finally { pending.finish() }
        }
    }
}
