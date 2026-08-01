package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"google.golang.org/protobuf/proto"

	"smsgverify/gen/chat"
	soulmsg "soulmsg"
)

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

	schema, err := soulmsg.NewSchema(descData)
	if err != nil {
		panic(err)
	}

	failures := 0
	for fullName, expected := range golden.Messages {
		meta := schema.Message(fullName)
		if meta == nil {
			panic("message not found: " + fullName)
		}

		// Build the same logical message as the Rust example using generated code.
		var msg proto.Message
		switch fullName {
		case "chat.ChatMessage":
			msg = &chat.ChatMessage{
				Sender:    "Alice",
				Content:   "Hello, World!",
				Timestamp: 1_699_999_999,
			}
		case "chat.RobotState":
			msg = &chat.RobotState{
				Name:     "R2-D2",
				Position: &chat.Position{X: 100.5, Y: 200.5, Z: 300.5},
				Status:   42,
			}
		default:
			panic("unknown fixture message: " + fullName)
		}

		wire, err := soulmsg.EnvelopeBytes(msg, meta)
		if err != nil {
			panic(err)
		}
		// Round-trip through the binding (strict).
		switch fullName {
		case "chat.ChatMessage":
			var out chat.ChatMessage
			if err := soulmsg.Decode(&out, wire, meta, soulmsg.Strict); err != nil {
				panic(err)
			}
		case "chat.RobotState":
			var out chat.RobotState
			if err := soulmsg.Decode(&out, wire, meta, soulmsg.Strict); err != nil {
				panic(err)
			}
		}
		payload, err := proto.Marshal(msg)
		if err != nil {
			panic(err)
		}

		checks := map[string]string{
			"name_hash":    hex.EncodeToString(meta.NameHash[:]),
			"version_hash": hex.EncodeToString(meta.VersionHash[:]),
			"payload":      hex.EncodeToString(payload),
			"envelope":     hex.EncodeToString(wire),
		}
		want := map[string]string{
			"name_hash":    expected.NameHash,
			"version_hash": expected.VersionHash,
			"payload":      expected.Payload,
			"envelope":     expected.Envelope,
		}
		ok := true
		for k, v := range checks {
			if v != want[k] {
				fmt.Printf("[go] %s.%s MISMATCH\n  rust = %s\n  go   = %s\n", fullName, k, want[k], v)
				ok = false
				failures++
			}
		}
		fmt.Printf("[go] %s: match (via soulmsg binding)\n", fullName)
		if !ok {
			failures++
		}
	}

	if failures > 0 {
		fmt.Printf("\n[go] FAIL: %d mismatches\n", failures)
		os.Exit(1)
	}
	fmt.Println("\n[go] PASS: byte-for-byte identical to Rust (via soulmsg binding + generated code)")
}
