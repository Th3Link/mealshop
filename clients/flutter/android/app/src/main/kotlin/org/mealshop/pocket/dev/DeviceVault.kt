package org.mealshop.pocket.dev

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.system.Os
import android.system.OsConstants
import android.util.AtomicFile
import java.io.ByteArrayOutputStream
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** One encrypted record; return only after file contents AND directory metadata are synced. */
internal class DeviceVault(private val context: Context) {
    private val limit = 1_048_576
    private val header = byteArrayOf(0x4d, 0x53, 0x56, 0x31)
    private fun location(namespace: String): AtomicFile {
        require(namespace == "mealshop.phone.v1" || namespace == "mealshop.phone.integration-test")
        return AtomicFile(File(context.noBackupFilesDir, "$namespace.bin"))
    }
    private fun key(namespace: String, create: Boolean): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        val alias = "mealshop.vault.$namespace"
        val existing = store.getKey(alias, null)
        if (existing is SecretKey) return existing
        check(create) { "Missing protected storage key" }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setRandomizedEncryptionRequired(true)
            .build())
        return generator.generateKey()
    }
    private fun syncDirectory() {
        val directory = Os.open(context.noBackupFilesDir.path, OsConstants.O_RDONLY, 0)
        try { Os.fsync(directory) } finally { Os.close(directory) }
    }
    fun read(namespace: String): String? {
        val file = location(namespace)
        if (!file.baseFile.exists() && !File(file.baseFile.path + ".bak").exists()) return null
        val bytes = file.openRead().use { input ->
            val result = ByteArrayOutputStream()
            val buffer = ByteArray(8192)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) break
                check(result.size() + count <= limit + 32)
                result.write(buffer, 0, count)
            }
            result.toByteArray()
        }
        check(bytes.size >= 32 && bytes.copyOfRange(0, 4).contentEquals(header))
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key(namespace, false), GCMParameterSpec(128, bytes.copyOfRange(4, 16)))
        cipher.updateAAD(namespace.toByteArray(Charsets.UTF_8))
        return cipher.doFinal(bytes, 16, bytes.size - 16).toString(Charsets.UTF_8)
    }
    fun write(namespace: String, value: String) {
        val file = location(namespace)
        val plain = value.toByteArray(Charsets.UTF_8)
        require(plain.size <= limit)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key(namespace, true))
        check(cipher.iv.size == 12)
        cipher.updateAAD(namespace.toByteArray(Charsets.UTF_8))
        val encrypted = header + cipher.iv + cipher.doFinal(plain)
        val output = file.startWrite()
        try {
            output.write(encrypted)
            output.fd.sync()
            file.finishWrite(output)
            check(!File(file.baseFile.path + ".new").exists())
            check(file.baseFile.readBytes().contentEquals(encrypted))
            Os.chmod(file.baseFile.path, 0x180) // 0600
            syncDirectory()
        } catch (failure: Exception) {
            file.failWrite(output)
            throw failure
        }
    }
    fun clear(namespace: String) {
        val file = location(namespace)
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        // Destroy the key first: any interrupted deletion leaves only unreadable ciphertext.
        store.deleteEntry("mealshop.vault.$namespace")
        file.delete()
        check(!file.baseFile.exists() && !File(file.baseFile.path + ".bak").exists() && !File(file.baseFile.path + ".new").exists())
        syncDirectory()
    }
}
