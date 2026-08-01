package main

import (
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"lukechampine.com/blake3"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/dynamicpb"
)

var (
	nameDomain    = []byte("smsg_proto:name:v1\x00")
	versionDomain = []byte("smsg_proto:version:v1\x00")
)

func le32(n int) []byte {
	b := make([]byte, 4)
	binary.LittleEndian.PutUint32(b, uint32(n))
	return b
}

func seg(b []byte) []byte {
	out := le32(len(b))
	out = append(out, b...)
	return out
}

func nameHash(short string) [32]byte {
	h := blake3.New(32, nil)
	h.Write(nameDomain)
	h.Write(seg([]byte(short)))
	var out [32]byte
	copy(out[:], h.Sum(nil))
	return out
}

func versionHash(full string, m *descriptorpb.DescriptorProto) [32]byte {
	h := blake3.New(32, nil)
	h.Write(versionDomain)
	h.Write(seg([]byte(full)))
	h.Write(le32(len(m.Field)))
	for _, f := range m.Field {
		h.Write(seg([]byte(f.GetName())))
		h.Write(le32(int(f.GetNumber())))
		h.Write(le32(int(f.GetType())))
		if tn := f.GetTypeName(); tn != "" {
			h.Write(seg([]byte(tn)))
		}
	}
	var out [32]byte
	copy(out[:], h.Sum(nil))
	return out
}

func envelopeBytes(name, version [32]byte, payload []byte) []byte {
	out := make([]byte, 0, 68+len(payload))
	out = append(out, name[:]...)
	out = append(out, version[:]...)
	out = append(out, le32(len(payload))...)
	out = append(out, payload...)
	return out
}

func main() {
	dir, _ := os.Getwd()
	verif := filepath.Dir(dir)

	descData, err := os.ReadFile(filepath.Join(verif, "descriptors.pb"))
	if err != nil {
		panic(err)
	}
	goldenRaw, err := os.ReadFile(filepath.Join(verif, "golden.json"))
	if err != nil {
		panic(err)
	}

	var golden struct {
		Messages map[string]struct {
			NameHash    string `json:"name_hash"`
			VersionHash string `json:"version_hash"`
			Payload     string `json:"payload"`
			Envelope    string `json:"envelope"`
		} `json:"messages"`
	}
	if err := json.Unmarshal(goldenRaw, &golden); err != nil {
		panic(err)
	}

	var fds descriptorpb.FileDescriptorSet
	if err := proto.Unmarshal(descData, &fds); err != nil {
		panic(err)
	}
	files, err := protodesc.NewFiles(&fds)
	if err != nil {
		panic(err)
	}

	failures := 0
	for fullName, expected := range golden.Messages {
		short := fullName[strings.LastIndex(fullName, ".")+1:]

		// Find the DescriptorProto to compute the canonical hash.
		var msgDesc *descriptorpb.DescriptorProto
		for _, f := range fds.File {
			for _, m := range f.MessageType {
				if f.GetPackage()+"."+m.GetName() == fullName {
					msgDesc = m
				}
			}
		}
		if msgDesc == nil {
			panic("message not found: " + fullName)
		}

		nameH := nameHash(short)
		versionH := versionHash(fullName, msgDesc)

		// Build the dynamic message and encode it.
		md, err := files.FindDescriptorByName(protoreflect.FullName(fullName))
		if err != nil {
			panic(err)
		}
		msgDesc2 := md.(protoreflect.MessageDescriptor)
		dm := dynamicpb.NewMessage(msgDesc2)
		fields := msgDesc2.Fields()
		switch fullName {
		case "chat.ChatMessage":
			dm.Set(fields.ByName("sender"), protoreflect.ValueOfString("Alice"))
			dm.Set(fields.ByName("content"), protoreflect.ValueOfString("Hello, World!"))
			dm.Set(fields.ByName("timestamp"), protoreflect.ValueOfInt64(1_699_999_999))
		case "chat.RobotState":
			pos := dynamicpb.NewMessage(fields.ByName("position").Message())
			pos.Set(pos.Descriptor().Fields().ByName("x"), protoreflect.ValueOfFloat64(100.5))
			pos.Set(pos.Descriptor().Fields().ByName("y"), protoreflect.ValueOfFloat64(200.5))
			pos.Set(pos.Descriptor().Fields().ByName("z"), protoreflect.ValueOfFloat64(300.5))
			dm.Set(fields.ByName("name"), protoreflect.ValueOfString("R2-D2"))
			dm.Set(fields.ByName("position"), protoreflect.ValueOfMessage(pos))
			dm.Set(fields.ByName("status"), protoreflect.ValueOfInt32(42))
		}
		payload, err := proto.Marshal(dm)
		if err != nil {
			panic(err)
		}
		envelope := envelopeBytes(nameH, versionH, payload)

		checks := map[string]string{
			"name_hash":    hex.EncodeToString(nameH[:]),
			"version_hash": hex.EncodeToString(versionH[:]),
			"payload":      hex.EncodeToString(payload),
			"envelope":     hex.EncodeToString(envelope),
		}
		ok := true
		for k, v := range checks {
			want := map[string]string{
				"name_hash":    expected.NameHash,
				"version_hash": expected.VersionHash,
				"payload":      expected.Payload,
				"envelope":     expected.Envelope,
			}[k]
			if v != want {
				fmt.Printf("[go] %s.%s MISMATCH\n  rust = %s\n  go   = %s\n", fullName, k, want, v)
				ok = false
				failures++
			}
		}
		if ok {
			fmt.Printf("[go] %s: name/version/payload/envelope all match rust\n", fullName)
		}
	}

	if failures > 0 {
		fmt.Printf("\n[go] FAIL: %d mismatches\n", failures)
		os.Exit(1)
	}
	fmt.Println("\n[go] PASS: byte-for-byte identical to Rust for all messages")
}
