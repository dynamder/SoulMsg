module smsgverify

go 1.26

require (
	google.golang.org/protobuf v1.36.11
	soulmsg v0.0.0
)

require (
	github.com/klauspost/cpuid/v2 v2.0.9 // indirect
	lukechampine.com/blake3 v1.4.1 // indirect
)

replace soulmsg => ../../../bindings/go/soulmsg
