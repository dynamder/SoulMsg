"""SoulMsg Python binding: schema-verified protobuf messages with a hash envelope.

The payload is encoded by the caller's protobuf runtime; this module only adds the
canonical name/version hashes and the envelope framing, so the produced bytes are
byte-identical across all SoulMsg languages.

Wire layout: [name_hash:32][version_hash:32][payload_len:u32(LE)][payload]
"""

from __future__ import annotations

import enum
import struct
from typing import Optional

import blake3  # pip install blake3
from google.protobuf import descriptor_pb2  # pip install protobuf

NAME_DOMAIN = b"smsg_proto:name:v1\0"
VERSION_DOMAIN = b"smsg_proto:version:v1\0"
HEADER_LEN = 68


class Policy(enum.Enum):
    STRICT = "strict"  # reject hash mismatches (default)
    LENIENT = "lenient"  # skip hash checks, rely on protobuf's tolerant evolution


class SmsgError(Exception):
    pass


class NotAnEnvelope(SmsgError):
    pass


class TypeMismatch(SmsgError):
    def __init__(self, expected, actual):
        super().__init__("name_hash mismatch")
        self.expected = expected
        self.actual = actual


class VersionMismatch(SmsgError):
    def __init__(self, expected, actual):
        super().__init__("version_hash mismatch")
        self.expected = expected
        self.actual = actual


class DeserializeError(SmsgError):
    pass


def _seg(b: bytes) -> bytes:
    return struct.pack("<I", len(b)) + b


def _blake3(*chunks: bytes) -> bytes:
    h = blake3.blake3()
    for c in chunks:
        h.update(c)
    return h.digest()


def name_hash(short_name: str) -> bytes:
    return _blake3(NAME_DOMAIN, _seg(short_name.encode("utf-8")))


def version_hash(full_name: str, msg_desc) -> bytes:
    chunks = [VERSION_DOMAIN, _seg(full_name.encode("utf-8"))]
    chunks.append(struct.pack("<I", len(msg_desc.field)))
    for f in msg_desc.field:
        chunks.append(_seg(f.name.encode("utf-8")))
        chunks.append(struct.pack("<I", f.number))
        chunks.append(struct.pack("<I", f.type))
        if f.type_name:
            chunks.append(_seg(f.type_name.encode("utf-8")))
    return _blake3(*chunks)


class MessageRef:
    __slots__ = ("full_name", "name_hash", "version_hash")

    def __init__(self, full_name: str, name_hash: bytes, version_hash: bytes):
        self.full_name = full_name
        self.name_hash = name_hash
        self.version_hash = version_hash


class Schema:
    def __init__(self, messages):
        self.messages = messages
        self._by_name = {m.full_name: m for m in messages}
        self._by_name_hash = {m.name_hash: m for m in messages}

    @classmethod
    def from_descriptor_bytes(cls, data: bytes) -> "Schema":
        fds = descriptor_pb2.FileDescriptorSet.FromString(data)
        messages = []
        for f in fds.file:
            for m in f.message_type:
                full = f"{f.package}.{m.name}" if f.package else m.name
                messages.append(
                    MessageRef(
                        full_name=full,
                        name_hash=name_hash(m.name),
                        version_hash=version_hash(full, m),
                    )
                )
        return cls(messages)

    @classmethod
    def from_descriptor_file(cls, path: str) -> "Schema":
        with open(path, "rb") as fh:
            return cls.from_descriptor_bytes(fh.read())

    def by_name(self, full_name: str) -> Optional[MessageRef]:
        return self._by_name.get(full_name)

    def by_name_hash(self, name_hash: bytes) -> Optional[MessageRef]:
        return self._by_name_hash.get(name_hash)


class Envelope:
    @staticmethod
    def new(msg, meta: MessageRef) -> bytes:
        payload = msg.SerializeToString()
        return meta.name_hash + meta.version_hash + struct.pack("<I", len(payload)) + payload

    @staticmethod
    def try_deserialize(data: bytes, meta: MessageRef, msg_cls, policy: Policy = Policy.STRICT):
        if len(data) < HEADER_LEN:
            raise NotAnEnvelope("data too short for the envelope header")
        name_h = data[0:32]
        version_h = data[32:64]
        if policy is Policy.STRICT:
            if name_h != meta.name_hash:
                raise TypeMismatch(meta.name_hash, name_h)
            if version_h != meta.version_hash:
                raise VersionMismatch(meta.version_hash, version_h)
        plen = struct.unpack("<I", data[64:68])[0]
        if len(data) != HEADER_LEN + plen:
            raise NotAnEnvelope(
                f"payload length mismatch: header says {plen} but {len(data) - HEADER_LEN} remain"
            )
        msg = msg_cls()
        msg.ParseFromString(data[HEADER_LEN:])
        return msg

    @staticmethod
    def peek(data: bytes):
        if len(data) < HEADER_LEN:
            raise NotAnEnvelope("data too short for the envelope header")
        return data[0:32], data[32:64]
