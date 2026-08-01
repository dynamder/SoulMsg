// SoulMsg Kotlin (JVM) binding: schema-verified protobuf messages with a hash
// envelope. Adds the canonical name/version hashes and envelope framing on top of
// the caller's protobuf runtime, producing byte-identical output to every other
// SoulMsg language.
//
// Wire layout: [name_hash:32][version_hash:32][payload_len:u32(LE)][payload]
package soulmsg

import com.google.protobuf.DescriptorProtos
import org.bouncycastle.crypto.digests.Blake3Digest
import java.io.File

enum class Policy { STRICT, LENIENT }

sealed class SmsgError(message: String) : Exception(message) {
    class NotAnEnvelope(message: String) : SmsgError(message)
    class TypeMismatch(expected: ByteArray, actual: ByteArray) :
        SmsgError("name_hash mismatch") {
        val expected = expected.copyOf()
        val actual = actual.copyOf()
    }
    class VersionMismatch(expected: ByteArray, actual: ByteArray) :
        SmsgError("version_hash mismatch") {
        val expected = expected.copyOf()
        val actual = actual.copyOf()
    }
    class DeserializeError(message: String) : SmsgError(message)
}

data class MessageRef(
    val fullName: String,
    val nameHash: ByteArray,
    val versionHash: ByteArray,
) {
    override fun equals(other: Any?) =
        other is MessageRef &&
            fullName == other.fullName &&
            nameHash.contentEquals(other.nameHash) &&
            versionHash.contentEquals(other.versionHash)

    override fun hashCode() = fullName.hashCode()
}

class Schema(val messages: List<MessageRef>) {
    private val byName = messages.associateBy { it.fullName }
    private val byNameHash = messages.associateBy { it.nameHash.toList() }

    fun byName(fullName: String): MessageRef? = byName[fullName]
    fun byNameHash(nameHash: ByteArray): MessageRef? = byNameHash[nameHash.toList()]

    companion object {
        private val NAME_DOMAIN = "smsg_proto:name:v1\u0000".toByteArray(Charsets.UTF_8)
        private val VERSION_DOMAIN = "smsg_proto:version:v1\u0000".toByteArray(Charsets.UTF_8)

        fun fromDescriptorBytes(data: ByteArray): Schema {
            val fds = DescriptorProtos.FileDescriptorSet.parseFrom(data)
            val messages = mutableListOf<MessageRef>()
            for (f in fds.fileList) {
                for (m in f.messageTypeList) {
                    val full = if (f.`package`.isEmpty()) m.name else "${f.`package`}.${m.name}"
                    messages.add(
                        MessageRef(
                            fullName = full,
                            nameHash = nameHash(m.name),
                            versionHash = versionHash(full, m),
                        )
                    )
                }
            }
            return Schema(messages)
        }

        fun fromDescriptorFile(path: String): Schema = fromDescriptorBytes(File(path).readBytes())

        fun nameHash(short: String): ByteArray {
            val chunks = listOf(NAME_DOMAIN, seg(short.toByteArray(Charsets.UTF_8)))
            return blake3(*chunks.toTypedArray())
        }

        fun versionHash(full: String, msg: DescriptorProtos.DescriptorProto): ByteArray {
            val chunks = mutableListOf(ByteArray(0), ByteArray(0), ByteArray(0))
            chunks[0] = VERSION_DOMAIN
            chunks[1] = seg(full.toByteArray(Charsets.UTF_8))
            chunks[2] = u32le(msg.fieldCount)
            for (f in msg.fieldList) {
                chunks.add(seg(f.name.toByteArray(Charsets.UTF_8)))
                chunks.add(u32le(f.number))
                chunks.add(u32le(f.type.number))
                if (f.typeName.isNotEmpty()) chunks.add(seg(f.typeName.toByteArray(Charsets.UTF_8)))
            }
            return blake3(*chunks.toTypedArray())
        }
    }
}

object Envelope {
    const val HEADER_LEN = 68

    /** Wraps a message and returns the full envelope bytes. */
    fun new(msg: com.google.protobuf.Message, meta: MessageRef): ByteArray {
        val payload = msg.toByteArray()
        return meta.nameHash + meta.versionHash + u32le(payload.size) + payload
    }

    /** Decodes an envelope into the given builder under the given policy. */
    fun tryDeserialize(
        data: ByteArray,
        meta: MessageRef,
        builder: com.google.protobuf.Message.Builder,
        policy: Policy = Policy.STRICT,
    ): com.google.protobuf.Message {
        if (data.size < HEADER_LEN) throw SmsgError.NotAnEnvelope("data too short for the envelope header")
        val nameH = data.copyOfRange(0, 32)
        val versionH = data.copyOfRange(32, 64)
        if (policy == Policy.STRICT) {
            if (!nameH.contentEquals(meta.nameHash)) throw SmsgError.TypeMismatch(meta.nameHash, nameH)
            if (!versionH.contentEquals(meta.versionHash)) throw SmsgError.VersionMismatch(meta.versionHash, versionH)
        }
        val plen = ((data[64].toInt() and 0xff)) or
            ((data[65].toInt() and 0xff) shl 8) or
            ((data[66].toInt() and 0xff) shl 16) or
            ((data[67].toInt() and 0xff) shl 24)
        if (data.size != HEADER_LEN + plen) {
            throw SmsgError.NotAnEnvelope(
                "payload length mismatch: header says $plen but ${data.size - HEADER_LEN} remain"
            )
        }
        try {
            return builder.mergeFrom(data, HEADER_LEN, plen).build()
        } catch (e: Exception) {
            throw SmsgError.DeserializeError(e.message ?: "deserialize failed")
        }
    }

    /** Extracts the two hashes from an envelope without decoding the payload. */
    fun peek(data: ByteArray): Pair<ByteArray, ByteArray> {
        if (data.size < HEADER_LEN) throw SmsgError.NotAnEnvelope("data too short for the envelope header")
        return data.copyOfRange(0, 32) to data.copyOfRange(32, 64)
    }
}

internal fun u32le(n: Int): ByteArray = byteArrayOf(
    (n and 0xff).toByte(),
    ((n ushr 8) and 0xff).toByte(),
    ((n ushr 16) and 0xff).toByte(),
    ((n ushr 24) and 0xff).toByte(),
)

internal fun seg(b: ByteArray): ByteArray = u32le(b.size) + b

internal fun blake3(vararg chunks: ByteArray): ByteArray {
    val d = Blake3Digest()
    for (c in chunks) d.update(c, 0, c.size)
    val out = ByteArray(32)
    d.doFinal(out, 0)
    return out
}
