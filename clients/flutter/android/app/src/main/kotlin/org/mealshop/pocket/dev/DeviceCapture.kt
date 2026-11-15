package org.mealshop.pocket.dev

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import android.media.ExifInterface
import com.googlecode.tesseract.android.TessBaseAPI
import com.google.zxing.*
import com.google.zxing.common.HybridBinarizer
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import com.google.zxing.oned.UPCEReader
import java.util.concurrent.atomic.AtomicBoolean

/** Recognition has no network client, telemetry, account credential or household access. */
class DeviceCapture(private val context: Context) {
    companion object {
        private val worker = Executors.newSingleThreadExecutor()
        private val active = AtomicBoolean(false)
        private val timeoutWorker = Executors.newSingleThreadScheduledExecutor()
        const val INBOX = "mealshop.capture.inbox.v1"
        private val inboxWorker = DeviceReminders.worker
        fun receive(context: Context, intent: Intent?) {
            if (intent?.action != Intent.ACTION_SEND || intent.type != "text/plain") return
            val text = intent.getCharSequenceExtra(Intent.EXTRA_TEXT)?.toString()
            intent.action = null // Activity recreation must not enqueue the same payload again.
            if (text == null || text.length > 4096 || text.isBlank()) return
            inboxWorker.execute {
                try {
                    val vault = DeviceVault(context)
                    // Never overwrite an unreviewed incoming item.
                    if (vault.read(INBOX) == null) vault.write(INBOX, JSONObject().put("text", text).toString())
                } catch (_: Exception) { /* UI will expose unavailable protected storage on peek. */ }
            }
        }
    }
    fun handle(activity: MainActivity, call: MethodCall, result: MethodChannel.Result) {
        if (call.method == "shareText") {
            val text = call.argument<String>("text")
            if (text == null || text.length > 4096) { result.error("capture_invalid", "Invalid shared text", null); return }
            try {
                activity.startActivity(Intent.createChooser(Intent(Intent.ACTION_SEND).apply {
                    type = "text/plain"; putExtra(Intent.EXTRA_TEXT, text)
                }, null))
                result.success(null)
            } catch (_: Exception) { result.error("capture_unavailable", "No share destination", null) }
            return
        }
        if (call.method == "peekShare" || call.method == "clearShare") {
            inboxWorker.execute {
                try {
                    val vault = DeviceVault(context)
                    val text = if (call.method == "clearShare") { vault.clear(INBOX); null }
                        else vault.read(INBOX)?.let { JSONObject(it).getString("text") }
                    activity.runOnUiThread { result.success(text) }
                } catch (_: Exception) { activity.runOnUiThread { result.error("storage_failed", "Cannot read incoming share", null) } }
            }
            return
        }
        if (call.method == "discardPicked") {
            val path = call.argument<String>("path")
            try {
                val file = path?.let { File(it).canonicalFile }
                if (file != null && file.path.startsWith(context.cacheDir.canonicalPath + "/")) file.delete()
                result.success(null)
            } catch (_: Exception) { result.error("capture_failed", "Could not remove cached capture", null) }
            return
        }
        if (call.method != "recognize" || !active.compareAndSet(false, true)) {
            result.error("capture_busy", "Recognition is busy or unsupported", null); return
        }
        val bytes = call.argument<ByteArray>("bytes")
        val turn = call.argument<Int>("rotation") ?: 0
        val barcode = call.argument<Boolean>("barcode") ?: false
        if (bytes == null || bytes.size > 5242880 || turn !in listOf(0, 90, 180, 270)) {
            active.set(false); result.error("capture_invalid", "Capture exceeds supported bounds", null); return
        }
        worker.execute {
            var bitmap: Bitmap? = null
            try {
                bitmap = decode(bytes, turn)
                val value = if (barcode) scan(bitmap) else recognize(bitmap)
                activity.runOnUiThread { result.success(value) }
            } catch (_: Throwable) {
                activity.runOnUiThread { result.error("capture_failed", "Could not recognize this image; enter it manually", null) }
            } finally { bitmap?.recycle(); bytes.fill(0); active.set(false) }
        }
    }
    private fun decode(bytes: ByteArray, turn: Int): Bitmap {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
        require(bounds.outWidth in 1..16384 && bounds.outHeight in 1..16384
            && bounds.outWidth.toLong() * bounds.outHeight <= 64000000)
        var sample = 1
        while (bounds.outWidth / sample > 2500 || bounds.outHeight / sample > 2500) sample *= 2
        val bitmap = requireNotNull(BitmapFactory.decodeByteArray(bytes, 0, bytes.size,
            BitmapFactory.Options().apply { inSampleSize = sample; inPreferredConfig = Bitmap.Config.ARGB_8888 }))
        val exif = try { ExifInterface(ByteArrayInputStream(bytes)).getAttributeInt(ExifInterface.TAG_ORIENTATION, 1) } catch (_: Exception) { 1 }
        val matrix = Matrix()
        when (exif) {
            2 -> matrix.postScale(-1f, 1f)
            3 -> matrix.postRotate(180f)
            4 -> matrix.postScale(1f, -1f)
            5 -> { matrix.postScale(-1f, 1f); matrix.postRotate(270f) }
            6 -> matrix.postRotate(90f)
            7 -> { matrix.postScale(-1f, 1f); matrix.postRotate(90f) }
            8 -> matrix.postRotate(270f)
        }
        matrix.postRotate(turn.toFloat())
        val upright = Bitmap.createBitmap(bitmap, 0, 0, bitmap.width, bitmap.height, matrix, true)
        if (upright !== bitmap) bitmap.recycle()
        return upright
    }
    private fun recognize(bitmap: Bitmap): Map<String, Any?> {
        val directory = File(context.noBackupFilesDir, "ocr-models-87416418")
        val data = File(directory, "tessdata").apply { mkdirs() }
        for (language in listOf("eng", "deu")) {
            val target = File(data, "$language.traineddata")
            if (!target.exists()) {
                val temporary = File(data, "$language.tmp")
                context.assets.open("tessdata/$language.traineddata").use { input -> temporary.outputStream().use { input.copyTo(it) } }
                check(temporary.renameTo(target))
            }
        }
        val api = TessBaseAPI()
        try {
            check(api.init(directory.path, "eng+deu", TessBaseAPI.OEM_LSTM_ONLY))
            api.setImage(bitmap)
            val lock = Any()
            var finished = false
            var timedOut = false
            val timeout = timeoutWorker.schedule({ synchronized(lock) {
                if (!finished) { timedOut = true; api.stop() }
            } }, 20, TimeUnit.SECONDS)
            val text: String
            try {
                api.getHOCRText(0) // This recognition entry supports stop(); UTF8 alone does not.
                synchronized(lock) { finished = true; timeout.cancel(false) }
                check(!timedOut)
                text = api.utF8Text ?: ""
            } finally { synchronized(lock) { finished = true; timeout.cancel(false) } }
            require(text.toByteArray(Charsets.UTF_8).size <= 32768)
            return mapOf("text" to text, "score" to api.meanConfidence(), "engine" to "tesseract-5.5.1-fast-eng-deu")
        } finally { api.recycle() }
    }
    private fun scan(bitmap: Bitmap): Map<String, Any?> {
        val pixels = IntArray(bitmap.width * bitmap.height)
        bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
        val source = RGBLuminanceSource(bitmap.width, bitmap.height, pixels)
        val hints = mapOf(DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.EAN_8,
            BarcodeFormat.EAN_13, BarcodeFormat.UPC_A, BarcodeFormat.UPC_E), DecodeHintType.TRY_HARDER to true)
        val reader = MultiFormatReader()
        val codes = linkedSetOf<String>()
        for (candidate in listOf(source, source.invert())) {
            try {
                val decoded = reader.decode(BinaryBitmap(HybridBinarizer(candidate)), hints)
                codes.add(if (decoded.barcodeFormat == BarcodeFormat.UPC_E) UPCEReader.convertUPCEtoUPCA(decoded.text) else decoded.text)
            }
            catch (_: NotFoundException) {} finally { reader.reset() }
        }
        pixels.fill(0)
        return mapOf("barcodes" to codes.toList())
    }
}
