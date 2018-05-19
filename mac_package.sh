#!/usr/bin/env bash

set -eu

rm -rf build
mkdir -p build/3frac.app/Contents/MacOS

cargo build --release

cp target/release/3frac build/3frac.app/Contents/MacOS

cat > build/3frac.app/Contents/Info.plis << EOF
{
	CFBundleName = 3frac;
	CFBundleDisplayName = 3frac;
	CFBundleIdentifier = "com.avaglir.3frac";
	CFBundleVersion = "0.1.0";
	CFBundleInfoDictionaryVersion = "6.0";
	CFBundlePackageType = APPL;
	CFBundleExecutable = 3frac;
}
EOF
