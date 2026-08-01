#!/usr/bin/env bash
# Fetches the Kotlin/JVM jars (once) and runs the Kotlin verification against the
# `soulmsg` binding.
set -euo pipefail

cd "$(dirname "$0")"
BINDING=../../../bindings/kotlin/soulmsg/SoulMsg.kt

LIBS=(protobuf-java bcprov json kotlin-stdlib kotlin-reflect kotlinc coroutines kotlin-daemon trove4j annotations kotlin-script-runtime)
URLS=(
  "https://repo1.maven.org/maven2/com/google/protobuf/protobuf-java/3.25.5/protobuf-java-3.25.5.jar"
  "https://repo1.maven.org/maven2/org/bouncycastle/bcprov-jdk18on/1.78.1/bcprov-jdk18on-1.78.1.jar"
  "https://repo1.maven.org/maven2/org/json/json/20240303/json-20240303.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-reflect/2.0.21/kotlin-reflect-2.0.21.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-compiler-embeddable/2.0.21/kotlin-compiler-embeddable-2.0.21.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.9.0/kotlinx-coroutines-core-jvm-1.9.0.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-daemon-embeddable/2.0.21/kotlin-daemon-embeddable-2.0.21.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/intellij/deps/trove4j/1.0.20200330/trove4j-1.0.20200330.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/annotations/13.0/annotations-13.0.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-script-runtime/2.0.21/kotlin-script-runtime-2.0.21.jar"
)

mkdir -p lib
for i in "${!LIBS[@]}"; do
  f="lib/${LIBS[$i]}.jar"
  if [ ! -f "$f" ]; then
    curl -sSL -o "$f" "${URLS[$i]}"
  fi
done

# Compile the binding + verifier with the embeddable compiler.
COMPILER_CP="lib/kotlinc.jar;lib/kotlin-stdlib.jar;lib/coroutines.jar;lib/kotlin-reflect.jar;lib/kotlin-script-runtime.jar;lib/kotlin-daemon.jar;lib/trove4j.jar;lib/annotations.jar"
java -cp "$COMPILER_CP" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect \
  -classpath "lib/protobuf-java.jar;lib/bcprov.jar;lib/json.jar;lib/kotlin-stdlib.jar;lib/kotlin-reflect.jar" \
  -d lib/verify.jar "$BINDING" Verify.kt

RUN_CP="lib/verify.jar;lib/protobuf-java.jar;lib/bcprov.jar;lib/json.jar;lib/kotlin-stdlib.jar;lib/kotlin-reflect.jar"
java -cp "$RUN_CP" VerifyKt ..
