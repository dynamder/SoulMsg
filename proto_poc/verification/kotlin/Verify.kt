// Kotlin (JVM) verification of the smsg_proto envelope against the Rust golden dump.
//
// Uses protobuf-java descriptors + DynamicMessage (no code generation needed) and
// BouncyCastle BLAKE3. Run with:
//   kotlinc Verify.kt -include-runtime -d verify.jar -cp protobuf-java.jar:bcprov-jdk18on.jar
//   java -cp verify.jar;protobuf-java.jar;bcprov-jdk18on.jar;kotlin-stdlib.jar VerifyKt
//
// Paths: descriptors.pb and golden.json are one level above this file.

import com.google.protobuf.DescriptorProtos
import com.google.protobuf.DynamicMessage
import java.io.File

private val NAME_DOMAIN = "smsg_proto:name:v1\u0000".toByteArray(Charsets.UTF_8)
private val VERSION_DOMAIN = "smsg_proto:version:v1\u0000".toByteArray(Charsets.UTF_8)

private fun u32le(n: Int): ByteArray = byteArrayOf(
    (n and 0xff).toByte(), ((n ushr 8) and 0xff).toByte(),
    ((n ushr 16) and 0xff).toByte(), ((n ushr 24) and 0xff).toByte(),
)

private fun seg(b: ByteArray): ByteArray = u32le(b.size) + b

private fun blake3(vararg chunks: ByteArray): ByteArray {
    val d = org.bouncycastle.crypto.digests.Blake3Digest()
    for (c in chunks) d.update(c, 0, c.size)
    val out = ByteArray(32)
    d.doFinal(out, 0)
    return out
}

private fun nameHash(short: String): ByteArray =
    blake3(NAME_DOMAIN, seg(short.toByteArray(Charsets.UTF_8)))

private fun versionHash(full: String, msg: DescriptorProtos.DescriptorProto): ByteArray {
    val chunks = ArrayList<ByteArray>()
    chunks.add(VERSION_DOMAIN)
    chunks.add(seg(full.toByteArray(Charsets.UTF_8)))
    chunks.add(u32le(msg.fieldCount))
    for (f in msg.fieldList) {
        chunks.add(seg(f.name.toByteArray(Charsets.UTF_8)))
        chunks.add(u32le(f.number))
        chunks.add(u32le(f.type.number))
        if (f.typeName.isNotEmpty()) chunks.add(seg(f.typeName.toByteArray(Charsets.UTF_8)))
    }
    return blake3(*chunks.toTypedArray())
}

private fun envelope(name: ByteArray, version: ByteArray, payload: ByteArray): ByteArray =
    name + version + u32le(payload.size) + payload

private fun hex(b: ByteArray): String = b.joinToString("") { "%02x".format(it) }

fun main(args: Array<String>) {
    // verif dir (containing descriptors.pb and golden.json) passed as the first argument.
    val verif = File(if (args.isNotEmpty()) args[0] else "..")
    val fds = DescriptorProtos.FileDescriptorSet.parseFrom(File(verif, "descriptors.pb").readBytes())
    val golden = org.json.JSONObject(File(verif, "golden.json").readText())
    val messages = golden.getJSONObject("messages")

    val fd = com.google.protobuf.Descriptors.FileDescriptor.buildFrom(fds.fileList[0], arrayOf())
    var failures = 0

    // golden message order is fixed by the JSON; iterate its keys.
    val it = messages.keys()
    while (it.hasNext()) {
        val fullName = it.next()
        val expected = messages.getJSONObject(fullName)
        val short = fullName.substringAfterLast('.')

        val msgDesc = findMessageProto(fds, fullName)
        val nameH = nameHash(short)
        val versionH = versionHash(fullName, msgDesc)

        val md = fd.messageTypes.first { m -> m.fullName == fullName }
        val builder = DynamicMessage.newBuilder(md)
        when (fullName) {
            "chat.ChatMessage" -> {
                builder.setField(md.findFieldByName("sender"), "Alice")
                builder.setField(md.findFieldByName("content"), "Hello, World!")
                builder.setField(md.findFieldByName("timestamp"), 1_699_999_999L)
            }
            "chat.RobotState" -> {
                val posMd = md.findFieldByName("position").messageType
                val pos = DynamicMessage.newBuilder(posMd)
                    .setField(posMd.findFieldByName("x"), 100.5)
                    .setField(posMd.findFieldByName("y"), 200.5)
                    .setField(posMd.findFieldByName("z"), 300.5)
                    .build()
                builder.setField(md.findFieldByName("name"), "R2-D2")
                builder.setField(md.findFieldByName("position"), pos)
                builder.setField(md.findFieldByName("status"), 42)
            }
        }
        val payload = builder.build().toByteArray()
        val envelopeBytes = envelope(nameH, versionH, payload)

        val checks = mapOf(
            "name_hash" to hex(nameH),
            "version_hash" to hex(versionH),
            "payload" to hex(payload),
            "envelope" to hex(envelopeBytes),
        )
        var ok = true
        for ((k, v) in checks) {
            if (v != expected.getString(k)) {
                println("[kotlin] $fullName.$k MISMATCH\n  rust = ${expected.getString(k)}\n  kot  = $v")
                ok = false
                failures++
            }
        }
        println(if (ok) "[kotlin] $fullName: all match rust" else "[kotlin] $fullName: MISMATCH")
    }

    if (failures > 0) {
        println("\n[kotlin] FAIL: $failures mismatches")
        kotlin.system.exitProcess(1)
    }
    println("\n[kotlin] PASS: byte-for-byte identical to Rust for all messages")
}

private fun findMessageProto(
    fds: DescriptorProtos.FileDescriptorSet,
    fullName: String,
): DescriptorProtos.DescriptorProto {
    for (f in fds.fileList) {
        for (m in f.messageTypeList) {
            if ("${f.`package`}.${m.name}" == fullName) return m
        }
    }
    error("message not found: $fullName")
}
