// Package soulmsg is the Go binding for SoulMsg: schema-verified protobuf
// messages with a hash envelope. It adds the canonical name/version hashes and
// the envelope framing on top of the caller's protobuf runtime, producing
// byte-identical output to every other SoulMsg language.
//
// Wire layout: [name_hash:32][version_hash:32][payload_len:u32(LE)][payload]
package soulmsg

import (
	"encoding/binary"
	"fmt"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"
	"lukechampine.com/blake3"
)

var (
	nameDomain    = []byte("smsg_proto:name:v1\x00")
	versionDomain = []byte("smsg_proto:version:v1\x00")
)

// HeaderLen is the fixed header size: name_hash(32) + version_hash(32) + payload_len(4).
const HeaderLen = 68

// Policy selects the decode-time schema compatibility behavior.
type Policy int

const (
	// Strict rejects name/version hash mismatches (default).
	Strict Policy = iota
	// Lenient skips hash checks and relies on protobuf's tolerant evolution.
	Lenient
)

// Kind classifies envelope errors.
type Kind int

const (
	KindNotAnEnvelope Kind = iota
	KindTypeMismatch
	KindVersionMismatch
	KindDeserialize
)

// Error is a typed SoulMsg envelope error.
type Error struct {
	Kind     Kind
	Expected [32]byte
	Actual   [32]byte
	Message  string
}

func (e *Error) Error() string {
	return e.Message
}

func newErr(kind Kind, msg string) *Error {
	return &Error{Kind: kind, Message: msg}
}

// MessageRef is schema-derived metadata for a message type.
type MessageRef struct {
	FullName    string
	NameHash    [32]byte
	VersionHash [32]byte
}

// Schema is a parsed descriptor set with per-message hashes and lookups.
type Schema struct {
	messages   []MessageRef
	byName     map[string]*MessageRef
	byNameHash map[[32]byte]*MessageRef
}

// NewSchema builds a schema from a protoc FileDescriptorSet.
func NewSchema(bytes []byte) (*Schema, error) {
	var fds descriptorpb.FileDescriptorSet
	if err := proto.Unmarshal(bytes, &fds); err != nil {
		return nil, err
	}
	s := &Schema{byName: map[string]*MessageRef{}, byNameHash: map[[32]byte]*MessageRef{}}
	for _, f := range fds.GetFile() {
		pkg := f.GetPackage()
		for _, m := range f.GetMessageType() {
			short := m.GetName()
			full := short
			if pkg != "" {
				full = pkg + "." + short
			}
			ref := &MessageRef{
				FullName:    full,
				NameHash:    nameHash(short),
				VersionHash: versionHash(full, m),
			}
			s.messages = append(s.messages, *ref)
			s.byName[full] = ref
			s.byNameHash[ref.NameHash] = ref
		}
	}
	return s, nil
}

// Message looks up a message by its full name.
func (s *Schema) Message(fullName string) *MessageRef {
	return s.byName[fullName]
}

// ByNameHash looks up a message by its name hash.
func (s *Schema) ByNameHash(h [32]byte) *MessageRef {
	return s.byNameHash[h]
}

func le32(n int) []byte {
	b := make([]byte, 4)
	binary.LittleEndian.PutUint32(b, uint32(n))
	return b
}

func seg(b []byte) []byte {
	out := le32(len(b))
	return append(out, b...)
}

func hashChunks(domain []byte, chunks ...[]byte) [32]byte {
	h := blake3.New(32, nil)
	h.Write(domain)
	for _, c := range chunks {
		h.Write(c)
	}
	var out [32]byte
	copy(out[:], h.Sum(nil))
	return out
}

func nameHash(short string) [32]byte {
	return hashChunks(nameDomain, seg([]byte(short)))
}

func versionHash(full string, m *descriptorpb.DescriptorProto) [32]byte {
	chunks := make([][]byte, 0, 3+2*len(m.Field))
	chunks = append(chunks, seg([]byte(full)), le32(len(m.Field)))
	for _, f := range m.GetField() {
		chunks = append(chunks, seg([]byte(f.GetName())))
		chunks = append(chunks, le32(int(f.GetNumber())), le32(int(f.GetType())))
		if tn := f.GetTypeName(); tn != "" {
			chunks = append(chunks, seg([]byte(tn)))
		}
	}
	return hashChunks(versionDomain, chunks...)
}

// EnvelopeBytes wraps a message and serializes it to full envelope bytes.
func EnvelopeBytes(msg proto.Message, meta *MessageRef) ([]byte, error) {
	payload, err := proto.Marshal(msg)
	if err != nil {
		return nil, err
	}
	out := make([]byte, 0, HeaderLen+len(payload))
	out = append(out, meta.NameHash[:]...)
	out = append(out, meta.VersionHash[:]...)
	out = append(out, le32(len(payload))...)
	out = append(out, payload...)
	return out, nil
}

// Decode decodes an envelope into msg under the given policy.
func Decode(msg proto.Message, data []byte, meta *MessageRef, policy Policy) error {
	if len(data) < HeaderLen {
		return newErr(KindNotAnEnvelope, "data too short for the envelope header")
	}
	var nameH, versionH [32]byte
	copy(nameH[:], data[0:32])
	copy(versionH[:], data[32:64])
	if policy == Strict {
		if nameH != meta.NameHash {
			return &Error{Kind: KindTypeMismatch, Expected: meta.NameHash, Actual: nameH, Message: "name_hash mismatch"}
		}
		if versionH != meta.VersionHash {
			return &Error{Kind: KindVersionMismatch, Expected: meta.VersionHash, Actual: versionH, Message: "version_hash mismatch"}
		}
	}
	plen := int(binary.LittleEndian.Uint32(data[64:68]))
	if len(data) != HeaderLen+plen {
		return newErr(KindNotAnEnvelope, fmt.Sprintf("payload length mismatch: header says %d but %d remain", plen, len(data)-HeaderLen))
	}
	if err := proto.Unmarshal(data[HeaderLen:], msg); err != nil {
		return &Error{Kind: KindDeserialize, Message: err.Error()}
	}
	return nil
}

// Peek extracts the two hashes from an envelope without decoding the payload.
func Peek(data []byte) ([32]byte, [32]byte, error) {
	if len(data) < HeaderLen {
		return [32]byte{}, [32]byte{}, newErr(KindNotAnEnvelope, "data too short for the envelope header")
	}
	var nameH, versionH [32]byte
	copy(nameH[:], data[0:32])
	copy(versionH[:], data[32:64])
	return nameH, versionH, nil
}
