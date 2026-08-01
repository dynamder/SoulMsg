// Kotlin (JVM) verification of the SoulMsg envelope against the Rust golden dump,
// using the `soulmsg` binding.
//
// Uses protobuf-java descriptors + DynamicMessage (no code generation needed).
// Run with:
//   kotlinc SoulMsg.kt Verify.kt -include-runtime -d verify.jar \
//     -cp protobuf-java.jar:bcprov-jdk18on.jar:json.jar:kotlin-stdlib.jar:kotlin-reflect.jar
//   java -cp verify.jar;protobuf-java.jar;bcprov-jdk18on.jar;json.jar;kotlin-stdlib.jar;kotlin-reflect.jar \
//     VerifyKt <verif-dir>
//
// verif-dir contains descriptors.pb and golden.json.

import com.google.protobuf.DescriptorProtos
import com.google.protobuf.DynamicMessage
import org.json.JSONObject
import soulmsg.Envelope
import soulmsg.Policy
import soulmsg.Schema
import java.io.File

fun main(args: Array<String>) {
    val verif = File(if (args.isNotEmpty()) args[0] else "..")
    val schema = Schema.fromDescriptorFile(File(verif, "descriptors.pb").absolutePath)

    val golden = JSONObject(File(verif, "golden.json").readText())
    val messages = golden.getJSONObject("messages")
    val fd = com.google.protobuf.Descriptors.FileDescriptor.buildFrom(
        DescriptorProtos.FileDescriptorSet.parseFrom(File(verif, "descriptors.pb").readBytes()).fileList[0],
        arrayOf(),
    )
    var failures = 0

    val it = messages.keys()
    while (it.hasNext()) {
        val fullName = it.next()
        val expected = messages.getJSONObject(fullName)
        val meta = schema.byName(fullName) ?: error("message not found: $fullName")

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
        val msg = builder.build()
        val wire = Envelope.new(msg, meta)
        val roundtripped = Envelope.tryDeserialize(wire, meta, DynamicMessage.newBuilder(md), Policy.STRICT)
        check(roundtripped == msg) { "roundtrip mismatch for $fullName" }

        val checks = mapOf(
            "name_hash" to hex(meta.nameHash),
            "version_hash" to hex(meta.versionHash),
            "payload" to hex(msg.toByteArray()),
            "envelope" to hex(wire),
        )
        var ok = true
        for ((k, v) in checks) {
            if (v != expected.getString(k)) {
                println("[kotlin] $fullName.$k MISMATCH\n  rust = ${expected.getString(k)}\n  kot  = $v")
                ok = false
                failures++
            }
        }
        println(if (ok) "[kotlin] $fullName: match (via soulmsg binding)" else "[kotlin] $fullName: MISMATCH")
    }

    if (failures > 0) {
        println("\n[kotlin] FAIL: $failures mismatches")
        kotlin.system.exitProcess(1)
    }
    println("\n[kotlin] PASS: byte-for-byte identical to Rust (via soulmsg binding)")
}

private fun hex(b: ByteArray): String = b.joinToString("") { "%02x".format(it) }
