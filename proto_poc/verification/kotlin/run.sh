#!/usr/bin/env bash
# Fetches the Kotlin/JVM jars (once) and runs the Kotlin verification.
set -euo pipefail

cd "$(dirname "$0")"

LIBS=(protobuf-java bcprov json kotlin-stdlib kotlin-reflect)
URLS=(
  "https://repo1.maven.org/maven2/com/google/protobuf/protobuf-java/3.25.5/protobuf-java-3.25.5.jar"
  "https://repo1.maven.org/maven2/org/bouncycastle/bcprov-jdk18on/1.78.1/bcprov-jdk18on-1.78.1.jar"
  "https://repo1.maven.org/maven2/org/json/json/20240303/json-20240303.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar"
  "https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-reflect/2.0.21/kotlin-reflect-2.0.21.jar"
)

mkdir -p lib
for i in "${!LIBS[@]}"; do
  f="lib/${LIBS[$i]}.jar"
  if [ ! -f "$f" ]; then
    curl -sSL -o "$f" "${URLS[$i]}"
  fi
done

RUN_CP="lib/verify.jar;lib/protobuf-java.jar;lib/bcprov.jar;lib/json.jar;lib/kotlin-stdlib.jar;lib/kotlin-reflect.jar"
java -cp "$RUN_CP" VerifyKt ..
